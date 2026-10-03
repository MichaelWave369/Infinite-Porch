use anyhow::Result;
use porch_core::*;
use porch_node::{Config, Node, storage};
use rusqlite::params;
use serde_json::json;

#[test]
fn completion_rename_recovery_and_expired_transfer_cleanup() -> Result<()> {
    let root = tempfile::tempdir()?;
    let n = Node::open(
        root.path(),
        Config {
            storage_quota: MAX_BLOB as u64,
            ..Config::default()
        },
    )?;
    let owner = libp2p::identity::Keypair::generate_ed25519()
        .public()
        .to_peer_id()
        .to_string();
    n.trust(&owner, "owner", None, &json!({}))?;
    let grant = n.issue_grant(Grant {
        issuer: n.id.clone(),
        recipient: owner.clone(),
        capability: "blob.storage".into(),
        resource: "vault".into(),
        action: "store".into(),
        porch: None,
        limits: Limits {
            max_storage_bytes: MAX_BLOB as u64,
            ..Limits::default()
        },
        created_at: now(),
        expires_at: now() + 600,
        nonce: nonce(),
        exact_input_digest: None,
    })?;
    let cipher = seal(b"completion fault fixture", &random_bytes::<32>(), &owner)?;
    let cid = digest(&cipher);
    storage::receive(
        &n,
        &owner,
        "blob.begin",
        &json!({"cid":cid,"grant":grant,"size":cipher.len(),"encryption":"xchacha20poly1305"}),
    )?;
    // Inject precisely the persistent state after final rename and before DB
    // commit. This is a crash-state fixture, not a power-loss qualification.
    let temporary = root
        .path()
        .join("transfers")
        .join(format!("{cid}-{}", digest(owner.as_bytes())));
    std::fs::write(&temporary, &cipher)?;
    std::fs::create_dir_all(root.path().join("vault"))?;
    std::fs::rename(&temporary, root.path().join("vault").join(&cid))?;
    drop(n);
    let n = Node::open(
        root.path(),
        Config {
            storage_quota: MAX_BLOB as u64,
            ..Config::default()
        },
    )?;
    let out = storage::receive(
        &n,
        &owner,
        "blob.begin",
        &json!({"cid":cid,"grant":grant,"size":cipher.len(),"encryption":"xchacha20poly1305"}),
    )?;
    assert_eq!(out["offset"], cipher.len());
    assert_eq!(storage::list(&n)?["blobs"][0]["cid"], cid);
    assert!(
        n.db.records("ledger")?
            .iter()
            .any(|r| r["payload"]["event_type"] == "storage.completion.recovered")
    );
    let cipher2 = seal(b"expired transfer", &random_bytes::<32>(), &owner)?;
    let cid2 = digest(&cipher2);
    storage::receive(
        &n,
        &owner,
        "blob.begin",
        &json!({"cid":cid2,"grant":grant,"size":cipher2.len(),"encryption":"xchacha20poly1305"}),
    )?;
    n.db.transaction(|tx| {
        tx.execute("UPDATE uploads SET expires=0 WHERE cid=?1", [&cid2])?;
        Ok(())
    })?;
    storage::receive(
        &n,
        &owner,
        "blob.begin",
        &json!({"cid":cid2,"grant":grant,"size":cipher2.len(),"encryption":"xchacha20poly1305"}),
    )?;
    assert!(
        n.db.records("ledger")?
            .iter()
            .any(|r| r["payload"]["event_type"] == "storage.transfer.expired")
    );
    Ok(())
}
#[tokio::test]
async fn uncertain_jobs_never_reexecute_after_restart() -> Result<()> {
    let root = tempfile::tempdir()?;
    let n = Node::open(root.path(), Config::default())?;
    let j = Job {
        version: VERSION,
        id: nonce(),
        requester: n.id.clone(),
        capability: "compute.hash".into(),
        resource: "sha256".into(),
        input: "abc".into(),
        input_digest: digest(b"abc"),
        max_output_tokens: 16,
        timeout_ms: 1000,
        privacy: Privacy::LocalOnly,
        execution_mode: "SINGLE_NODE".into(),
        preferred_peer: None,
        grant: None,
    };
    let hash = digest(&canonical(&j)?);
    n.db.transaction(|tx| {
        tx.execute(
            "INSERT INTO jobs(requester,id,hash,status) VALUES(?1,?2,?3,'RUNNING')",
            params![n.id, j.id, hash],
        )?;
        Ok(())
    })?;
    drop(n);
    let n = Node::open(root.path(), Config::default())?;
    assert!(
        n.execute_job(j, true)
            .await
            .unwrap_err()
            .to_string()
            .contains("DUPLICATE_JOB_UNCERTAIN")
    );
    n.db.transaction(|tx| {
        tx.execute_batch("PRAGMA user_version=2")?;
        Ok(())
    })?;
    drop(n);
    assert!(
        Node::open(root.path(), Config::default())
            .err()
            .unwrap()
            .to_string()
            .contains("DATABASE_SCHEMA_UNSUPPORTED")
    );
    Ok(())
}
