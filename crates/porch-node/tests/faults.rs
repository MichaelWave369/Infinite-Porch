//! Application-layer faults over real loopback TCP. These never control a host
//! firewall, physical network or system-wide provider process.
use anyhow::Result;
use libp2p::identity::Keypair;
use porch_core::*;
use porch_node::{
    Config, Node,
    models::ModelSpec,
    network::{self, Transport},
};
use serde_json::json;
use std::{
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
    time::Duration,
};

struct FaultTransport {
    inner: Arc<dyn Transport>,
    mode: AtomicU8,
}
impl Transport for FaultTransport {
    fn frame_limit(&self) -> usize {
        self.inner.frame_limit()
    }
    fn connect(&self, a: &str, p: Option<&str>) -> Result<()> {
        self.inner.connect(a, p)
    }
    fn disconnect(&self, p: &str) -> Result<()> {
        self.inner.disconnect(p)
    }
    fn request<'a>(
        &'a self,
        peer: &'a str,
        request: Signed<WireRequest>,
    ) -> futures::future::BoxFuture<'a, Result<Signed<WireResponse>>> {
        Box::pin(async move {
            // Fault only job frames; resource discovery remains operational.
            let mode = if request.payload.operation == "job.run" {
                self.mode.swap(0, Ordering::SeqCst)
            } else {
                0
            };
            if mode == 4 {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            let mut response = self.inner.request(peer, request.clone()).await?;
            if mode == 1 {
                anyhow::bail!("INJECTED_RESPONSE_LOST");
            }
            if mode == 2 {
                let duplicate = self.inner.request(peer, request).await?;
                assert_eq!(duplicate.payload.status, "REFUSED");
                assert!(duplicate.payload.code.contains("REPLAY"));
            }
            if mode == 3 {
                response.payload.status = "CORRUPTED".into();
            }
            // An extra copy is held locally; the transport correlates one reply
            // per request ID. It cannot cause a second provider execution.
            if mode == 5 {
                let _duplicate_response = response.clone();
            }
            Ok(response)
        })
    }
}
async fn pair() -> Result<(
    tempfile::TempDir,
    tempfile::TempDir,
    Arc<Node>,
    Arc<Node>,
    Arc<FaultTransport>,
)> {
    let ar = tempfile::tempdir()?;
    let br = tempfile::tempdir()?;
    let cfg = Config {
        listen: vec!["/ip4/127.0.0.1/tcp/0".into()],
        mdns: false,
        models: vec![ModelSpec {
            name: "fault-model".into(),
            provider: "mock".into(),
            exposed: true,
            version: None,
        }],
        ..Config::default()
    };
    let a = Node::open(ar.path(), cfg.clone())?;
    let b = Node::open(
        br.path(),
        Config {
            models: vec![],
            ..cfg
        },
    )?;
    network::start(a.clone()).await?;
    network::start(b.clone()).await?;
    a.trust(&b.id, "B", None, &json!({}))?;
    b.trust(&a.id, "A", None, &json!({}))?;
    let mut address = None;
    for _ in 0..100 {
        address = a.observations.read().unwrap()["addresses"][0]
            .as_str()
            .map(str::to_string);
        if address.is_some() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    b.network
        .read()
        .unwrap()
        .as_ref()
        .unwrap()
        .connect(&address.unwrap(), Some(&a.id))?;
    b.import_grant(a.issue_grant(Grant {
        issuer: a.id.clone(),
        recipient: b.id.clone(),
        capability: "model.inference".into(),
        resource: "fault-model".into(),
        action: "run".into(),
        porch: None,
        limits: Limits::default(),
        created_at: now(),
        expires_at: now() + 600,
        nonce: nonce(),
        exact_input_digest: None,
    })?)?;
    let inner = b.network.read().unwrap().as_ref().unwrap().clone();
    let faults = Arc::new(FaultTransport {
        inner,
        mode: AtomicU8::new(0),
    });
    *b.network.write().unwrap() = Some(faults.clone());
    Ok((ar, br, a, b, faults))
}
#[tokio::test]
async fn lost_response_retries_original_executor_without_duplicate_effect() -> Result<()> {
    let (_ar, _br, a, b, faults) = pair().await?;
    let args = json!({"id":"lost-response","resource":"fault-model","input":"public fault fixture","privacy":"TRUSTED_PEERS"});
    faults.mode.store(1, Ordering::SeqCst);
    assert!(b.run(&args).await.is_err());
    assert_eq!(a.db.grants()?[0]["consumed_calls"], 1);
    let result = b.run(&args).await?;
    assert_eq!(result["status"], "COMPLETED");
    assert_eq!(a.db.grants()?[0]["consumed_calls"], 1);
    let original = a.db.records("jobs")?[0].clone();
    assert_eq!(result["receipt"], original);
    Ok(())
}
#[tokio::test]
async fn duplicate_delayed_and_corrupted_frames_preserve_binding() -> Result<()> {
    let (_ar, _br, a, b, faults) = pair().await?;
    for mode in [2, 4, 5] {
        faults.mode.store(mode, Ordering::SeqCst);
        let r=b.run(&json!({"resource":"fault-model","input":"public fault fixture","privacy":"TRUSTED_PEERS"})).await?;
        assert_eq!(r["status"], "COMPLETED");
    }
    assert_eq!(a.db.grants()?[0]["consumed_calls"], 3);
    faults.mode.store(3, Ordering::SeqCst);
    let args = json!({"id":"corrupted-response","resource":"fault-model","input":"public fault fixture","privacy":"TRUSTED_PEERS"});
    let err = b.run(&args).await.unwrap_err();
    assert!(err.to_string().contains("RESPONSE_SIGNATURE_INVALID"));
    assert_eq!(b.run(&args).await?["status"], "COMPLETED");
    assert_eq!(a.db.grants()?[0]["consumed_calls"], 4);
    Ok(())
}
#[tokio::test]
async fn stolen_or_replayed_invite_and_pairing_race_have_one_winner() -> Result<()> {
    let ar = tempfile::tempdir()?;
    let br = tempfile::tempdir()?;
    let cr = tempfile::tempdir()?;
    let a = Node::open(ar.path(), Config::default())?;
    let b = Node::open(br.path(), Config::default())?;
    let c = Node::open(cr.path(), Config::default())?;
    a.create_porch("Race")?;
    let invite = a.invite(&json!({"recipient":b.id}))?;
    let request = |key: &Keypair, alias: &str| {
        Signed::new(
            key,
            "porch.request.v1",
            WireRequest {
                id: nonce(),
                nonce: nonce(),
                created_at: now(),
                expires_at: now() + 60,
                operation: "porch.join".into(),
                args: json!({"invite":invite,"alias":alias}),
            },
        )
        .unwrap()
    };
    let stolen = a.receive(c.id.clone(), request(&c.key, "C")).await?;
    assert!(stolen.payload.code.contains("RECIPIENT"));
    let first = a.receive(b.id.clone(), request(&b.key, "B"));
    let second = a.receive(b.id.clone(), request(&b.key, "B"));
    let (one, two) = tokio::join!(first, second);
    let responses = [one?, two?];
    assert_eq!(
        responses
            .iter()
            .filter(|r| r.payload.status == "OK")
            .count(),
        1
    );
    assert_eq!(a.db.trust_rows()?.len(), 1);
    assert!(!a.db.trusted(&c.id)?);
    let mut forged = invite.clone();
    forged.payload.name = "forged".into();
    assert!(b.join(forged).await.is_err());
    Ok(())
}
