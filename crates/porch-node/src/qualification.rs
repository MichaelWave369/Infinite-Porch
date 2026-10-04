//! Operator-run evidence. A declared physical environment is an attestation,
//! never an automatic claim that two computers or a disconnected WAN exist.
use crate::{Node, db::Db, models, network, safe_error};
use anyhow::{Context, Result, ensure};
use porch_core::*;
use rusqlite::params;
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path, sync::Arc};

pub const PUBLIC_PROMPT: &str =
    "Infinite Porch public qualification probe. Reply briefly with PORCH_OK.";
pub fn is_physical(value: &str) -> bool {
    matches!(value, "PHYSICAL" | "PHYSICAL_LAN" | "PHYSICAL_WAN")
}
pub fn valid_class(value: &str) -> bool {
    matches!(
        value,
        "SIMULATED" | "LOOPBACK" | "NATIVE_HOSTED" | "PHYSICAL_LAN" | "PHYSICAL_WAN"
    )
}
pub fn build() -> Value {
    json!({"software_version":env!("CARGO_PKG_VERSION"),"source_tree_dirty_at_build":env!("PORCH_BUILD_TREE_DIRTY")=="true","protocol_version":VERSION,
        "commit":env!("PORCH_BUILD_COMMIT"),"branch":env!("PORCH_BUILD_BRANCH"),
        "target":env!("PORCH_BUILD_TARGET"),"release_status":"CANDIDATE",
        "independent_security_review":"UNVERIFIED","physical_qualification":"UNVERIFIED"})
}
pub fn fingerprint(public_key: &str) -> Result<String> {
    Ok(digest(&hex::decode(public_key)?))
}
pub fn local_fingerprint(node: &Node) -> String {
    digest(&node.key.public().encode_protobuf())
}
pub fn status(node: &Node) -> Result<Value> {
    let jobs = node.db.records("jobs")?;
    let latest=jobs.first().map(|v|json!({"receipt_hash":digest(&canonical(v).unwrap_or_default()),"executor":v["payload"]["executor"],"status":v["payload"]["status"],"provider":v["payload"]["provider"],"input_digest":v["payload"]["input_digest"],"output_digest":v["payload"]["output_digest"]}));
    let results = node.db.records("qualification")?;
    let last_physical = results
        .iter()
        .find(|r| is_physical(r["payload"]["environment"].as_str().unwrap_or("")));
    Ok(
        json!({"build":build(),"node":{"peer_id":node.id,"alias":node.config.read().unwrap().alias,"fingerprint_sha256":local_fingerprint(node)},
        "porch":node.db.get("porch")?,"network":network::diagnostics(node),"peers":node.db.trust_rows()?,
        "models":models::states(node)?,"grants":node.db.grants()?.iter().map(|g|json!({"nonce":g["grant"]["payload"]["nonce"],"issuer":g["grant"]["payload"]["issuer"],"recipient":g["grant"]["payload"]["recipient"],"capability":g["grant"]["payload"]["capability"],"resource":g["grant"]["payload"]["resource"],"expires_at":g["grant"]["payload"]["expires_at"],"revoked":g["revoked"],"consumed_calls":g["consumed_calls"]})).collect::<Vec<_>>(),
        "active_jobs":node.db.active_jobs()?,"latest_receipt":latest,"ledger_verified":node.db.verify_ledger(&node.id).is_ok(),
        "results":results,"last_physical_qualification":last_physical,"gates":gates(node),"readiness":["CANDIDATE","PLATFORM_PARTIAL","SECURITY_REVIEW_PENDING"],
        "physical_environment_is_operator_attested":true,"production_qualified":false}),
    )
}
fn scenario(
    id: &str,
    expectation: &str,
    observation: Value,
    result: &str,
    peers: &[String],
) -> Value {
    json!({"id":id,"expectation":expectation,"observation":observation,"result":result,"evidence_file":"run.json","timestamp":now(),"peers":peers})
}
pub async fn run(node: &Arc<Node>, args: &Value) -> Result<Value> {
    let environment = args["environment"].as_str().unwrap_or("LOOPBACK");
    ensure!(
        matches!(
            environment,
            "SIMULATED"
                | "LOOPBACK"
                | "NATIVE_HOSTED"
                | "PHYSICAL"
                | "PHYSICAL_LAN"
                | "PHYSICAL_WAN"
        ),
        "INVALID_QUALIFICATION_ENVIRONMENT"
    );
    ensure!(
        environment != "PHYSICAL",
        "LEGACY_PHYSICAL_CLASS_READ_ONLY_USE_PHYSICAL_LAN"
    );
    let phase = args["phase"].as_str().unwrap_or("baseline");
    ensure!(
        matches!(
            phase,
            "baseline" | "revoked" | "offline" | "restored" | "restart"
        ),
        "INVALID_QUALIFICATION_PHASE"
    );
    if is_physical(environment) {
        ensure!(
            std::env::var("GITHUB_ACTIONS").as_deref() != Ok("true"),
            "HOSTED_RUNNER_CANNOT_CLAIM_PHYSICAL_CLASS"
        );
        ensure!(
            args["separate_machines_confirmed"] == true,
            "PHYSICAL_REQUIRES_OPERATOR_ATTESTATION"
        );
    }
    let peer = args["peer"].as_str();
    if let Some(p) = peer {
        let _: libp2p::PeerId = p.parse()?;
    }
    let mut peers = vec![node.id.clone()];
    if let Some(p) = peer {
        peers.push(p.to_string());
    }
    let run_id = nonce();
    let session = args["session"].as_str();
    if let Some(value) = session {
        ensure!(crate::valid_label(value, 64), "INVALID_FIELD_SESSION");
    }
    let started = std::time::Instant::now();
    let mut scenarios = vec![];
    scenarios.push(scenario(
        "identity",
        "Persisted cryptographic identity matches this node",
        json!({"peer_id":node.id,"fingerprint_sha256":local_fingerprint(node)}),
        "PASS",
        &peers,
    ));
    let ledger = node.db.verify_ledger(&node.id).is_ok();
    scenarios.push(scenario(
        "ledger",
        "Entire local signed ledger verifies",
        json!({"verified":ledger}),
        if ledger { "PASS" } else { "FAIL" },
        &peers,
    ));
    let inventory = models::scan(node).await?;
    scenarios.push(scenario("ollama_local","Probe local provider without enabling sharing",json!({"reachable":inventory["currently_reachable"],"model_count":inventory["inventory"].as_array().map(Vec::len)}),if inventory["currently_reachable"]==true{"PASS"}else{"SKIPPED_ENVIRONMENT"},&peers));
    let before = node.observations.read().unwrap()["outbound_job_requests"]
        .as_u64()
        .unwrap_or(0);
    let local=node.run(&json!({"capability":"compute.hash","resource":"sha256","input":PUBLIC_PROMPT,"privacy":"LOCAL_ONLY","max_output_tokens":16,"timeout_ms":1000})).await;
    let after = node.observations.read().unwrap()["outbound_job_requests"]
        .as_u64()
        .unwrap_or(0);
    scenarios.push(scenario("local_only","LOCAL_ONLY generates no outbound remote job even with peers available",json!({"outbound_before":before,"outbound_after":after,"local_status":local.as_ref().ok().map(|v|v["status"].clone())}),if before==after && local.as_ref().is_ok_and(|v|v["status"]=="COMPLETED"){"PASS"}else{"FAIL"},&peers));
    let mut receipts = vec![];
    if let Ok(v) = local
        && !v["receipt"].is_null()
    {
        receipts.push(v["receipt"].clone());
    }
    let mut route = Value::Null;
    if let Some(peer) = peer {
        let _ = node.refresh().await;
        let trusted = node.db.trusted(peer)?;
        scenarios.push(scenario(
            "peer_trust",
            "Intended peer has explicit active trust",
            json!({"trusted":trusted}),
            if trusted { "PASS" } else { "FAIL" },
            &peers,
        ));
        let path = network::diagnostics(node)["peers"][peer].clone();
        let connected = path["connected"] == true;
        scenarios.push(scenario(
            "peer_path",
            "Authenticated encrypted connection to the intended peer",
            path.clone(),
            if connected && path["authenticated"] == true && path["encrypted"] == true {
                "PASS"
            } else {
                "UNVERIFIED"
            },
            &peers,
        ));
        let physical_path = path["path"] == "LAN_DIRECT" || path["path"] == "WAN_DIRECT";
        scenarios.push(scenario("separate_physical_nodes","Two separate computers confirmed by the operator, with a non-loopback direct path",json!({"operator_attested":args["separate_machines_confirmed"]==true,"observed_path":path["path"],"independent_physical_verification":false}),if is_physical(environment) && physical_path{"PARTIAL"}else{"UNVERIFIED"},&peers));
        if let Some(model) = args["model"].as_str() {
            ensure!(crate::valid_label(model, 128), "INVALID_MODEL_NAME");
            let args = json!({"resource":model,"input":PUBLIC_PROMPT,"privacy":"TRUSTED_PEERS","preferred_peer":peer,"max_output_tokens":16,"timeout_ms":node.config.read().unwrap().limits.model_timeout_ms,"id":session.map(|s|format!("field-{s}-{phase}-{}",nonce()))});
            let result = node.run(&args).await;
            let expected_refusal = phase == "revoked";
            match result {
                Ok(v) => {
                    route = v["route"].clone();
                    let completed = v["status"] == "COMPLETED";
                    let real = v["receipt"]["payload"]["provider"] == "ollama";
                    let result = if expected_refusal {
                        if v["status"] == "REFUSED"
                            && v["receipt"]["payload"]["reason"]
                                .as_str()
                                .is_some_and(|code| code.contains("REVOKED"))
                        {
                            "REFUSED_EXPECTED"
                        } else {
                            "FAIL"
                        }
                    } else if completed && real {
                        "PASS"
                    } else if completed {
                        "PARTIAL"
                    } else {
                        "FAIL"
                    };
                    scenarios.push(scenario("remote_inference",if expected_refusal{"A new job after grant revocation fails closed"}else{"The selected remote executor completes real Ollama inference with a valid signed receipt"},json!({"status":v["status"],"reason":v["reason"],"executor":v["receipt"]["payload"]["executor"],"provider":v["receipt"]["payload"]["provider"],"receipt_hash":if v["receipt"].is_null(){None}else{Some(digest(&canonical(&v["receipt"])?))},"route":route}),result,&peers));
                    if !v["receipt"].is_null() {
                        receipts.push(v["receipt"].clone());
                    }
                }
                Err(e) => scenarios.push(scenario(
                    "remote_inference",
                    "A remote diagnostic job either completes or records bounded refusal",
                    json!({"reason":safe_error(&e)}),
                    "FAIL",
                    &peers,
                )),
            };
        } else {
            scenarios.push(scenario(
                "remote_inference",
                "Explicit --model selects actual remote inference",
                json!({"reason":"MODEL_NOT_SELECTED"}),
                "SKIPPED_ENVIRONMENT",
                &peers,
            ));
        }
        if args["message"] == true {
            let delivery=tokio::time::timeout(std::time::Duration::from_secs(5),node.send_message(&json!({"peer":peer,"text":"Infinite Porch public qualification message","queue_if_offline":false}))).await.unwrap_or_else(|_|Err(anyhow::anyhow!("QUALIFICATION_MESSAGE_TIMEOUT")));
            match delivery {
                Ok(v) => {
                    scenarios.push(scenario(
                        "messaging",
                        "Public diagnostic message returns a signed delivery receipt",
                        json!({"status":v["status"]}),
                        if v["status"] == "DELIVERED" {
                            "PASS"
                        } else {
                            "FAIL"
                        },
                        &peers,
                    ));
                    receipts.push(v["receipt"].clone());
                }
                Err(e) => scenarios.push(scenario(
                    "messaging",
                    "Bounded message grant authorizes delivery",
                    json!({"reason":safe_error(&e)}),
                    "FAIL",
                    &peers,
                )),
            }
        } else {
            scenarios.push(scenario(
                "messaging",
                "--message exercises a separately granted inbox",
                json!({"reason":"MESSAGE_NOT_SELECTED"}),
                "SKIPPED_ENVIRONMENT",
                &peers,
            ));
        }
        let advertisement = node
            .db
            .records("advertisements")?
            .into_iter()
            .find(|a| a["signer"] == peer);
        scenarios.push(scenario("signed_discovery","Fresh signed service advertisement is bound to the selected peer",json!({"peer":peer,"advertisement_hash":advertisement.as_ref().map(|v|digest(&canonical(v).unwrap_or_default()))}),if advertisement.as_ref().is_some_and(|v|serde_json::from_value::<Signed<Advertisement>>(v.clone()).is_ok_and(|a|a.verify("porch.advertisement.v1").is_ok() && a.signer==peer && a.payload.expires_at>now())){"PASS"}else{"FAIL"},&peers));
        if let Some(model) = args["model"].as_str() {
            let grant = node.remote_grant(peer, "model.inference", model, "run")?;
            scenarios.push(scenario("bounded_authorization","An explicit bounded grant selects this model and recipient",json!({"grant_nonce":grant.as_ref().map(|g|g.payload.nonce.clone()),"limits":grant.as_ref().map(|g|g.payload.limits.clone())}),if grant.is_some(){"PASS"}else{"FAIL"},&peers));
        }
        if args["storage"] == true {
            let result=async {
                let blob=crate::storage::put_local(node,PUBLIC_PROMPT.as_bytes())?;
                let cid=blob["cid"].as_str().context("FIELD_BLOB_CID_REQUIRED")?;
                let replicated=crate::storage::replicate(node,peer,cid).await?;
                let fetched=crate::storage::fetch(node,peer,cid).await?;
                Ok::<Value,anyhow::Error>(json!({"cid":cid,"owner":node.id,"peer":peer,"verified_replicas":replicated["verified_replicas"],"retrieved_digest":digest(&hex::decode(fetched["plaintext_hex"].as_str().unwrap_or(""))?),"integrity":fetched["integrity"]}))
            }.await;
            let observation = result
                .as_ref()
                .cloned()
                .unwrap_or_else(|_| json!({"reason":"FIELD_STORAGE_REFUSED"}));
            let ok = result.as_ref().is_ok_and(|v| {
                v["verified_replicas"] == 1
                    && v["integrity"] == "VERIFIED"
                    && v["retrieved_digest"] == digest(PUBLIC_PROMPT.as_bytes())
            });
            for id in ["encrypted_storage", "storage_integrity"] {
                scenarios.push(scenario(
                    id,
                    "Transfer encrypted public test bytes and retrieve with integrity verification",
                    observation.clone(),
                    if ok { "PASS" } else { "FAIL" },
                    &peers,
                ));
            }
        }
        if phase == "offline" || phase == "restored" {
            let local = path["path"] == "LAN_DIRECT";
            scenarios.push(scenario("wan_condition","Human disconnects/restores upstream WAN while preserving LAN; compare both peers' evidence",json!({"phase":phase,"operator_attested":args["wan_condition_confirmed"]==true,"observed_path":path["path"],"wan_condition_automatically_proven":false}),if is_physical(environment) && local && args["wan_condition_confirmed"]==true{"PARTIAL"}else{"UNVERIFIED"},&peers));
        }
    } else {
        scenarios.push(scenario(
            "physical_peer",
            "Explicit --peer selects the other machine",
            json!({"reason":"PEER_NOT_SELECTED"}),
            "SKIPPED_ENVIRONMENT",
            &peers,
        ));
    }
    for (id, expectation) in [
        (
            "restart_recovery",
            "Compare pre/post restart identity, trust, revocation, storage and ledger",
        ),
        (
            "independent_security_review",
            "Independent reviewer evaluates the physical-network attack surface",
        ),
        (
            "windows_verification",
            "Execute native Windows qualification",
        ),
        (
            "macos_verification",
            "Execute native macOS qualification if claiming support",
        ),
    ] {
        scenarios.push(scenario(
            id,
            expectation,
            json!({"external_evidence_required":true}),
            "UNVERIFIED",
            &peers,
        ));
    }
    let mut observed_network = network::diagnostics(node);
    if let Some(paths) = observed_network["peers"].as_object_mut() {
        paths.retain(|id, _| peers.contains(id));
    }
    let payload = json!({"schema_version":1,"run_id":run_id,"timestamp":now(),"build":build(),"environment":environment,"phase":phase,"operator_attested_separate_machines":args["separate_machines_confirmed"]==true,"peers":peers,"scenarios":scenarios,"signed_receipts":receipts,"route":route,"network":observed_network,"duration_ms":started.elapsed().as_millis() as u64,"session":session,"prompt_policy":"fixed public diagnostic only","production_qualified":false});
    let signed = Signed::new(&node.key, "porch.qualification.v1", payload)?;
    node.db.transaction(|tx|{
        tx.execute("INSERT INTO qualification(id,record) VALUES(?1,?2)",params![run_id,serde_json::to_string(&signed)?])?;
        tx.execute("DELETE FROM qualification WHERE id NOT IN (SELECT id FROM qualification ORDER BY rowid DESC LIMIT 64)",[])?;
        Db::append(tx,&node.key,"qualification.recorded",&run_id,json!({"run_hash":signed.hash()?,"environment":environment,"phase":phase}))?;Ok(())
    })?;
    Ok(serde_json::to_value(signed)?)
}

/// Whitelisted evidence files only: no general log scan, database or key copy.
pub fn export(node: &Node) -> Result<Value> {
    let records = node.db.records("qualification")?;
    let latest = records.first().context("RUN_QUALIFICATION_BEFORE_EXPORT")?;
    let run: Signed<Value> = serde_json::from_value(latest.clone())?;
    run.verify("porch.qualification.v1")?;
    node.db.verify_ledger(&node.id)?;
    let mut files = BTreeMap::<String, Value>::new();
    files.insert("run.json".into(), latest.clone());
    files.insert("node.json".into(),json!({"build":build(),"peer_id":node.id,"alias":node.config.read().unwrap().alias,"fingerprint_sha256":local_fingerprint(node)}));
    // Full-chain signatures are verifiable; stored subjects have been validated.
    let ledger = node.db.transaction(|tx| {
        let count: u64 = tx.query_row("SELECT count(*) FROM ledger", [], |r| r.get(0))?;
        ensure!(
            count <= 20000,
            "LEDGER_EXPORT_LIMIT_USE_A_FRESH_QUALIFICATION_NODE"
        );
        let mut s = tx.prepare("SELECT record FROM ledger ORDER BY sequence")?;
        Ok(s.query_map([], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?)
    })?;
    // Baseline ledgers may contain arbitrary old refusal strings; export a signed
    // tip checkpoint rather than copying potentially private historical logs.
    let tip = ledger
        .last()
        .map(|r| serde_json::from_str::<Value>(r))
        .transpose()?;
    files.insert("ledger-verification.json".into(),serde_json::to_value(Signed::new(&node.key,"porch.qualification.ledger.v1",json!({"verified":true,"record_count":ledger.len(),"tip_hash":tip.as_ref().map(|v|digest(&canonical(v).unwrap_or_default())),"verified_at":now(),"verifier_peer":node.id,"entire_chain_exported":false}))?)?);
    files.insert("results.json".into(), run.payload["scenarios"].clone());
    files.insert(
        "receipts.json".into(),
        run.payload["signed_receipts"].clone(),
    );
    files.insert("sanitized-log.json".into(),json!({"source":"qualification-only","private_prompts_exported":false,"private_outputs_exported":false,"public_probe_outputs_exported":true,"events":run.payload["scenarios"].as_array().unwrap().iter().map(|s|json!({"id":s["id"],"result":s["result"],"timestamp":s["timestamp"]})).collect::<Vec<_>>()}));
    let hashes = files
        .iter()
        .map(|(n, v)| Ok((n.clone(), digest(&serde_json::to_vec_pretty(v)?))))
        .collect::<Result<BTreeMap<_, _>>>()?;
    let manifest = Signed::new(
        &node.key,
        "porch.qualification.manifest.v1",
        json!({"schema_version":1,"run_id":run.payload["run_id"],"created_at":now(),"peer_id":node.id,"environment":run.payload["environment"],"files":hashes,"signed_by_node_identity":true,"official_release_signature":false,"production_qualified":false}),
    )?;
    Ok(json!({"manifest":manifest,"files":files}))
}
pub fn write_export(directory: &Path, bundle: &Value) -> Result<()> {
    validate_export(bundle)?;
    ensure!(!directory.exists(), "EVIDENCE_DIRECTORY_ALREADY_EXISTS");
    std::fs::create_dir_all(directory)?;
    let result = (|| -> Result<()> {
        for (name, value) in bundle["files"]
            .as_object()
            .context("INVALID_EVIDENCE_FILES")?
        {
            ensure!(
                matches!(
                    name.as_str(),
                    "run.json"
                        | "node.json"
                        | "results.json"
                        | "receipts.json"
                        | "ledger-verification.json"
                        | "sanitized-log.json"
                ),
                "INVALID_EVIDENCE_FILENAME"
            );
            let bytes = serde_json::to_vec_pretty(value)?;
            ensure!(
                bundle["manifest"]["payload"]["files"][name].as_str()
                    == Some(digest(&bytes).as_str()),
                "EVIDENCE_HASH_MISMATCH"
            );
            private_write(&directory.join(name), &bytes)?;
        }
        private_write(
            &directory.join("manifest.json"),
            &serde_json::to_vec_pretty(&bundle["manifest"])?,
        )?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_dir_all(directory);
    }
    result
}

pub fn read_export(path: &Path) -> Result<Value> {
    if path.is_file() {
        ensure!(
            std::fs::metadata(path)?.len() <= 32 * 1024 * 1024,
            "EVIDENCE_TOO_LARGE"
        );
        return Ok(serde_json::from_slice(&std::fs::read(path)?)?);
    }
    let manifest_path = path.join("manifest.json");
    let mut total = std::fs::metadata(&manifest_path)?.len();
    ensure!(total <= 64 * 1024, "EVIDENCE_MANIFEST_TOO_LARGE");
    let manifest: Value = serde_json::from_slice(&std::fs::read(manifest_path)?)?;
    let mut files = BTreeMap::new();
    for name in manifest["payload"]["files"]
        .as_object()
        .context("INVALID_MANIFEST_FILES")?
        .keys()
    {
        ensure!(allowed_file(name), "INVALID_EVIDENCE_FILENAME");
        let file = path.join(name);
        ensure!(
            std::fs::metadata(&file)?.len() <= 16 * 1024 * 1024,
            "EVIDENCE_TOO_LARGE"
        );
        total = total
            .checked_add(std::fs::metadata(&file)?.len())
            .context("EVIDENCE_TOO_LARGE")?;
        ensure!(total <= 32 * 1024 * 1024, "EVIDENCE_TOO_LARGE");
        files.insert(
            name.clone(),
            serde_json::from_slice::<Value>(&std::fs::read(file)?)?,
        );
    }
    Ok(json!({"manifest":manifest,"files":files}))
}
fn allowed_file(name: &str) -> bool {
    matches!(
        name,
        "run.json"
            | "node.json"
            | "results.json"
            | "receipts.json"
            | "ledger-verification.json"
            | "sanitized-log.json"
    )
}
pub fn validate_export(bundle: &Value) -> Result<Value> {
    let manifest: Signed<Value> = serde_json::from_value(bundle["manifest"].clone())?;
    manifest.verify("porch.qualification.manifest.v1")?;
    ensure!(
        manifest.payload["schema_version"] == 1
            && manifest.payload["production_qualified"] == false,
        "INVALID_MANIFEST_SCHEMA"
    );
    let files = bundle["files"]
        .as_object()
        .context("INVALID_EVIDENCE_FILES")?;
    let hashes = manifest.payload["files"]
        .as_object()
        .context("INVALID_MANIFEST_FILES")?;
    ensure!(
        files.len() == 6 && files.len() == hashes.len(),
        "EVIDENCE_FILE_SET_MISMATCH"
    );
    for (name, value) in files {
        ensure!(allowed_file(name), "INVALID_EVIDENCE_FILENAME");
        ensure!(
            hashes.get(name).context("MISSING_EVIDENCE_HASH")?.as_str()
                == Some(digest(&serde_json::to_vec_pretty(value)?).as_str()),
            "EVIDENCE_HASH_MISMATCH"
        );
    }
    let run: Signed<Value> = serde_json::from_value(files["run.json"].clone())?;
    run.verify("porch.qualification.v1")?;
    ensure!(
        run.signer == manifest.signer
            && files["node.json"]["peer_id"] == run.signer
            && run.payload["schema_version"] == 1
            && run.payload["production_qualified"] == false
            && run.payload["run_id"] == manifest.payload["run_id"]
            && run.payload["environment"] == manifest.payload["environment"],
        "EVIDENCE_BINDING_MISMATCH"
    );
    ensure!(
        matches!(
            run.payload["environment"].as_str(),
            Some(
                "SIMULATED"
                    | "LOOPBACK"
                    | "NATIVE_HOSTED"
                    | "PHYSICAL"
                    | "PHYSICAL_LAN"
                    | "PHYSICAL_WAN"
            )
        ),
        "INVALID_ENVIRONMENT"
    );
    if is_physical(run.payload["environment"].as_str().unwrap_or("")) {
        ensure!(
            run.payload["operator_attested_separate_machines"] == true,
            "PHYSICAL_ATTESTATION_MISSING"
        );
    }
    let results = files["results.json"]
        .as_array()
        .context("INVALID_SCENARIOS")?;
    ensure!(
        !results.is_empty()
            && results.len() <= 64
            && files["results.json"] == run.payload["scenarios"]
            && files["receipts.json"] == run.payload["signed_receipts"],
        "SCENARIO_BINDING_MISMATCH"
    );
    let mut counts = BTreeMap::<String, usize>::new();
    let mut ids = std::collections::HashSet::new();
    for s in results {
        let state = s["result"].as_str().context("SCENARIO_RESULT_REQUIRED")?;
        ensure!(
            matches!(
                state,
                "PASS"
                    | "FAIL"
                    | "REFUSED_EXPECTED"
                    | "SKIPPED_ENVIRONMENT"
                    | "UNVERIFIED"
                    | "PARTIAL"
            ),
            "INVALID_SCENARIO_RESULT"
        );
        let id = s["id"].as_str().context("SCENARIO_ID_REQUIRED")?;
        ensure!(
            ids.insert(id)
                && crate::valid_label(id, 64)
                && s["expectation"]
                    .as_str()
                    .is_some_and(|v| crate::valid_label(v, 512))
                && s["observation"].is_object()
                && s["timestamp"].as_u64().is_some()
                && s["evidence_file"] == "run.json"
                && s["peers"].as_array().is_some_and(|p| !p.is_empty()
                    && p.iter().all(|v| v
                        .as_str()
                        .is_some_and(|v| v.parse::<libp2p::PeerId>().is_ok()))),
            "INVALID_SCENARIO_SCHEMA"
        );
        *counts.entry(state.into()).or_default() += 1;
    }
    for receipt in files["receipts.json"]
        .as_array()
        .context("INVALID_RECEIPTS")?
    {
        if receipt["domain"] == "porch.job.receipt.v1" {
            let r: Signed<Receipt> = serde_json::from_value(receipt.clone())?;
            r.verify("porch.job.receipt.v1")?;
            ensure!(
                r.payload.executor == r.signer
                    && r.payload.requester == run.signer
                    && r.payload.input_digest == digest(PUBLIC_PROMPT.as_bytes())
                    && r.payload.output_digest == digest(&canonical(&r.payload.output)?),
                "QUALIFICATION_RECEIPT_BINDING_MISMATCH"
            );
        } else {
            let r: Signed<Value> = serde_json::from_value(receipt.clone())?;
            r.verify("porch.message.receipt.v1")?;
            ensure!(
                r.payload["sender"] == run.signer && r.payload["recipient"] == r.signer,
                "MESSAGE_RECEIPT_BINDING_MISMATCH"
            );
        }
    }
    let ledger: Signed<Value> = serde_json::from_value(files["ledger-verification.json"].clone())?;
    ledger.verify("porch.qualification.ledger.v1")?;
    ensure!(
        ledger.signer == run.signer && ledger.payload["verified"] == true,
        "LEDGER_VERIFICATION_BINDING_MISMATCH"
    );
    Ok(
        json!({"valid":true,"environment":run.payload["environment"],"scenario_counts":counts,"skip_counts_as_pass":false,"production_qualified":false,"official_release_signature":false}),
    )
}

pub async fn doctor(node: &Node) -> Value {
    let scan = models::scan(node).await;
    let mut checks = vec![];
    let mut add = |category: &str, result: &str, observation: Value, remediation: &str| {
        checks.push(json!({"category":category,"result":result,"observation":observation,"remediation":remediation}))
    };
    add(
        "IDENTITY",
        "PASS",
        json!({"peer_id":node.id,"fingerprint":local_fingerprint(node)}),
        "Protect state directory and encrypted identity backups.",
    );
    let db_ok = node
        .db
        .transaction(|tx| Ok(tx.query_row("PRAGMA quick_check", [], |r| r.get::<_, String>(0))?))
        .is_ok_and(|v| v == "ok");
    add(
        "DATABASE",
        if db_ok { "PASS" } else { "FAIL" },
        json!({"schema_version":2,"quick_check":db_ok}),
        "Stop node; preserve state and migration backup before repair.",
    );
    let ledger = node.db.verify_ledger(&node.id).is_ok();
    add(
        "LEDGER",
        if ledger { "PASS" } else { "FAIL" },
        json!({"verified":ledger}),
        "Preserve failed chain; do not delete entries to bypass verification.",
    );
    add(
        "CONTROL API",
        "PASS",
        json!({"bind":node.config.read().unwrap().api,"authentication":"bearer","origin":"exact same origin"}),
        "Use literal loopback address and local operator token.",
    );
    let paths = network::diagnostics(node);
    add(
        "NETWORK LISTEN",
        if paths["addresses"].as_array().is_some_and(|a| !a.is_empty()) {
            "PASS"
        } else {
            "FAIL"
        },
        paths["addresses"].clone(),
        "Allow porch-node TCP/UDP 7332 on the private network only.",
    );
    add(
        "LAN DISCOVERY",
        if node.config.read().unwrap().mdns {
            "UNVERIFIED"
        } else {
            "SKIPPED_ENVIRONMENT"
        },
        json!({"enabled":node.config.read().unwrap().mdns}),
        "Confirm mDNS on two machines; use recipient-bound invitation with pinned IP as fallback.",
    );
    add(
        "PEER CONNECTIVITY",
        if paths["peers"]
            .as_object()
            .is_some_and(|p| p.values().any(|v| v["connected"] == true))
        {
            "PASS"
        } else {
            "UNVERIFIED"
        },
        json!({"connected_peers":paths["peers"].as_object().map(|p|p.values().filter(|v|v["connected"]==true).count())}),
        "Pair intended peer, verify fingerprint, then check its direct path.",
    );
    add(
        "NAT STATUS",
        "UNVERIFIED",
        json!({"relay_enabled":false,"public_nat_probe":false}),
        "0.1.1 qualifies LAN; WAN traversal remains an external gate.",
    );
    let reachable = scan
        .as_ref()
        .is_ok_and(|v| v["currently_reachable"] == true);
    add(
        "OLLAMA",
        if reachable {
            "PASS"
        } else {
            "SKIPPED_ENVIRONMENT"
        },
        json!({"reachable":reachable}),
        "Start local Ollama; use model scan, then verify a model. No mock fallback.",
    );
    add(
        "MODEL INVENTORY",
        if reachable { "PASS" } else { "UNVERIFIED" },
        scan.unwrap_or(Value::Null),
        "Inventory is provider-reported; verify actual invocation separately.",
    );
    let probe = node.root.join(format!("doctor-{}", nonce()));
    let writable = private_write(&probe, b"probe").is_ok() && std::fs::remove_file(&probe).is_ok();
    add(
        "STORAGE",
        if writable { "PASS" } else { "FAIL" },
        json!({"writable":writable}),
        "Check free disk, user ownership and operator storage cap.",
    );
    add(
        "AUTHORITY",
        if node.db.grants().is_ok() {
            "PASS"
        } else {
            "FAIL"
        },
        json!({"explicit_grants_required":true}),
        "Grant exact resource to exact peer with short expiry and bounded calls.",
    );
    let clock = node.db.time().is_ok();
    add(
        "CLOCK",
        if clock { "PASS" } else { "FAIL" },
        json!({"backward_skew_seconds":2,"anomaly":node.db.get("clock_anomaly").ok().flatten()}),
        "Correct system clock; do not extend grant expiry or erase the clock bound.",
    );
    add(
        "PACKAGING",
        "UNVERIFIED",
        build(),
        "Verify portable package manifest; unsigned binaries do not imply publisher trust.",
    );
    json!({"checks":checks,"ledger_chain":if ledger{"VERIFIED"}else{"FAILED"},"physical_qualification":"UNVERIFIED","independent_review":"UNVERIFIED"})
}

/// A package may carry explicit gate evidence tied to this build's commit.
/// Missing evidence remains UNVERIFIED; qualification labels never promote it.
pub fn gates(node: &Node) -> Value {
    let mut categories = serde_json::Map::new();
    for (category, items) in [
        (
            "SOFTWARE",
            vec![
                ("linux_native", "Linux native"),
                ("windows_native", "Windows native"),
                ("macos_native", "macOS native"),
            ],
        ),
        (
            "NETWORK",
            vec![
                ("hosted_loopback", "Hosted loopback"),
                ("physical_lan", "Physical LAN"),
                ("wan_off_lan", "WAN-off LAN"),
                ("wan_traversal", "WAN traversal"),
            ],
        ),
        (
            "MODEL",
            vec![
                ("mock_provider", "Mock provider"),
                ("hosted_live_ollama", "Hosted live Ollama"),
                ("physical_remote_ollama", "Physical remote Ollama"),
            ],
        ),
        (
            "SECURITY",
            vec![
                ("internal_regression", "Internal regression"),
                ("dependency_audit", "Dependency audit"),
                ("independent_review", "Independent review"),
            ],
        ),
        (
            "PACKAGING",
            vec![
                ("linux_package", "Linux"),
                ("windows_package", "Windows"),
                ("macos_package", "macOS"),
            ],
        ),
    ] {
        categories.insert(category.into(),json!(items.into_iter().map(|(id,label)|json!({"id":id,"label":label,"result":"UNVERIFIED","evidence_class":null,"source":null})).collect::<Vec<_>>()));
    }

    if let Ok(Some(report)) = node.db.get("qualification_gates")
        && report["commit"] == build()["commit"]
        && let Some(entries) = report["gates"].as_array()
    {
        for items in categories.values_mut() {
            for item in items.as_array_mut().unwrap() {
                if let Some(entry) = entries.iter().find(|e| e["id"] == item["id"]) {
                    item["result"] = entry["result"].clone();
                    item["evidence_class"] = entry["evidence_class"].clone();
                    item["source"] = entry["source"].clone();
                }
            }
        }
    }
    json!({"categories":categories,"production_qualified":false,"scope":"Separate evidence gates; absence, skips and partial results never count as PASS"})
}

pub fn import_gates(node: &Node, report: &Value) -> Result<Value> {
    ensure!(
        serde_json::to_vec(report)?.len() <= 64 * 1024
            && report["schema_version"] == 1
            && report["commit"] == build()["commit"]
            && report["production_qualified"] == false,
        "GATE_REPORT_BUILD_MISMATCH"
    );
    let entries = report["gates"]
        .as_array()
        .context("GATE_ENTRIES_REQUIRED")?;
    ensure!(entries.len() <= 16, "GATE_ENTRY_LIMIT");
    let allowed = [
        "linux_native",
        "windows_native",
        "macos_native",
        "hosted_loopback",
        "mock_provider",
        "hosted_live_ollama",
        "internal_regression",
        "dependency_audit",
        "linux_package",
        "windows_package",
        "macos_package",
    ];
    let mut ids = std::collections::HashSet::new();
    for e in entries {
        let id = e["id"].as_str().context("GATE_ID_REQUIRED")?;
        ensure!(
            allowed.contains(&id)
                && ids.insert(id)
                && matches!(
                    e["result"].as_str(),
                    Some("PASS" | "FAIL" | "PARTIAL" | "UNVERIFIED" | "SKIPPED_ENVIRONMENT")
                )
                && e["evidence_class"] == "NATIVE_HOSTED"
                && e["source"]
                    .as_str()
                    .is_some_and(|s| crate::valid_label(s, 512)),
            "INVALID_GATE_EVIDENCE"
        );
    }
    node.db.set("qualification_gates", report)?;
    Ok(gates(node))
}
