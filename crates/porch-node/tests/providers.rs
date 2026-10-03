//! HTTP fixtures exercise the real Ollama adapter contract. They contain no
//! model weights and do not qualify live inference or hardware reservations.
use anyhow::Result;
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use porch_core::*;
use porch_node::{
    Config, Node,
    models::{self, ModelSpec},
};
use serde_json::{Value, json};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

struct Fixture {
    calls: AtomicUsize,
    inputs: Mutex<Vec<Value>>,
}
async fn fixture() -> Result<(String, Arc<Fixture>, tokio::task::JoinHandle<()>)> {
    let state = Arc::new(Fixture {
        calls: AtomicUsize::new(0),
        inputs: Mutex::new(vec![]),
    });
    let app=Router::new().route("/api/tags",get(||async{Json(json!({"models":[{"name":"fixture","digest":"fixture-digest"}]}))})).route("/api/generate",post(|State(s):State<Arc<Fixture>>,Json(v):Json<Value>|async move {
        s.calls.fetch_add(1,Ordering::SeqCst);s.inputs.lock().unwrap().push(v.clone());
        match v["model"].as_str().unwrap_or("") {
            "invalid"=>Json(json!({"done":false,"error":"fixture failure"})),
            "large"=>Json(json!({"done":true,"response":"x".repeat(MAX_FRAME)})),
            "slow"=>{tokio::time::sleep(Duration::from_millis(500)).await;Json(json!({"done":true,"response":"contract fixture","eval_count":3}))},
            _=>Json(json!({"done":true,"response":"contract fixture","eval_count":3,"prompt_eval_count":5,"total_duration":1000000})),
        }
    })).with_state(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let url = format!("http://{}", listener.local_addr()?);
    let task = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    Ok((url, state, task))
}
fn model(name: &str) -> ModelSpec {
    ModelSpec {
        name: name.into(),
        provider: "ollama".into(),
        exposed: true,
        version: Some("fixture-digest".into()),
    }
}
fn job(peer: &str, name: &str) -> Job {
    Job {
        version: VERSION,
        id: nonce(),
        requester: peer.into(),
        capability: "model.inference".into(),
        resource: name.into(),
        input: "private fixture input".into(),
        input_digest: digest(b"private fixture input"),
        max_output_tokens: 16,
        timeout_ms: 2000,
        privacy: Privacy::LocalOnly,
        execution_mode: "SINGLE_NODE".into(),
        preferred_peer: None,
        grant: None,
    }
}
#[tokio::test]
async fn ollama_http_contract_and_failure_behavior() -> Result<()> {
    let (url, state, task) = fixture().await?;
    let installed = models::discover(&url).await?;
    assert_eq!(installed[0].version.as_deref(), Some("fixture-digest"));
    let j = job("fixture-peer", "fixture");
    let (out, provider, version, usage) =
        models::execute(&j, Some(&model("fixture")), &url).await?;
    assert_eq!(provider, "ollama");
    assert_eq!(out["mock"], false);
    assert_eq!(version.as_deref(), Some("fixture-digest"));
    assert_eq!(usage["output_tokens"], 3);
    let sent = state.inputs.lock().unwrap()[0].clone();
    assert_eq!(sent["prompt"], j.input);
    assert_eq!(sent["stream"], false);
    assert_eq!(sent["options"]["num_predict"], 16);
    for name in ["invalid", "large"] {
        assert!(
            models::execute(&job("fixture-peer", name), Some(&model(name)), &url)
                .await
                .is_err()
        );
    }
    assert!(models::validate_ollama("http://192.0.2.1:11434").is_err());
    let unused = std::net::TcpListener::bind("127.0.0.1:0")?;
    let unavailable = format!("http://{}", unused.local_addr()?);
    drop(unused);
    assert!(
        models::execute(&j, Some(&model("fixture")), &unavailable)
            .await
            .is_err(),
        "unavailable real adapter must not substitute mock output"
    );
    task.abort();
    Ok(())
}
#[tokio::test]
async fn slow_provider_duplicate_timeout_and_revocation_are_bounded() -> Result<()> {
    let (url, state, task) = fixture().await?;
    let root = tempfile::tempdir()?;
    let n = Node::open(
        root.path(),
        Config {
            ollama_url: url,
            models: vec![model("slow")],
            ..Config::default()
        },
    )?;
    let j = job(&n.id, "slow");
    let one = n.clone();
    let original = j.clone();
    let work = tokio::spawn(async move { one.execute_job(original, true).await });
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(
        n.execute_job(j.clone(), true)
            .await
            .unwrap_err()
            .to_string()
            .contains("DUPLICATE_JOB_RUNNING")
    );
    let receipt = work.await??;
    assert_eq!(receipt.payload.status, "COMPLETED");
    assert_eq!(n.execute_job(j, true).await?.signature, receipt.signature);
    assert_eq!(state.calls.load(Ordering::SeqCst), 1);
    let mut timeout = job(&n.id, "slow");
    timeout.timeout_ms = 100;
    let r = n.execute_job(timeout, true).await?;
    assert_eq!(r.payload.status, "REFUSED");
    assert!(r.payload.reason.unwrap().contains("TIMEOUT"));
    assert!(r.payload.output.is_none());
    let peer = libp2p::identity::Keypair::generate_ed25519()
        .public()
        .to_peer_id()
        .to_string();
    n.trust(&peer, "approved-test-peer", None, &json!({}))?;
    let grant = n.issue_grant(Grant {
        issuer: n.id.clone(),
        recipient: peer.clone(),
        capability: "model.inference".into(),
        resource: "slow".into(),
        action: "run".into(),
        porch: None,
        limits: Limits::default(),
        created_at: now(),
        expires_at: now() + 600,
        nonce: nonce(),
        exact_input_digest: None,
    })?;
    let mut remote = job(&peer, "slow");
    remote.privacy = Privacy::TrustedPeers;
    remote.execution_mode = "REMOTE_NODE".into();
    remote.grant = Some(grant.clone());
    let one = n.clone();
    let work = tokio::spawn(async move { one.execute_job(remote, false).await });
    tokio::time::sleep(Duration::from_millis(100)).await;
    n.revoke_grant(&grant.payload.nonce)?;
    let r = work.await??;
    assert_eq!(r.payload.status, "REFUSED");
    assert!(r.payload.reason.unwrap().contains("REVOKED"));
    assert!(r.payload.output.is_none());
    task.abort();
    Ok(())
}
