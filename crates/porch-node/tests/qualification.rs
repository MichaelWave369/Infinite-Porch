use anyhow::Result;
use porch_core::*;
use porch_node::{Config, Node, api, limits::LimitsConfig, qualification, storage};
use serde_json::json;

#[tokio::test]
async fn evidence_is_signed_sanitized_and_schema_validated() -> Result<()> {
    let root = tempfile::tempdir()?;
    let node = Node::open(root.path(), Config::default())?;
    node.run(&json!({"capability":"compute.hash","resource":"sha256","input":"PRIVATE-UNRELATED-JOB-CONTENT","privacy":"LOCAL_ONLY"})).await?;
    let run = qualification::run(&node, &json!({"environment":"SIMULATED"})).await?;
    assert_eq!(run["payload"]["production_qualified"], false);
    let bundle = qualification::export(&node)?;
    let bytes = serde_json::to_string(&bundle)?;
    assert!(!bytes.contains("PRIVATE-UNRELATED-JOB-CONTENT"));
    assert!(!bytes.contains(&node.api_token));
    assert!(!bytes.contains(&hex::encode(node.key.to_protobuf_encoding()?)));
    let report = qualification::validate_export(&bundle)?;
    assert_eq!(report["skip_counts_as_pass"], false);
    assert!(report["scenario_counts"]["UNVERIFIED"].as_u64().unwrap() > 0);
    let out = root.path().join("export");
    qualification::write_export(&out, &bundle)?;
    qualification::validate_export(&qualification::read_export(&out)?)?;
    let mut tampered = bundle.clone();
    tampered["files"]["results.json"][0]["result"] = json!("PASS");
    // The original identity scenario was already PASS; change an unverified gate.
    tampered["files"]["results.json"][4]["result"] = json!("PASS");
    assert!(qualification::validate_export(&tampered).is_err());
    let mut signed: Signed<serde_json::Value> = serde_json::from_value(run)?;
    signed.payload["scenarios"][0]["result"] = json!("SKIPPED_IS_PASS");
    signed = Signed::new(&node.key, "porch.qualification.v1", signed.payload)?;
    node.db.transaction(|tx| {
        tx.execute(
            "UPDATE qualification SET record=?1",
            [serde_json::to_string(&signed)?],
        )?;
        Ok(())
    })?;
    assert!(qualification::validate_export(&qualification::export(&node)?).is_err());
    assert!(
        qualification::run(&node, &json!({"environment":"PHYSICAL"}))
            .await
            .is_err()
    );
    Ok(())
}
#[test]
fn schema_one_migration_backs_up_and_preserves_authority_and_identity() -> Result<()> {
    let root = tempfile::tempdir()?;
    let cfg = Config::default();
    let n = Node::open(root.path(), cfg.clone())?;
    let peer = libp2p::identity::Keypair::generate_ed25519()
        .public()
        .to_peer_id()
        .to_string();
    n.trust(&peer, "B", None, &json!({}))?;
    let porch = n.create_porch("Migration Porch")?;
    let grant = n.issue_grant(Grant {
        issuer: n.id.clone(),
        recipient: peer.clone(),
        capability: "message.direct".into(),
        resource: "inbox".into(),
        action: "send".into(),
        porch: None,
        limits: Limits::default(),
        created_at: now(),
        expires_at: now() + 600,
        nonce: nonce(),
        exact_input_digest: None,
    })?;
    n.revoke_grant(&grant.payload.nonce)?;
    let blob = storage::put_local(&n, b"migration private vault")?;
    let mut old_config = serde_json::to_value(&cfg)?;
    old_config.as_object_mut().unwrap().remove("limits");
    n.db.set("config", &old_config)?;
    n.db.transaction(|tx| {
        tx.execute_batch("DROP TABLE qualification; PRAGMA user_version=1;")?;
        Ok(())
    })?;
    let id = n.id.clone();
    let key = std::fs::read(root.path().join("identity.key"))?;
    drop(n);
    let n = Node::open(root.path(), Config::load(root.path())?)?;
    assert_eq!(n.id, id);
    assert_eq!(std::fs::read(root.path().join("identity.key"))?, key);
    assert!(n.db.trusted(&peer)?);
    assert_eq!(n.db.get("porch")?.unwrap()["id"], porch["id"]);
    assert_eq!(n.db.grants()?[0]["revoked"], true);
    n.db.verify_ledger(&id)?;
    assert_eq!(
        storage::get_local(&n, blob["cid"].as_str().unwrap())?["plaintext_hex"],
        hex::encode(b"migration private vault")
    );
    let backup = root.path().join("porch.v1-backup.sqlite");
    assert!(backup.exists());
    let c = rusqlite::Connection::open(backup)?;
    assert_eq!(
        c.query_row("PRAGMA user_version", [], |r| r.get::<_, u32>(0))?,
        1
    );
    assert!(c.query_row("SELECT revoked FROM grants", [], |r| r.get::<_, bool>(0))?);
    Ok(())
}
#[test]
fn state_lock_missing_identity_and_unsafe_limits_fail_closed() -> Result<()> {
    let root = tempfile::tempdir()?;
    let n = Node::open(root.path(), Config::default())?;
    assert!(
        Node::open(root.path(), Config::default())
            .err()
            .unwrap()
            .to_string()
            .contains("STATE_ALREADY_IN_USE")
    );
    drop(n);
    std::fs::remove_file(root.path().join("identity.key"))?;
    assert!(Node::open(root.path(), Config::default()).is_err());
    assert!(!root.path().join("identity.key").exists());
    let root = tempfile::tempdir()?;
    let bad = Config {
        limits: LimitsConfig {
            concurrent_jobs: 17,
            ..Default::default()
        },
        ..Config::default()
    };
    assert!(Node::open(root.path(), bad).is_err());
    let bad = Config {
        alias: "fake\u{202e}device".into(),
        ..Config::default()
    };
    assert!(Node::open(root.path(), bad).is_err());
    Ok(())
}
#[tokio::test]
async fn fingerprint_mismatch_is_refused_before_connecting() -> Result<()> {
    let ar = tempfile::tempdir()?;
    let br = tempfile::tempdir()?;
    let a = Node::open(ar.path(), Config::default())?;
    let b = Node::open(br.path(), Config::default())?;
    a.create_porch("Pairing")?;
    let invite = a.invite(&json!({"recipient":b.id}))?;
    let e = api::operate(
        &b,
        "qualification.join",
        &json!({"invite":invite,"fingerprint":"00".repeat(32)}),
    )
    .await
    .unwrap_err();
    assert!(e.to_string().contains("FINGERPRINT_MISMATCH"));
    assert!(b.db.trust_rows()?.is_empty());
    assert!(
        b.observations.read().unwrap()["peers"]
            .as_object()
            .unwrap()
            .is_empty()
    );
    Ok(())
}
#[tokio::test]
async fn malformed_private_peer_data_never_enters_shared_refusal_evidence() -> Result<()> {
    let root = tempfile::tempdir()?;
    let n = Node::open(root.path(), Config::default())?;
    let key = libp2p::identity::Keypair::generate_ed25519();
    let peer = key.public().to_peer_id().to_string();
    n.trust(&peer, "B", None, &json!({}))?;
    let request = Signed::new(
        &key,
        "porch.request.v1",
        WireRequest {
            id: nonce(),
            nonce: nonce(),
            created_at: now(),
            expires_at: now() + 60,
            operation: "job.run".into(),
            args: json!({"privacy":"SECRET-PARSER-INJECTION","input":"PRIVATE-PROMPT-INJECTION"}),
        },
    )?;
    let response = n.receive(peer, request).await?;
    assert_eq!(response.payload.status, "REFUSED");
    let ledger = serde_json::to_string(&n.db.records("ledger")?)?;
    assert!(!ledger.contains("SECRET-PARSER-INJECTION"));
    assert!(!ledger.contains("PRIVATE-PROMPT-INJECTION"));
    assert!(!serde_json::to_string(&response)?.contains("SECRET-PARSER-INJECTION"));
    Ok(())
}
#[test]
fn observed_path_classification_never_labels_loopback_as_physical() {
    use porch_node::network::classify_path;
    assert_eq!(classify_path("/ip4/127.0.0.1/tcp/7332", false), "LOOPBACK");
    assert_eq!(
        classify_path("/ip6/::1/udp/7332/quic-v1", false),
        "LOOPBACK"
    );
    assert_eq!(
        classify_path("/ip4/192.168.1.5/tcp/7332", false),
        "LAN_DIRECT"
    );
    assert_eq!(classify_path("/ip4/203.0.113.1/tcp/7332", true), "RELAY");
    assert_eq!(classify_path("invalid", false), "UNKNOWN");
}

#[tokio::test]
async fn invitation_address_fallback_stays_pinned_to_the_confirmed_identity() -> Result<()> {
    let ar = tempfile::tempdir()?;
    let br = tempfile::tempdir()?;
    let cfg = Config {
        listen: vec!["/ip4/127.0.0.1/tcp/0".into()],
        mdns: false,
        ..Default::default()
    };
    let a = Node::open(ar.path(), cfg.clone())?;
    let b = Node::open(br.path(), cfg)?;
    porch_node::network::start(a.clone()).await?;
    porch_node::network::start(b.clone()).await?;
    for _ in 0..100 {
        if !a.observations.read().unwrap()["addresses"]
            .as_array()
            .unwrap()
            .is_empty()
            && !b.observations.read().unwrap()["addresses"]
                .as_array()
                .unwrap()
                .is_empty()
        {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    let mut wrong: libp2p::Multiaddr = b.observations.read().unwrap()["addresses"][0]
        .as_str()
        .unwrap()
        .parse()?;
    wrong.pop();
    let wrong = wrong
        .with(libp2p::multiaddr::Protocol::P2p(a.id.parse()?))
        .to_string();
    a.observations.write().unwrap()["addresses"]
        .as_array_mut()
        .unwrap()
        .insert(0, json!(wrong));
    a.create_porch("Pinned fallback")?;
    let invite = a.invite(&json!({"recipient":b.id}))?;
    tokio::time::timeout(std::time::Duration::from_secs(5), b.join(invite)).await??;
    assert!(b.db.trusted(&a.id)?);
    assert!(a.db.trusted(&b.id)?);
    assert_eq!(b.db.trust_rows()?.len(), 1);
    assert!(b.refresh().await?[0]["updated"].as_bool().unwrap());
    Ok(())
}
