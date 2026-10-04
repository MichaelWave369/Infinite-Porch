//! Dynamic HTTP contract fixtures, explicitly without Ollama weights.
use anyhow::Result;
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use porch_node::{
    Config, Node,
    models::{self, ModelSpec},
};
use serde_json::{Value, json};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};
#[derive(Default)]
struct Runtime {
    present: AtomicBool,
    loaded: AtomicBool,
    changed: AtomicBool,
    change_during: AtomicBool,
    die_during: AtomicBool,
    calls: AtomicUsize,
}
async fn fixture() -> Result<(String, Arc<Runtime>, tokio::task::JoinHandle<()>)> {
    let state = Arc::new(Runtime::default());
    state.present.store(true, Ordering::SeqCst);
    let app=Router::new()
        .route("/api/tags",get(|State(s):State<Arc<Runtime>>|async move {Json(json!({"models":if s.present.load(Ordering::SeqCst){vec![json!({"name":"runtime","digest":if s.changed.load(Ordering::SeqCst){"digest-new"}else{"digest-old"}})]}else{vec![]}}))}))
        .route("/api/ps",get(|State(s):State<Arc<Runtime>>|async move {Json(json!({"models":if s.loaded.load(Ordering::SeqCst){vec![json!({"name":"runtime"})]}else{vec![]}}))}))
        .route("/api/version",get(||async{Json(json!({"version":"HTTP-FIXTURE"}))}))
        .route("/api/generate",post(|State(s):State<Arc<Runtime>>,Json(_):Json<Value>|async move {
            s.calls.fetch_add(1,Ordering::SeqCst);s.loaded.store(true,Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(100)).await;
            if s.change_during.swap(false,Ordering::SeqCst){s.changed.store(true,Ordering::SeqCst);}
            if s.die_during.swap(false,Ordering::SeqCst){return (StatusCode::SERVICE_UNAVAILABLE,Json(json!({"error":"fixture process death"})));}
            (StatusCode::OK,Json(json!({"done":true,"response":"public HTTP fixture","eval_count":3})))
        })).with_state(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let url = format!("http://{}", listener.local_addr()?);
    let task = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    Ok((url, state, task))
}
fn cfg(url: &str) -> Config {
    Config {
        ollama_url: url.into(),
        models: vec![ModelSpec {
            name: "runtime".into(),
            provider: "ollama".into(),
            exposed: true,
            version: Some("digest-old".into()),
        }],
        ..Config::default()
    }
}
fn args() -> Value {
    json!({"resource":"runtime","input":"public fixture","max_output_tokens":16,"privacy":"LOCAL_ONLY"})
}
#[tokio::test]
async fn reachability_loaded_inventory_and_invocation_are_separate() -> Result<()> {
    let (url, state, task) = fixture().await?;
    let root = tempfile::tempdir()?;
    let n = Node::open(root.path(), cfg(&url))?;
    models::scan(&n).await?;
    let before = models::states(&n)?;
    assert_eq!(before["states"][0]["installed_now"], true);
    assert_eq!(before["states"][0]["loaded_now"], false);
    assert!(before["states"][0]["last_verified"].is_null());
    assert_eq!(
        before["states"][0]["remote_invocation_permitted_for_at_least_one_peer"],
        false
    );
    assert_eq!(n.run(&args()).await?["status"], "COMPLETED");
    models::scan(&n).await?;
    assert_eq!(models::states(&n)?["states"][0]["loaded_now"], true);
    assert_eq!(
        models::states(&n)?["states"][0]["last_verified"]["invocation_successful"],
        true
    );
    state.present.store(false, Ordering::SeqCst);
    models::scan(&n).await?;
    assert_eq!(models::states(&n)?["states"][0]["installed_now"], false);
    let r = n.run(&args()).await?;
    assert_eq!(r["status"], "REFUSED");
    assert_eq!(r["receipt"]["payload"]["reason"], "MODEL_NOT_INSTALLED");
    assert_eq!(state.calls.load(Ordering::SeqCst), 1);
    task.abort();
    models::scan(&n).await?;
    assert_eq!(
        models::states(&n)?["states"][0]["currently_reachable"],
        false
    );
    Ok(())
}
#[tokio::test]
async fn changed_model_timeout_and_provider_death_never_return_substitute_output() -> Result<()> {
    let (url, state, task) = fixture().await?;
    let root = tempfile::tempdir()?;
    let n = Node::open(root.path(), cfg(&url))?;
    state.change_during.store(true, Ordering::SeqCst);
    let r = n.run(&args()).await?;
    assert_eq!(r["status"], "REFUSED");
    assert_eq!(
        r["receipt"]["payload"]["reason"],
        "MODEL_CHANGED_DURING_INFERENCE"
    );
    assert!(r["receipt"]["payload"]["output"].is_null());
    let r = n.run(&args()).await?;
    assert_eq!(
        r["receipt"]["payload"]["reason"],
        "MODEL_VERSION_CHANGED_RESHARE_REQUIRED"
    );
    assert_eq!(state.calls.load(Ordering::SeqCst), 1);
    state.changed.store(false, Ordering::SeqCst);
    state.die_during.store(true, Ordering::SeqCst);
    assert_eq!(n.run(&args()).await?["status"], "REFUSED");
    let mut a = args();
    a["timeout_ms"] = json!(20);
    let r = n.run(&a).await?;
    assert_eq!(r["status"], "REFUSED");
    assert!(r["receipt"]["payload"]["output"].is_null());
    assert_eq!(
        models::states(&n)?["states"][0]["last_verified"]["invocation_successful"],
        false
    );
    task.abort();
    Ok(())
}
