use anyhow::{Result, ensure};
use futures::StreamExt;
use porch_core::{Job, MAX_FRAME, ModelManifest, digest, now};
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
            && matches!(url.host_str(), Some("127.0.0.1" | "[::1]" | "::1"))
            && url.username().is_empty()
            && url.password().is_none()
            && url.path() == "/"
            && url.query().is_none()
            && url.fragment().is_none(),
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
        client()?
            .get(format!("{}/api/tags", url.trim_end_matches('/')))
            .send()
            .await?,
        MAX_FRAME,
    )
    .await?;
    let mut out = Vec::new();
    let models = value["models"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("INVALID_MODEL_INVENTORY"))?;
    ensure!(models.len() <= 128, "MODEL_INVENTORY_LIMIT");
    {
        for m in models {
            if let Some(name) = m["name"].as_str() {
                ensure!(crate::valid_label(name, 128), "INVALID_MODEL_NAME");
                let version = m["digest"]
                    .as_str()
                    .filter(|v| crate::valid_label(v, 128))
                    .ok_or_else(|| anyhow::anyhow!("INVALID_MODEL_DIGEST"))?;
                out.push(ModelManifest{model:name.into(),provider:"ollama".into(),version:Some(version.to_string()),claims:json!({"context_limit":null,"vision":null,"tools":null,"status":"provider-reported-installed"})});
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
    // Installed once does not establish present availability or a stable digest.
    let inventory = discover(url).await?;
    let installed = inventory
        .iter()
        .find(|v| v.model == m.name)
        .ok_or_else(|| anyhow::anyhow!("MODEL_NOT_INSTALLED"))?;
    ensure!(
        m.version.is_none() || installed.version == m.version,
        "MODEL_VERSION_CHANGED_RESHARE_REQUIRED"
    );
    let reply=client()?.post(format!("{}/api/generate",url.trim_end_matches('/'))).json(&json!({"model":m.name,"prompt":job.input,"stream":false,"think":false,"options":{"num_predict":job.max_output_tokens,"num_ctx":8192}})).send().await?;
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
    let after = discover(url).await?;
    ensure!(
        after
            .iter()
            .any(|v| v.model == m.name && v.version == installed.version),
        "MODEL_CHANGED_DURING_INFERENCE"
    );
    Ok((
        json!({"text":v["response"],"mock":false}),
        "ollama".into(),
        installed.version.clone(),
        json!({"input_bytes":job.input.len(),"output_tokens":tokens,"prompt_tokens":v["prompt_eval_count"].as_u64(),"provider_total_duration_ns":v["total_duration"].as_u64(),"gpu_seconds":null}),
    ))
}

pub async fn scan(node: &crate::Node) -> Result<Value> {
    let url = node.config.read().unwrap().ollama_url.clone();
    let started = std::time::Instant::now();
    let result = tokio::time::timeout(Duration::from_secs(3), discover(&url)).await;
    let (reachable, inventory, reason) = match result {
        Ok(Ok(v)) => (true, v, None),
        Ok(Err(e)) => (false, vec![], Some(crate::safe_error(&e))),
        Err(_) => (false, vec![], Some("OLLAMA_SCAN_TIMEOUT".into())),
    };
    let loaded = if reachable {
        match tokio::time::timeout(Duration::from_secs(2), async {
            bounded_json(
                client()?
                    .get(format!("{}/api/ps", url.trim_end_matches('/')))
                    .send()
                    .await?,
                MAX_FRAME,
            )
            .await
        })
        .await
        {
            Ok(Ok(v)) => Some(v["models"].clone()),
            _ => None,
        }
    } else {
        None
    };
    let version = if reachable {
        match tokio::time::timeout(Duration::from_secs(2), async {
            bounded_json(
                client()?
                    .get(format!("{}/api/version", url.trim_end_matches('/')))
                    .send()
                    .await?,
                4096,
            )
            .await
        })
        .await
        {
            Ok(Ok(v)) => v["version"]
                .as_str()
                .filter(|s| crate::valid_label(s, 64))
                .map(str::to_string),
            _ => None,
        }
    } else {
        None
    };
    // Retain only selected names from ps. Provider responses are not trusted logs.
    let loaded_names = loaded.and_then(|v| {
        v.as_array().map(|a| {
            a.iter()
                .take(128)
                .filter_map(|m| {
                    m["name"]
                        .as_str()
                        .filter(|s| crate::valid_label(s, 128))
                        .map(str::to_string)
                })
                .collect::<Vec<_>>()
        })
    });
    let value = json!({"provider":"ollama","observed_at":now(),"observation_ttl_seconds":30,"currently_reachable":reachable,"provider_version":version,"inventory":inventory,"loaded_models":loaded_names,"reason":reason,"duration_ms":started.elapsed().as_millis() as u64,"sharing_automatically_enabled":false});
    node.db.set("provider_inventory", &value)?;
    Ok(value)
}
pub fn states(node: &crate::Node) -> Result<Value> {
    let scan = node.db.get("provider_inventory")?.unwrap_or(Value::Null);
    let fresh = scan["observed_at"]
        .as_u64()
        .is_some_and(|t| now().saturating_sub(t) <= 30);
    let config = node.config.read().unwrap().clone();
    let grants = node.db.grants()?;
    let time = node.db.time()?;
    let mut models = Vec::new();
    for m in &config.models {
        let last = node
            .db
            .get(&format!("model_verification:{}", digest(m.name.as_bytes())))?;
        let installed = scan["inventory"]
            .as_array()
            .is_some_and(|a| a.iter().any(|v| v["model"] == m.name));
        let loaded = scan["loaded_models"]
            .as_array()
            .map(|a| a.iter().any(|v| v == &m.name));
        let grant = m.exposed
            && grants.iter().any(|row| {
                let Ok(g) = serde_json::from_value::<porch_core::Signed<porch_core::Grant>>(
                    row["grant"].clone(),
                ) else {
                    return false;
                };
                row["revoked"] == false
                    && g.signer == node.id
                    && node.db.trusted(&g.payload.recipient).unwrap_or(false)
                    && g.payload
                        .validate(
                            &node.id,
                            &g.payload.recipient,
                            "model.inference",
                            &m.name,
                            "run",
                            time,
                        )
                        .is_ok()
                    && row["consumed_calls"].as_u64().unwrap_or(u64::MAX)
                        < g.payload.limits.max_calls
            });
        models.push(json!({"model":m.name,"provider":m.provider,"advertised":m.exposed,"installed_now":if m.provider=="mock"{Some(true)}else if fresh{Some(installed)}else{None},"currently_reachable":if m.provider=="mock"{Some(true)}else if fresh{scan["currently_reachable"].as_bool()}else{None},"loaded_now":if fresh{loaded}else{None},"last_verified":last,"remote_invocation_permitted_for_at_least_one_peer":grant,"local_operator_invocation_permitted":true,"observation_fresh":fresh,"availability_is_not_execution_proof":true}));
    }
    Ok(json!({"scan":scan,"states":models}))
}
type ExecutionResult = Result<(Value, String, Option<String>, Value)>;
pub fn record_execution(
    node: &crate::Node,
    model: &ModelSpec,
    result: &ExecutionResult,
) -> Result<()> {
    node.db.set(&format!("model_verification:{}",digest(model.name.as_bytes())),&json!({"verified_at":now(),"invocation_successful":result.is_ok(),"provider":model.provider,"version":result.as_ref().ok().and_then(|r|r.2.clone()),"reason":result.as_ref().err().map(crate::safe_error),"evidence":"actual provider invocation; not proof of future availability"}))
}
