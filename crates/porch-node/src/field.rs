//! Independently signed participant observations. Public qualification jobs only.
//! An identity signature establishes provenance; it cannot prove physical ownership.
use crate::{Node, qualification};
use anyhow::{Context, Result, ensure};
use porch_core::*;
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

pub fn snapshot(node: &Node, args: &Value) -> Result<Value> {
    let session = args["session"].as_str().context("FIELD_SESSION_REQUIRED")?;
    ensure!(crate::valid_label(session, 64), "INVALID_FIELD_SESSION");
    let class = args["environment"].as_str().unwrap_or("LOOPBACK");
    ensure!(qualification::valid_class(class), "INVALID_EVIDENCE_CLASS");
    let physical = qualification::is_physical(class);
    ensure!(
        !physical || args["separate_machines_confirmed"] == true,
        "PHYSICAL_ATTESTATION_REQUIRED"
    );
    node.db.verify_ledger(&node.id)?;
    let mut jobs = Vec::new();
    node.db.transaction(|tx| {
        let mut stmt = tx.prepare("SELECT job,route,result FROM outbound_jobs ORDER BY rowid DESC LIMIT 200")?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,Option<String>>(2)?)))?;
        for row in rows {
            let (raw, route, result) = row?;
            let job: Job = serde_json::from_str(&raw)?;
            if job.input != qualification::PUBLIC_PROMPT || !job.id.starts_with(&format!("field-{session}-")) { continue; }
            let result: Value = result.map(|v| serde_json::from_str(&v)).transpose()?.unwrap_or(Value::Null);
            jobs.push(json!({"role":"ORIGIN","job_id":job.id,"requester":job.requester,"executor":job.preferred_peer,"request_digest":digest(&canonical(&job)?),"input_digest":job.input_digest,"job_manifest":job,"grant":job.grant,"route":serde_json::from_str::<Value>(&route)?,"receipt":result["receipt"]}));
        }
        Ok(())
    })?;
    node.db.transaction(|tx| {
        let mut stmt = tx.prepare("SELECT hash,receipt FROM jobs WHERE requester<>?1 AND receipt IS NOT NULL ORDER BY rowid DESC LIMIT 200")?;
        let rows = stmt.query_map([&node.id], |r| Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?)))?;
        for row in rows {
            let (hash, raw) = row?;
            let receipt: Signed<Receipt> = serde_json::from_str(&raw)?;
            if receipt.payload.input_digest != digest(qualification::PUBLIC_PROMPT.as_bytes()) || !receipt.payload.job_id.starts_with(&format!("field-{session}-")) { continue; }
            jobs.push(json!({"role":"EXECUTOR","job_id":receipt.payload.job_id,"requester":receipt.payload.requester,"executor":receipt.payload.executor,"request_digest":hash,"input_digest":receipt.payload.input_digest,"receipt":receipt}));
        }
        Ok(())
    })?;
    ensure!(jobs.len() <= 64, "FIELD_JOB_EXPORT_LIMIT");
    let runs = node
        .db
        .records("qualification")?
        .into_iter()
        .filter(|r| r["payload"]["session"] == session)
        .take(16)
        .collect::<Vec<_>>();
    let storage = crate::storage::list(node)?;
    // No decryption keys, plaintext, arbitrary log lines, or unselected prompts.
    let payload = json!({"schema_version":1,"session":session,"peer_id":node.id,"boot_id":node.boot_id,"alias":node.config.read().unwrap().alias,"fingerprint_sha256":qualification::local_fingerprint(node),"porch":node.db.get("porch")?.map(|p|p["id"].clone()),"timestamp":now(),"environment":class,"operator_attested_separate_machines":physical && args["separate_machines_confirmed"]==true,"build":qualification::build(),"network":crate::network::diagnostics(node),"trust":node.db.trust_rows()?.iter().map(|t|json!({"peer":t["peer"],"active":t["active"],"porch":t["porch"]})).collect::<Vec<_>>(),"grants":node.db.grants()?.iter().map(|g|json!({"nonce":g["grant"]["payload"]["nonce"],"issuer":g["grant"]["payload"]["issuer"],"recipient":g["grant"]["payload"]["recipient"],"revoked":g["revoked"],"consumed_calls":g["consumed_calls"]})).collect::<Vec<_>>(),"jobs":jobs,"runs":runs,"storage":storage["blobs"].as_array().map(|b|b.iter().map(|v|json!({"cid":v["cid"],"owner":v["owner"],"size":v["ciphertext_bytes"],"tombstone":v["tombstone"]})).collect::<Vec<_>>()),"ledger_verified":true,"ledger_tip":node.db.transaction(|tx|Ok(tx.query_row("SELECT sequence,hash FROM ledger ORDER BY sequence DESC LIMIT 1",[],|r|Ok(json!({"sequence":r.get::<_,u64>(0)?,"hash":r.get::<_,String>(1)?})))?))?,"production_qualified":false,"privacy_policy":"fixed public probe; no keys, tokens, private inputs or general database export"});
    let signed = serde_json::to_value(Signed::new(
        &node.key,
        "porch.field.participant.v1",
        payload,
    )?)?;
    validate_snapshot(&signed)?;
    Ok(signed)
}

pub fn validate_snapshot(value: &Value) -> Result<Signed<Value>> {
    ensure!(
        serde_json::to_vec(value)?.len() <= 16 * 1024 * 1024,
        "FIELD_EVIDENCE_TOO_LARGE"
    );
    let signed: Signed<Value> = serde_json::from_value(value.clone())?;
    signed.verify("porch.field.participant.v1")?;
    let p = &signed.payload;
    let session = p["session"].as_str().context("FIELD_SESSION_REQUIRED")?;
    ensure!(
        crate::valid_label(session, 64)
            && p["schema_version"] == 1
            && p["peer_id"] == signed.signer
            && p["production_qualified"] == false
            && p["timestamp"].as_u64().is_some()
            && p["ledger_verified"] == true,
        "FIELD_SCHEMA_OR_IDENTITY_MISMATCH"
    );
    let class = p["environment"].as_str().context("FIELD_CLASS_REQUIRED")?;
    ensure!(qualification::valid_class(class), "INVALID_EVIDENCE_CLASS");
    ensure!(
        !qualification::is_physical(class) || p["operator_attested_separate_machines"] == true,
        "PHYSICAL_ATTESTATION_REQUIRED"
    );
    let jobs = p["jobs"].as_array().context("FIELD_JOBS_REQUIRED")?;
    ensure!(jobs.len() <= 64, "FIELD_JOB_EXPORT_LIMIT");
    let mut seen = std::collections::HashSet::new();
    for j in jobs {
        let id = j["job_id"].as_str().context("FIELD_JOB_ID_REQUIRED")?;
        ensure!(
            id.len() <= 128
                && id.starts_with(&format!("field-{session}-"))
                && seen.insert((j["requester"].to_string(), id)),
            "FIELD_JOB_SESSION_OR_DUPLICATE"
        );
        ensure!(
            j["input_digest"] == digest(qualification::PUBLIC_PROMPT.as_bytes()),
            "FIELD_PRIVATE_INPUT_REFUSED"
        );
        let origin = j["role"] == "ORIGIN";
        ensure!(
            (origin && j["requester"] == signed.signer)
                || (j["role"] == "EXECUTOR" && j["executor"] == signed.signer),
            "FIELD_ROLE_MISMATCH"
        );
        if origin {
            let job: Job = serde_json::from_value(j["job_manifest"].clone())?;
            job.validate(&signed.signer)?;
            ensure!(
                job.input == qualification::PUBLIC_PROMPT
                    && job.id == id
                    && j["request_digest"] == digest(&canonical(&job)?)
                    && j["grant"] == json!(job.grant)
                    && j["executor"] == json!(job.preferred_peer),
                "FIELD_REQUEST_MANIFEST_MISMATCH"
            );
            let g: Signed<Grant> = serde_json::from_value(j["grant"].clone())?;
            g.verify("porch.grant.v1")?;
            ensure!(
                g.signer == g.payload.issuer
                    && j["executor"] == g.signer
                    && j["requester"] == g.payload.recipient
                    && j["route"]["chosen"] == j["executor"],
                "FIELD_GRANT_OR_ROUTE_MISMATCH"
            );
        }
        if !j["receipt"].is_null() {
            let r: Signed<Receipt> = serde_json::from_value(j["receipt"].clone())?;
            r.verify("porch.job.receipt.v1")?;
            let r = &r.payload;
            ensure!(
                j["executor"] == j["receipt"]["signer"]
                    && j["executor"] == r.executor
                    && j["requester"] == r.requester
                    && id == r.job_id
                    && j["input_digest"] == r.input_digest
                    && j["request_digest"] == r.request_digest
                    && r.output_digest == digest(&canonical(&r.output)?)
                    && r.finished_at >= r.started_at
                    && r.finished_at <= p["timestamp"].as_u64().unwrap() + 2
                    && matches!(
                        r.status.as_str(),
                        "COMPLETED" | "REFUSED" | "UNCERTAIN" | "FAILED"
                    ),
                "FIELD_RECEIPT_BINDING_MISMATCH"
            );
            if origin {
                ensure!(
                    j["grant"]["payload"]["nonce"] == json!(r.grant_nonce),
                    "FIELD_GRANT_NONCE_MISMATCH"
                );
            }
        } else {
            ensure!(origin, "EXECUTOR_RECEIPT_REQUIRED");
        }
    }
    for run in p["runs"].as_array().context("FIELD_RUNS_REQUIRED")? {
        let r: Signed<Value> = serde_json::from_value(run.clone())?;
        r.verify("porch.qualification.v1")?;
        ensure!(
            r.signer == signed.signer
                && r.payload["session"] == session
                && r.payload["production_qualified"] == false,
            "FIELD_RUN_BINDING_MISMATCH"
        );
    }
    Ok(signed)
}

pub fn read_snapshot(path: &Path) -> Result<Value> {
    ensure!(
        std::fs::metadata(path)?.len() <= 16 * 1024 * 1024,
        "FIELD_EVIDENCE_TOO_LARGE"
    );
    let v = serde_json::from_slice(&std::fs::read(path)?)?;
    validate_snapshot(&v)?;
    Ok(v)
}
fn path_ok(p: &Value, remote: &str) -> bool {
    let n = &p["network"]["peers"][remote];
    n["connected"] == true
        && n["authenticated"] == true
        && n["encrypted"] == true
        && n["path"] == "LAN_DIRECT"
}
/// This is bilateral correlation, not independent hardware certification.
pub fn correlate(a: &Value, b: &Value) -> Result<Value> {
    let a = validate_snapshot(a)?;
    let b = validate_snapshot(b)?;
    let mut problems = Vec::<String>::new();
    let ap = &a.payload;
    let bp = &b.payload;
    if a.signer == b.signer {
        problems.push("SAME_IDENTITY".into());
    }
    if ap["session"] != bp["session"] {
        problems.push("SESSION_MISMATCH".into());
    }
    if ap["porch"].is_null() || ap["porch"] != bp["porch"] {
        problems.push("PORCH_MISMATCH".into());
    }
    if ap["timestamp"]
        .as_u64()
        .unwrap()
        .abs_diff(bp["timestamp"].as_u64().unwrap())
        > 86400
    {
        problems.push("TIMESTAMP_WINDOW_MISMATCH".into());
    }
    let mut grouped = BTreeMap::<String, Vec<(&str, &Value)>>::new();
    for (s, p) in [(&a.signer, ap), (&b.signer, bp)] {
        for j in p["jobs"].as_array().unwrap() {
            grouped
                .entry(format!(
                    "{}/{}",
                    j["requester"].as_str().unwrap_or(""),
                    j["job_id"].as_str().unwrap_or("")
                ))
                .or_default()
                .push((s, j));
        }
    }
    let mut matches = Vec::new();
    let mut live = false;
    let mut revoked = false;
    for (id, rows) in grouped {
        let origin = rows.iter().find(|(_, j)| j["role"] == "ORIGIN");
        let executor = rows.iter().find(|(_, j)| j["role"] == "EXECUTOR");
        let mut ok = rows.len() == 2;
        if let (Some((os, o)), Some((es, e))) = (origin, executor) {
            ok = ok
                && os != es
                && o["requester"] == *os
                && o["executor"] == *es
                && o["request_digest"] == e["request_digest"]
                && o["input_digest"] == e["input_digest"]
                && !o["receipt"].is_null()
                && o["receipt"] == e["receipt"];
            if ok {
                let r = &o["receipt"]["payload"];
                live |= r["status"] == "COMPLETED" && r["provider"] == "ollama";
                revoked |= r["status"] == "REFUSED"
                    && r["reason"].as_str().is_some_and(|s| s.contains("REVOKED"));
                matches.push(json!({"job":id,"result":"MATCH","request_digest":o["request_digest"],"input_digest":o["input_digest"],"output_digest":r["output_digest"],"grant_nonce":r["grant_nonce"],"started_at":r["started_at"],"finished_at":r["finished_at"],"status":r["status"],"route":o["route"]}));
            }
        }
        if !ok {
            problems.push(format!("EVIDENCE_MISMATCH:{id}"));
            matches.push(json!({"job":id,"result":"EVIDENCE_MISMATCH"}));
        }
    }
    let physical = ap["environment"] == "PHYSICAL_LAN"
        && bp["environment"] == "PHYSICAL_LAN"
        && path_ok(ap, &b.signer)
        && path_ok(bp, &a.signer);
    let mut failed = false;
    let mut checks = BTreeMap::new();
    for name in [
        "identity",
        "ledger",
        "peer_path",
        "trusted_peer",
        "messaging",
        "signed_discovery",
        "bounded_authorization",
        "encrypted_storage",
        "storage_integrity",
        "restart_persistence",
    ] {
        let observed = [ap, bp]
            .iter()
            .flat_map(|p| p["runs"].as_array().unwrap())
            .flat_map(|r| r["payload"]["scenarios"].as_array().into_iter().flatten())
            .filter(|s| s["id"] == name)
            .collect::<Vec<_>>();
        let fail = observed.iter().any(|s| s["result"] == "FAIL");
        failed |= fail;
        checks.insert(
            name,
            if fail {
                "FAIL"
            } else if observed.iter().any(|s| s["result"] == "PASS") {
                "PASS"
            } else {
                "UNVERIFIED"
            },
        );
    }
    let mut storage_correlations = Vec::new();
    for (owner, remote) in [(ap, bp), (bp, ap)] {
        for run in owner["runs"].as_array().unwrap() {
            for scenario in run["payload"]["scenarios"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|s| s["id"] == "encrypted_storage" && s["result"] == "PASS")
            {
                let o = &scenario["observation"];
                let cid = &o["cid"];
                let stored = remote["storage"].as_array().is_some_and(|rows| {
                    rows.iter().any(|v| {
                        v["cid"] == *cid
                            && v["owner"] == owner["peer_id"]
                            && v["tombstone"] == false
                    })
                });
                if !stored {
                    problems.push("EVIDENCE_MISMATCH:STORAGE_PROVIDER_RECORD_MISSING".into());
                }
                storage_correlations.push(json!({"cid":cid,"owner":owner["peer_id"],"provider":remote["peer_id"],"result":if stored{"MATCH"}else{"EVIDENCE_MISMATCH"},"retrieved_digest":o["retrieved_digest"]}));
            }
        }
    }
    let all = checks.values().all(|s| *s == "PASS") && !storage_correlations.is_empty();
    let state = if !problems.is_empty() || failed {
        "FAILED"
    } else if matches.is_empty()
        && ap["runs"].as_array().unwrap().is_empty()
        && bp["runs"].as_array().unwrap().is_empty()
    {
        "NOT_RUN"
    } else if physical && all && revoked && live {
        "PHYSICAL_LAN_AND_LIVE_MODEL_QUALIFIED"
    } else if physical && all && revoked {
        "PHYSICAL_LAN_QUALIFIED"
    } else {
        "PARTIAL"
    };
    Ok(
        json!({"schema_version":1,"state":state,"session":ap["session"],"porch":ap["porch"],"peers":[a.signer,b.signer],"participant_hashes":[a.hash()?,b.hash()?],"classes":[ap["environment"],bp["environment"]],"job_correlations":matches,"storage_correlations":storage_correlations,"problems":problems,"checks":checks,"bilateral_lan_path":physical,"live_ollama_match":live,"revoked_refusal_match":revoked,"wan_off":"UNVERIFIED: examine offline reachability observations and operator attestations on both PCs","independent_security_review":"UNVERIFIED","production_qualified":false,"physical_scope":"operator-attested two-machine run plus signed bilateral observations; not independent hardware certification"}),
    )
}
pub fn write_report(out: &Path, report: &Value) -> Result<()> {
    ensure!(!out.exists(), "REPORT_DIRECTORY_ALREADY_EXISTS");
    std::fs::create_dir_all(out)?;
    private_write(
        &out.join("physical-qualification-report.json"),
        &serde_json::to_vec_pretty(report)?,
    )?;
    let text = format!(
        "# Physical qualification report\n\nState: **{}**\n\nSession: {}\n\nProduction qualified: **false**. Independent security review: **UNVERIFIED**.\n\n{}\n\n```json\n{}\n```\n",
        report["state"].as_str().unwrap_or("FAILED"),
        report["session"],
        report["physical_scope"],
        serde_json::to_string_pretty(report)?
    );
    private_write(
        &out.join("PHYSICAL_QUALIFICATION_REPORT.md"),
        text.as_bytes(),
    )?;
    Ok(())
}

/// Compare an independently signed pre-stop checkpoint with this fresh process.
pub async fn restart(node: &std::sync::Arc<Node>, args: &Value) -> Result<Value> {
    let before = validate_snapshot(&args["before"])?;
    ensure!(
        before.signer == node.id,
        "RESTART_CHECKPOINT_IDENTITY_MISMATCH"
    );
    let p = &before.payload;
    ensure!(p["boot_id"] != node.boot_id, "RESTART_REQUIRES_NEW_PROCESS");
    let current = snapshot(
        node,
        &json!({"session":p["session"],"environment":p["environment"],"separate_machines_confirmed":p["operator_attested_separate_machines"]}),
    )?;
    let c = &current["payload"];
    let prefix = node.db.transaction(|tx| {
        Ok(tx.query_row(
            "SELECT hash FROM ledger WHERE sequence=?1",
            [p["ledger_tip"]["sequence"]
                .as_u64()
                .context("CHECKPOINT_LEDGER_TIP_REQUIRED")?],
            |r| r.get::<_, String>(0),
        )?)
    })?;
    let same = p["porch"] == c["porch"]
        && p["trust"] == c["trust"]
        && p["grants"] == c["grants"]
        && p["ledger_tip"]["hash"] == prefix
        && p["jobs"] == c["jobs"];
    let scan = crate::models::scan(node).await?;
    let peer_ids = vec![node.id.clone()];
    let scenario = json!({"id":"restart_persistence","expectation":"New process preserves identity, membership, trust, revocation, ledger prefix and cached job receipts without duplicate grant accounting","observation":{"identity_unchanged":true,"new_process":true,"authority_jobs_and_ledger_unchanged":same,"provider_scan_after_restart":scan["currently_reachable"]},"result":if same{"PASS"}else{"FAIL"},"evidence_file":"run.json","timestamp":now(),"peers":peer_ids});
    let run_id = nonce();
    let signed = Signed::new(
        &node.key,
        "porch.qualification.v1",
        json!({"schema_version":1,"run_id":run_id,"timestamp":now(),"build":qualification::build(),"environment":p["environment"],"phase":"restart","session":p["session"],"operator_attested_separate_machines":p["operator_attested_separate_machines"],"peers":peer_ids,"scenarios":[scenario],"signed_receipts":[],"route":null,"network":crate::network::diagnostics(node),"production_qualified":false}),
    )?;
    node.db.transaction(|tx|{tx.execute("INSERT INTO qualification(id,record) VALUES(?1,?2)",rusqlite::params![run_id,serde_json::to_string(&signed)?])?;tx.execute("DELETE FROM qualification WHERE id NOT IN (SELECT id FROM qualification ORDER BY rowid DESC LIMIT 64)",[])?;Ok(())})?;
    Ok(serde_json::to_value(signed)?)
}
