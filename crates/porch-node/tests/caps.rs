use anyhow::Result;
use porch_core::*;
use porch_node::{Config, Node, limits::LimitsConfig, qualification, storage};
use serde_json::json;

#[tokio::test]
async fn operator_caps_refuse_before_effect_and_keep_revocations() -> Result<()> {
    let root = tempfile::tempdir()?;
    let cfg = Config {
        limits: LimitsConfig {
            input_bytes: 8,
            message_bytes: 8,
            blob_bytes: 41,
            peer_frame_bytes: 4096,
            grants: 1,
            model_timeout_ms: 100,
            services: 4,
            ..Default::default()
        },
        ..Default::default()
    };
    let n = Node::open(root.path(), cfg)?;
    let key = libp2p::identity::Keypair::generate_ed25519();
    let peer = key.public().to_peer_id().to_string();
    n.trust(&peer, "bounded peer", None, &json!({}))?;
    let grant = Grant {
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
    };
    let g = n.issue_grant(grant.clone())?;
    let mut next = grant;
    next.nonce = nonce();
    assert!(n.issue_grant(next).is_err());
    n.revoke_grant(&g.payload.nonce)?;
    let mut replacement = g.payload.clone();
    replacement.nonce = nonce();
    assert!(n.issue_grant(replacement).is_err());
    assert_eq!(n.db.grants()?[0]["revoked"], true);
    let e=n.run(&json!({"capability":"compute.hash","resource":"sha256","input":"123456789","privacy":"LOCAL_ONLY"})).await.unwrap_err();
    assert_eq!(e.to_string(), "INPUT_TOO_LARGE");
    assert_eq!(n.observations.read().unwrap()["outbound_job_requests"], 0);
    assert!(storage::put_local(&n, b"12").is_err());
    let message = Signed::new(
        &key,
        "porch.request.v1",
        WireRequest {
            id: nonce(),
            nonce: nonce(),
            created_at: now(),
            expires_at: now() + 60,
            operation: "message.send".into(),
            args: json!({"grant":g,"id":"cap-msg","text":"123456789"}),
        },
    )?;
    assert_eq!(
        n.receive(peer.clone(), message).await?.payload.code,
        "MESSAGE_TOO_LARGE"
    );
    let huge = Signed::new(
        &key,
        "porch.request.v1",
        WireRequest {
            id: nonce(),
            nonce: nonce(),
            created_at: now(),
            expires_at: now() + 60,
            operation: "resources".into(),
            args: json!({"untrusted":"x".repeat(4096)}),
        },
    )?;
    assert_eq!(
        n.receive(peer, huge).await?.payload.code,
        "REQUEST_TOO_LARGE"
    );
    let job = Job {
        version: VERSION,
        id: nonce(),
        requester: n.id.clone(),
        capability: "compute.hash".into(),
        resource: "sha256".into(),
        input: "abc".into(),
        input_digest: digest(b"abc"),
        max_output_tokens: 16,
        timeout_ms: 101,
        privacy: Privacy::LocalOnly,
        execution_mode: "SINGLE_NODE".into(),
        preferred_peer: None,
        grant: None,
    };
    assert_eq!(
        n.execute_job(job, true).await?.payload.reason.as_deref(),
        Some("JOB_EXCEEDS_OPERATOR_CAP")
    );
    Ok(())
}
#[test]
fn migration_collision_and_oversized_evidence_preserve_source_state() -> Result<()> {
    let root = tempfile::tempdir()?;
    let n = Node::open(root.path(), Config::default())?;
    let id = n.id.clone();
    n.create_porch("Migration collision")?;
    n.db.transaction(|tx| {
        tx.execute_batch("PRAGMA user_version=1")?;
        Ok(())
    })?;
    drop(n);
    assert!(
        porch_node::db::Db::open(&root.path().join("porch.sqlite"), "wrong owner")
            .err()
            .unwrap()
            .to_string()
            .contains("DATABASE_IDENTITY_MISMATCH")
    );
    assert!(!root.path().join("porch.v1-backup.sqlite").exists());
    std::fs::write(
        root.path().join("porch.v1-backup.sqlite"),
        b"existing operator backup",
    )?;
    assert!(
        Node::open(root.path(), Config::default())
            .err()
            .unwrap()
            .to_string()
            .contains("MIGRATION_BACKUP_ALREADY_EXISTS")
    );
    assert_eq!(identity(root.path())?.public().to_peer_id().to_string(), id);
    let db = rusqlite::Connection::open(root.path().join("porch.sqlite"))?;
    assert_eq!(
        db.query_row("PRAGMA user_version", [], |r| r.get::<_, u32>(0))?,
        1
    );
    assert_eq!(
        std::fs::read(root.path().join("porch.v1-backup.sqlite"))?,
        b"existing operator backup"
    );
    let out = root.path().join("export");
    std::fs::create_dir(&out)?;
    let manifest = std::fs::File::create(out.join("manifest.json"))?;
    manifest.set_len(65537)?;
    assert_eq!(
        qualification::read_export(&out).unwrap_err().to_string(),
        "EVIDENCE_MANIFEST_TOO_LARGE"
    );
    Ok(())
}
#[tokio::test]
async fn stale_advertisements_cannot_route_or_authorize_a_prompt() -> Result<()> {
    let ar = tempfile::tempdir()?;
    let br = tempfile::tempdir()?;
    let a = Node::open(
        ar.path(),
        Config {
            models: vec![porch_node::models::ModelSpec {
                name: "stale-model".into(),
                provider: "mock".into(),
                exposed: true,
                version: None,
            }],
            ..Default::default()
        },
    )?;
    let b = Node::open(br.path(), Config::default())?;
    a.trust(&b.id, "B", None, &json!({}))?;
    b.trust(&a.id, "A", None, &json!({}))?;
    let mut ad = a.advertisement()?.payload;
    ad.created_at = now() - 61;
    ad.expires_at = now() - 1;
    let stale = Signed::new(&a.key, "porch.advertisement.v1", ad)?;
    b.db.transaction(|tx| {
        tx.execute(
            "INSERT INTO advertisements(peer,record,expires) VALUES(?1,?2,?3)",
            rusqlite::params![
                a.id,
                serde_json::to_string(&stale)?,
                stale.payload.expires_at
            ],
        )?;
        Ok(())
    })?;
    let grant = a.issue_grant(Grant {
        issuer: a.id.clone(),
        recipient: b.id.clone(),
        capability: "model.inference".into(),
        resource: "stale-model".into(),
        action: "run".into(),
        porch: None,
        limits: Limits::default(),
        created_at: now(),
        expires_at: now() + 600,
        nonce: nonce(),
        exact_input_digest: None,
    })?;
    b.import_grant(grant)?;
    let result=b.run(&json!({"resource":"stale-model","input":"private stale sentinel","privacy":"TRUSTED_PEERS","preferred_peer":a.id})).await?;
    assert_eq!(result["status"], "REFUSED");
    assert_eq!(result["route"]["candidates"][0]["fresh"], false);
    assert_eq!(b.observations.read().unwrap()["outbound_job_requests"], 0);
    assert!(!serde_json::to_string(&b.db.records("ledger")?)?.contains("private stale sentinel"));
    let q = qualification::run(
        &b,
        &json!({"environment":"LOOPBACK","peer":a.id,"model":"stale-model","phase":"revoked"}),
    )
    .await?;
    assert_eq!(
        q["payload"]["scenarios"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["id"] == "remote_inference")
            .unwrap()["result"],
        "FAIL",
        "unrelated route refusal cannot prove grant revocation"
    );
    Ok(())
}
