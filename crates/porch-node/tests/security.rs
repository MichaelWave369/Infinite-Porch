use anyhow::Result;
use porch_core::*;
use porch_node::{Config, Node, models::ModelSpec, network};
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};

async fn node(root: &std::path::Path, name: &str) -> Result<Arc<Node>> {
    let n = Node::open(
        root,
        Config {
            alias: name.into(),
            listen: vec!["/ip4/127.0.0.1/tcp/0".into()],
            mdns: false,
            models: vec![ModelSpec {
                name: "test-model".into(),
                provider: "mock".into(),
                exposed: true,
                version: None,
            }],
            ..Config::default()
        },
    )?;
    network::start(n.clone()).await?;
    for _ in 0..100 {
        if !n.observations.read().unwrap()["addresses"]
            .as_array()
            .unwrap()
            .is_empty()
        {
            return Ok(n);
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    anyhow::bail!("LISTENER_START_TIMEOUT")
}
async fn connect(a: &Arc<Node>, b: &Arc<Node>) -> Result<()> {
    let address = b.observations.read().unwrap()["addresses"][0]
        .as_str()
        .unwrap()
        .to_string();
    a.network
        .read()
        .unwrap()
        .as_ref()
        .unwrap()
        .connect(&address, Some(&b.id))?;
    tokio::time::sleep(Duration::from_millis(30)).await;
    Ok(())
}
fn request(key: &libp2p::identity::Keypair, operation: &str, args: Value) -> Signed<WireRequest> {
    Signed::new(
        key,
        "porch.request.v1",
        WireRequest {
            id: nonce(),
            nonce: nonce(),
            created_at: now(),
            expires_at: now() + 60,
            operation: operation.into(),
            args,
        },
    )
    .unwrap()
}
#[tokio::test]
async fn signing_material_survives_the_actual_wire_codec() -> Result<()> {
    use libp2p::request_response::Codec;
    let root = tempfile::tempdir()?;
    let n = Node::open(root.path(), Config::default())?;
    let original = Signed::new(
        &n.key,
        "porch.response.v1",
        WireResponse {
            request_nonce: "fixture".into(),
            request_digest: "fixture".into(),
            status: "OK".into(),
            code: "OK".into(),
            output: serde_json::to_value(n.advertisement()?)?,
            timestamp: now(),
        },
    )?;
    original.verify("porch.response.v1")?;
    let mut codec = libp2p::request_response::json::codec::Codec::<
        Signed<WireRequest>,
        Signed<WireResponse>,
    >::default();
    let mut io = futures::io::Cursor::new(Vec::new());
    let protocol = libp2p::StreamProtocol::new("/infinite-porch/rpc/1");
    codec
        .write_response(&protocol, &mut io, original.clone())
        .await?;
    io.set_position(0);
    let decoded = codec.read_response(&protocol, &mut io).await?;
    assert_eq!(
        digest(&canonical(&original)?),
        digest(&canonical(&decoded)?),
        "wire codec changed signing material"
    );
    decoded.verify("porch.response.v1")?;
    Ok(())
}
#[tokio::test]
async fn actual_transport_rejects_forgery_replay_and_wrong_principal() -> Result<()> {
    let aroot = tempfile::tempdir()?;
    let broot = tempfile::tempdir()?;
    let croot = tempfile::tempdir()?;
    let a = node(aroot.path(), "A").await?;
    let b = node(broot.path(), "B").await?;
    let c = node(croot.path(), "C").await?;
    a.trust(&b.id, "B", None, &json!({}))?;
    b.trust(&a.id, "A", None, &json!({}))?;
    connect(&b, &a).await?;
    connect(&c, &a).await?;
    let unknown = c.rpc(&a.id, "resources", json!({})).await?;
    assert!(unknown.payload.code.contains("UNKNOWN"));
    let handle = b.network.read().unwrap().as_ref().unwrap().clone();
    let mut forged = request(&b.key, "resources", json!({}));
    forged.signature = "00".repeat(64);
    let r = handle.request(&a.id, forged).await?;
    assert_eq!(r.payload.status, "REFUSED");
    assert!(r.payload.code.contains("SIGNATURE"));
    let impersonation = request(&c.key, "resources", json!({}));
    let r = handle.request(&a.id, impersonation).await?;
    assert!(r.payload.code.contains("TRANSPORT_IDENTITY"));
    let good = request(&b.key, "resources", json!({}));
    assert_eq!(
        handle.request(&a.id, good.clone()).await?.payload.status,
        "OK"
    );
    assert!(
        handle
            .request(&a.id, good)
            .await?
            .payload
            .code
            .contains("REPLAY")
    );
    let grant = a.issue_grant(Grant {
        issuer: a.id.clone(),
        recipient: b.id.clone(),
        capability: "model.inference".into(),
        resource: "test-model".into(),
        action: "run".into(),
        porch: None,
        limits: Limits {
            max_calls: 1,
            calls_per_hour: 1,
            ..Limits::default()
        },
        created_at: now(),
        expires_at: now() + 600,
        nonce: nonce(),
        exact_input_digest: None,
    })?;
    let mut job = Job {
        version: 1,
        id: nonce(),
        requester: c.id.clone(),
        capability: "model.inference".into(),
        resource: "test-model".into(),
        input: "private".into(),
        input_digest: digest(b"private"),
        max_output_tokens: 16,
        timeout_ms: 1000,
        privacy: Privacy::TrustedPeers,
        execution_mode: "REMOTE_NODE".into(),
        preferred_peer: Some(a.id.clone()),
        grant: Some(grant.clone()),
    };
    let r = handle
        .request(
            &a.id,
            request(&b.key, "job.run", serde_json::to_value(&job)?),
        )
        .await?;
    assert!(r.payload.code.contains("REQUESTER_TRANSPORT"));
    job.requester = b.id.clone();
    let one = b.rpc(&a.id, "job.run", serde_json::to_value(&job)?).await?;
    let receipt: Signed<Receipt> = serde_json::from_value(one.payload.output)?;
    assert_eq!(receipt.payload.status, "COMPLETED");
    receipt.verify("porch.job.receipt.v1")?;
    job.id = nonce();
    let two = b.rpc(&a.id, "job.run", serde_json::to_value(&job)?).await?;
    let receipt: Signed<Receipt> = serde_json::from_value(two.payload.output)?;
    assert!(receipt.payload.reason.unwrap().contains("QUOTA"));
    assert_eq!(a.db.grants()?[0]["consumed_calls"], 1);
    a.revoke_peer(&b.id)?;
    match b.rpc(&a.id, "resources", json!({})).await {
        Ok(r) => assert!(r.payload.code.contains("REVOKED")),
        Err(e) => assert!(e.to_string().contains("TRANSPORT_FAILURE")),
    };
    assert!(!a.db.trusted(&b.id)?);
    assert_eq!(a.db.grants()?[0]["consumed_calls"], 1);
    Ok(())
}
#[tokio::test]
async fn expired_invites_and_grants_fail_closed() -> Result<()> {
    let aroot = tempfile::tempdir()?;
    let broot = tempfile::tempdir()?;
    let a = node(aroot.path(), "A").await?;
    let b = node(broot.path(), "B").await?;
    a.create_porch("Test Porch")?;
    let mut invite = a.invite(&json!({"recipient":b.id,"ttl_seconds":30}))?;
    invite.payload.expires_at = now() - 1;
    invite = Signed::new(&a.key, "porch.invite.v1", invite.payload)?;
    assert!(b.join(invite).await.is_err());
    a.trust(&b.id, "B", None, &json!({}))?;
    b.trust(&a.id, "A", None, &json!({}))?;
    connect(&b, &a).await?;
    let grant = a.issue_grant(Grant {
        issuer: a.id.clone(),
        recipient: b.id.clone(),
        capability: "model.inference".into(),
        resource: "test-model".into(),
        action: "run".into(),
        porch: None,
        limits: Limits::default(),
        created_at: now(),
        expires_at: now() + 1,
        nonce: nonce(),
        exact_input_digest: None,
    })?;
    tokio::time::sleep(Duration::from_millis(1200)).await;
    let job = Job {
        version: 1,
        id: nonce(),
        requester: b.id.clone(),
        capability: "model.inference".into(),
        resource: "test-model".into(),
        input: "x".into(),
        input_digest: digest(b"x"),
        max_output_tokens: 16,
        timeout_ms: 1000,
        privacy: Privacy::TrustedPeers,
        execution_mode: "REMOTE_NODE".into(),
        preferred_peer: Some(a.id.clone()),
        grant: Some(grant),
    };
    let result = b.rpc(&a.id, "job.run", serde_json::to_value(job)?).await?;
    assert!(
        result.payload.output["payload"]["reason"]
            .as_str()
            .unwrap()
            .contains("EXPIRED")
    );
    Ok(())
}
#[test]
fn ledger_modification_and_backward_clock_are_detected() -> Result<()> {
    let root = tempfile::tempdir()?;
    let n = Node::open(root.path(), Config::default())?;
    n.db.set("clock", &json!(now() + 60))?;
    assert!(n.db.time().is_err());
    n.db.transaction(|tx| {
        tx.execute("UPDATE ledger SET hash='corrupted' WHERE sequence=1", [])?;
        Ok(())
    })?;
    assert!(n.db.verify_ledger(&n.id).is_err());
    Ok(())
}
