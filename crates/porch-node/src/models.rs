use anyhow::{Result, ensure};
use futures::StreamExt;
use porch_core::{Job, MAX_FRAME, ModelManifest, digest};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelSpec {
    pub name: String,
    pub provider: String,
    pub exposed: bool,
    pub version: Option<String>,
}
pub fn client() -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(30))
        .build()?)
}
pub fn validate_ollama(url: &str) -> Result<()> {
    let url = reqwest::Url::parse(url)?;
    ensure!(
        url.scheme() == "http"
            && matches!(
                url.host_str(),
                Some("127.0.0.1" | "[::1]" | "::1" | "localhost")
            )
            && url.username().is_empty()
            && url.password().is_none()
            && url.path() == "/"
            && url.query().is_none(),
        "OLLAMA_MUST_BE_LOOPBACK"
    );
    Ok(())
}
async fn bounded_json(response: reqwest::Response, limit: usize) -> Result<Value> {
    let response = response.error_for_status()?;
    if let Some(len) = response.content_length() {
        ensure!(len <= limit as u64, "PROVIDER_RESPONSE_TOO_LARGE");
    }
    let mut chunks = response.bytes_stream();
    let mut bytes = Vec::new();
    while let Some(chunk) = chunks.next().await {
        let chunk = chunk?;
        ensure!(
            bytes.len() + chunk.len() <= limit,
            "PROVIDER_RESPONSE_TOO_LARGE"
        );
        bytes.extend(chunk);
    }
    Ok(serde_json::from_slice(&bytes)?)
}
pub async fn discover(url: &str) -> Result<Vec<ModelManifest>> {
    validate_ollama(url)?;
    let value = bounded_json(
        client()?.get(format!("{url}/api/tags")).send().await?,
        MAX_FRAME,
    )
    .await?;
    let mut out = Vec::new();
    if let Some(models) = value["models"].as_array() {
        for m in models.iter().take(128) {
            if let Some(name) = m["name"].as_str() {
                out.push(ModelManifest{model:name.into(),provider:"ollama".into(),version:m["digest"].as_str().map(str::to_string),claims:json!({"context_limit":null,"vision":null,"tools":null,"status":"provider-reported-installed"})});
            }
        }
    }
    Ok(out)
}
pub async fn execute(
    job: &Job,
    model: Option<&ModelSpec>,
    url: &str,
) -> Result<(Value, String, Option<String>, Value)> {
    if job.capability == "compute.hash" {
        ensure!(job.resource == "sha256", "UNKNOWN_BUILTIN");
        return Ok((
            json!({"sha256":digest(job.input.as_bytes())}),
            "builtin-sha256".into(),
            Some("1".into()),
            json!({"input_bytes":job.input.len()}),
        ));
    }
    ensure!(
        job.capability == "model.inference",
        "CAPABILITY_NOT_INSTALLED"
    );
    let m = model.ok_or_else(|| anyhow::anyhow!("MODEL_UNAVAILABLE"))?;
    ensure!(job.resource == m.name, "MODEL_UNAVAILABLE");
    if m.provider == "mock" {
        return Ok((
            json!({"text":format!("MOCK inference {} input-sha256:{}",m.name,job.input_digest),"mock":true}),
            "mock".into(),
            Some("deterministic-v1".into()),
            json!({"input_bytes":job.input.len(),"tokens":null,"gpu_seconds":null}),
        ));
    }
    ensure!(m.provider == "ollama", "PROVIDER_NOT_INSTALLED");
    validate_ollama(url)?;
    let reply=client()?.post(format!("{url}/api/generate")).json(&json!({"model":m.name,"prompt":job.input,"stream":false,"think":false,"options":{"num_predict":job.max_output_tokens,"num_ctx":8192}})).send().await?;
    let v = bounded_json(reply, MAX_FRAME / 2).await?;
    ensure!(
        v["done"] == true && v["response"].is_string() && v.get("error").is_none(),
        "INVALID_PROVIDER_RESPONSE"
    );
    let tokens = v["eval_count"].as_u64();
    ensure!(
        tokens.is_none_or(|n| n <= job.max_output_tokens as u64),
        "PROVIDER_OUTPUT_TOKEN_LIMIT_EXCEEDED"
    );
    Ok((
        json!({"text":v["response"],"mock":false}),
        "ollama".into(),
        m.version.clone(),
        json!({"input_bytes":job.input.len(),"output_tokens":tokens,"prompt_tokens":v["prompt_eval_count"].as_u64(),"provider_total_duration_ns":v["total_duration"].as_u64(),"gpu_seconds":null}),
    ))
}
