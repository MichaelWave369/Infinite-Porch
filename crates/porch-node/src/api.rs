use crate::{
    Node, clients,
    models::{self, ModelSpec},
    network, qualification, storage,
};
use anyhow::{Context, Result, ensure};
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, Request, State},
    http::{HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response, Sse, sse::Event},
    routing::{get, post},
};
use futures::StreamExt;
use porch_core::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{convert::Infallible, path::PathBuf, sync::Arc, time::Duration};
use tower_http::services::ServeDir;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Control {
    pub operation: String,
    pub args: Value,
}
fn refused(e: anyhow::Error) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({"status":"REFUSED","reason":e.to_string()})),
    )
        .into_response()
}
async fn guard(State(node): State<Arc<Node>>, mut request: Request, next: Next) -> Response {
    let api = node.config.read().unwrap().api.clone();
    let host = request
        .headers()
        .get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    if host != api {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"status":"REFUSED","reason":"HOST_NOT_LOOPBACK_API"})),
        )
            .into_response();
    }
    if let Some(origin) = request.headers().get("origin")
        && origin.to_str().ok() != Some(format!("http://{api}").as_str())
    {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"status":"REFUSED","reason":"CROSS_ORIGIN_REFUSED"})),
        )
            .into_response();
    }
    if request.uri().path().starts_with("/v1/") {
        let supplied = request
            .headers()
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .unwrap_or("");
        // Fixed-size digest comparison avoids content-dependent early exits.
        let a = hex::decode(digest(supplied.as_bytes())).unwrap_or_default();
        let b = hex::decode(digest(node.api_token.as_bytes())).unwrap_or_default();
        let different = a.iter().zip(&b).fold(0u8, |d, (a, b)| d | (a ^ b));
        if different != 0 {
            let grant = match clients::lookup(&node, supplied) {
                Ok(g) => g,
                Err(e) => {
                    return (
                        StatusCode::UNAUTHORIZED,
                        Json(json!({"status":"REFUSED","reason":e.to_string()})),
                    )
                        .into_response();
                }
            };
            let path = request.uri().path().to_string();
            let authorization =
                if path == "/v1/control" && request.method() == axum::http::Method::POST {
                    let (parts, body) = request.into_parts();
                    let bytes =
                        match axum::body::to_bytes(body, MAX_FRAME).await {
                            Ok(b) => b,
                            Err(_) => return (
                                StatusCode::PAYLOAD_TOO_LARGE,
                                Json(
                                    json!({"status":"REFUSED","reason":"CLIENT_REQUEST_TOO_LARGE"}),
                                ),
                            )
                                .into_response(),
                        };
                    let control: Control =
                        match serde_json::from_slice(&bytes) {
                            Ok(c) => c,
                            Err(_) => return (
                                StatusCode::BAD_REQUEST,
                                Json(json!({"status":"REFUSED","reason":"INVALID_CLIENT_REQUEST"})),
                            )
                                .into_response(),
                        };
                    let allowed = clients::authorize(
                        &node,
                        &grant,
                        None,
                        Some(&control.operation),
                        &control.args,
                    );
                    request = Request::from_parts(parts, axum::body::Body::from(bytes));
                    allowed
                } else {
                    clients::authorize(
                        &node,
                        &grant,
                        Some(path.trim_start_matches("/v1/")),
                        None,
                        &json!({}),
                    )
                };
            if let Err(e) = authorization {
                return (
                    StatusCode::FORBIDDEN,
                    Json(json!({"status":"REFUSED","reason":e.to_string()})),
                )
                    .into_response();
            }
        }
    }
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert("cache-control", HeaderValue::from_static("no-store"));
    response.headers_mut().insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    response.headers_mut().insert("content-security-policy",HeaderValue::from_static("default-src 'self'; connect-src 'self'; img-src 'self' data:; style-src 'self'; script-src 'self'; frame-ancestors 'none'; base-uri 'none'"));
    response
}
async fn health() -> Json<Value> {
    Json(json!({"status":"READY","protocol_version":VERSION,"candidate":env!("CARGO_PKG_VERSION")}))
}
async fn read(State(node): State<Arc<Node>>, Path(kind): Path<String>) -> Response {
    if kind == "doctor" {
        return Json(qualification::doctor(&node).await).into_response();
    }
    let result = (|| -> Result<Value> {
        match kind.as_str() {
            "status" => node.snapshot(),
            "identity" => Ok(
                json!({"id":node.id,"public_key":hex::encode(node.key.public().encode_protobuf()),"address":format!("porch://peer/{}",node.id),"fingerprint_sha256":qualification::local_fingerprint(&node)}),
            ),
            "peers" => Ok(json!(node.db.trust_rows()?)),
            "resources" => Ok(serde_json::to_value(node.advertisement()?)?),
            "models" => Ok(
                json!({"local":node.config.read().unwrap().models,"remote":node.db.records("advertisements")?,"verification":models::states(&node)?}),
            ),
            "grants" => Ok(json!(node.db.grants()?)),
            "ledger" | "jobs" | "messages" => Ok(json!(node.db.records(&kind)?)),
            "storage" => storage::list(&node),
            "network" => Ok(network::diagnostics(&node)),
            "qualification" => qualification::status(&node),
            "active-jobs" => Ok(json!(node.db.active_jobs()?)),
            _ => anyhow::bail!("UNKNOWN_API_RESOURCE"),
        }
    })();
    match result {
        Ok(value) => bounded_response(&node, value),
        Err(e) => refused(e),
    }
}
async fn control(State(node): State<Arc<Node>>, Json(request): Json<Control>) -> Response {
    match operate(&node, &request.operation, &request.args).await {
        Ok(value) => bounded_response(&node, value),
        Err(e) => refused(e),
    }
}
async fn events(
    State(node): State<Arc<Node>>,
) -> Sse<impl futures::Stream<Item = std::result::Result<Event, Infallible>>> {
    let rx = node.events.subscribe();
    let stream = tokio_stream::wrappers::BroadcastStream::new(rx).map(|v| {
        Ok(Event::default().event("ledger").data(
            v.map(|v| v.to_string())
                .unwrap_or_else(|_| json!({"type":"GAP","action":"refetch-ledger"}).to_string()),
        ))
    });
    Sse::new(stream)
        .keep_alive(axum::response::sse::KeepAlive::new().interval(Duration::from_secs(15)))
}
pub async fn serve(node: Arc<Node>, ui: PathBuf) -> Result<()> {
    let api = node.config.read().unwrap().api.clone();
    let listener = tokio::net::TcpListener::bind(&api).await?;
    node.config.write().unwrap().api = listener.local_addr()?.to_string();
    let request_limit = node.config.read().unwrap().limits.request_body_bytes;
    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/control", post(control))
        .route("/v1/events", get(events))
        .route("/v1/{kind}", get(read))
        .fallback_service(ServeDir::new(ui).append_index_html_on_directories(true))
        .layer(DefaultBodyLimit::max(request_limit))
        .layer(middleware::from_fn_with_state(node.clone(), guard))
        .with_state(node);
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
pub async fn operate(node: &Arc<Node>, operation: &str, args: &Value) -> Result<Value> {
    let required = |key: &str| -> Result<String> {
        Ok(args[key]
            .as_str()
            .with_context(|| format!("{key}_REQUIRED"))?
            .to_string())
    };
    match operation {
        "qualification.run" => qualification::run(node,args).await,
        "qualification.export" => qualification::export(node),
        "qualification.host" => {
            let name=required("name")?;
            if let Some(current)=node.db.get("porch")?.filter(|v|!v.is_null()) { ensure!(current["name"]==name && current["issuer"]==node.id,"EXISTING_PORCH_REQUIRES_EXPLICIT_OPERATOR_CHANGE"); }
            else {node.create_porch(&name)?;}
            let recipient=required("recipient")?; let _:libp2p::PeerId=recipient.parse()?;
            Ok(serde_json::to_value(node.invite(&json!({"recipient":recipient,"ttl_seconds":600}))?)?)
        },
        "qualification.join" => {
            let invite:Signed<Invite>=serde_json::from_value(args["invite"].clone())?;
            invite.verify("porch.invite.v1")?;
            ensure!(qualification::fingerprint(&invite.public_key)?==required("fingerprint")?.replace([' ',':','-'],"").to_ascii_lowercase(),"PAIRING_FINGERPRINT_MISMATCH");
            ensure!(invite.payload.recipient.as_deref()==Some(&node.id),"QUALIFICATION_REQUIRES_RECIPIENT_BOUND_INVITE");
            node.join(invite).await
        },
        "model.scan" | "model.refresh" => models::scan(node).await,
        "model.verify" => node.run(&json!({"resource":required("model")?,"input":qualification::PUBLIC_PROMPT,"privacy":"LOCAL_ONLY","max_output_tokens":16})).await,
        "client.issue" => clients::issue(node, args),
        "client.revoke" => clients::revoke(node, &required("nonce")?),
        "peer.approve" => {
            let peer = required("peer")?;
            node.trust(
                &peer,
                &required("alias")?,
                args["porch"].as_str(),
                &json!({"source":"explicit-operator"}),
            )?;
            Ok(json!({"approved":peer,"effect_authority":false}))
        }
        "peer.revoke" => node.revoke_peer(&required("peer")?),
        "peer.rotate" => {
            let old: Signed<Value> = serde_json::from_value(args["old"].clone())?;
            let new: Signed<Value> = serde_json::from_value(args["new"].clone())?;
            old.verify("porch.rotation.v1")?;
            new.verify("porch.rotation.v1")?;
            ensure!(
                old.payload == new.payload
                    && old.payload["old_peer"] == old.signer
                    && old.payload["new_peer"] == new.signer
                    && old.payload["authority_carried"] == false
                    && old.payload["expires_at"].as_u64().unwrap_or(0) > node.db.time()?
                    && old.payload["created_at"].as_u64().unwrap_or(u64::MAX) <= now() + 2,
                "ROTATION_PROOF_INVALID"
            );
            ensure!(
                node.db.trusted(&old.signer)?,
                "ROTATION_OLD_IDENTITY_NOT_TRUSTED"
            );
            node.db.replay(
                &old.signer,
                old.payload["nonce"]
                    .as_str()
                    .context("ROTATION_NONCE_REQUIRED")?,
                old.payload["expires_at"].as_u64().unwrap_or(0),
            )?;
            let alias = node
                .db
                .trust_rows()?
                .into_iter()
                .find(|r| r["peer"] == old.signer)
                .context("OLD_PEER_NOT_FOUND")?["alias"]
                .as_str()
                .context("OLD_ALIAS_MISSING")?
                .to_string();
            node.revoke_peer(&old.signer)?;
            node.trust(&new.signer, &alias, None, args)?;
            Ok(
                json!({"old_revoked":old.signer,"new_approved":new.signer,"effect_authority_carried":false,"porch_membership_carried":false}),
            )
        }
        "peer.connect" => {
            node.network
                .read()
                .unwrap()
                .clone()
                .context("NETWORK_NOT_STARTED")?
                .connect(&required("address")?, args["expected_peer"].as_str())?;
            Ok(json!({"status":"CONNECTING"}))
        }
        "peer.request" => {
            let peer = required("peer")?;
            let operation = required("operation")?;
            ensure!(node.db.trusted(&peer)?, "UNTRUSTED_PEER");
            ensure!(
                matches!(
                    operation.as_str(),
                    "resources"
                        | "blob.begin"
                        | "blob.chunk"
                        | "blob.get"
                        | "blob.delete"
                        | "message.send"
                ),
                "RAW_PEER_OPERATION_NOT_ALLOWED"
            );
            Ok(serde_json::to_value(
                node.rpc(&peer, &operation, args["args"].clone()).await?,
            )?)
        }
        "porch.create" => node.create_porch(&required("name")?),
        "porch.invite" => Ok(serde_json::to_value(node.invite(args)?)?),
        "porch.join" => {
            node.join(serde_json::from_value(args["invite"].clone())?)
                .await
        }
        "porch.leave" => {
            let porch = node.db.get("porch")?.context("NOT_IN_PORCH")?;
            node.db.transaction(|tx| {
                tx.execute(
                    "UPDATE grants SET revoked=1 WHERE record LIKE ?1",
                    [format!("%{}%", porch["id"].as_str().unwrap_or(""))],
                )?;
                Ok(())
            })?;
            node.db.set("porch", &Value::Null)?;
            node.event("porch.left", porch["id"].as_str().unwrap_or(""), json!({}))?;
            Ok(json!({"left":porch["id"]}))
        }
        "grant.issue" => {
            let ttl = args["ttl_seconds"].as_u64().unwrap_or(3600);
            ensure!(ttl > 0 && ttl <= 86400, "INVALID_GRANT_TTL");
            let mut limits = Limits::default();
            if let Some(v) = args.get("limits") {
                let mut base = serde_json::to_value(&limits)?;
                for (k, v) in v.as_object().context("INVALID_LIMITS")? {
                    base[k] = v.clone();
                }
                limits = serde_json::from_value(base)?;
            }
            let g = Grant {
                issuer: node.id.clone(),
                recipient: required("recipient")?,
                capability: required("capability")?,
                resource: required("resource")?,
                action: args["action"].as_str().unwrap_or("run").into(),
                porch: args["porch"].as_str().map(str::to_string),
                limits,
                created_at: now(),
                expires_at: node.db.time()? + ttl,
                nonce: nonce(),
                exact_input_digest: args["exact_input_digest"].as_str().map(str::to_string),
            };
            Ok(serde_json::to_value(node.issue_grant(g)?)?)
        }
        "grant.import" => node.import_grant(serde_json::from_value(args["grant"].clone())?),
        "grant.revoke" => node.revoke_grant(&required("nonce")?),
        "resources.refresh" => node.refresh().await,
        "model.discover" => {
            let url = node.config.read().unwrap().ollama_url.clone();
            Ok(
                json!({"models":models::discover(&url).await?,"sharing_automatically_enabled":false}),
            )
        }
        "share.configure" => {
            let mut c = node.config.read().unwrap().clone();
            if let Some(v) = args["compute_hash"].as_bool() {
                c.compute_hash = v;
            }
            if let Some(v) = args["storage_quota"].as_u64() {
                ensure!(v <= c.limits.storage_total_bytes, "STORAGE_QUOTA_EXCEEDS_OPERATOR_CAP");
                c.storage_quota = v;
            }
            if let Some(model) = args.get("model") {
                let name = model["name"].as_str().context("MODEL_NAME_REQUIRED")?;
                ensure!(crate::valid_label(name,128), "INVALID_MODEL_NAME");
                let provider = model["provider"].as_str().unwrap_or("ollama");
                ensure!(
                    matches!(provider, "ollama" | "mock"),
                    "PROVIDER_NOT_INSTALLED"
                );
                let expose = model["exposed"].as_bool().unwrap_or(false);
                let version = if provider == "ollama" {
                    let models = models::discover(&c.ollama_url).await?;
                    models
                        .into_iter()
                        .find(|m| m.model == name)
                        .context("MODEL_NOT_INSTALLED")?
                        .version
                } else {
                    Some("deterministic-v1".into())
                };
                ensure!(c.models.len()<c.limits.services.saturating_sub(3) || c.models.iter().any(|m|m.name==name),"MODEL_SERVICE_LIMIT");
                if c.models.iter().any(|m|m.name==name && (m.version!=version || m.provider!=provider)) {
                    node.db.transaction(|tx| {
                        let rows={let mut query=tx.prepare("SELECT nonce,record FROM grants WHERE issuer=?1 AND revoked=0")?;
                            query.query_map([&node.id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?)))?.collect::<rusqlite::Result<Vec<_>>>()?};
                        for (nonce,record) in rows {let grant:Signed<Grant>=serde_json::from_str(&record)?;
                            if grant.payload.capability=="model.inference" && grant.payload.resource==name {
                                tx.execute("UPDATE grants SET revoked=1 WHERE nonce=?1",[nonce])?;
                            }
                        } Ok(())
                    })?;
                }
                c.models.retain(|m| m.name != name);
                c.models.push(ModelSpec {
                    name: name.into(),
                    provider: provider.into(),
                    exposed: expose,
                    version,
                });
            }
            let value = serde_json::to_value(&c)?;
            node.db.set("config", &value)?;
            node.db.transaction(|tx| {
                for row in {
                    let mut s = tx
                        .prepare("SELECT nonce,record FROM grants WHERE issuer=?1 AND revoked=0")?;
                    s.query_map([&node.id], |r| {
                        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
                    })?
                    .collect::<rusqlite::Result<Vec<_>>>()?
                } {
                    let (nonce, record) = row;
                    let grant: Signed<Grant> = serde_json::from_str(&record)?;
                    let allowed = match grant.payload.capability.as_str() {
                        "model.inference" => c
                            .models
                            .iter()
                            .any(|m| m.name == grant.payload.resource && m.exposed),
                        "compute.hash" => c.compute_hash,
                        "blob.storage" => {
                            c.storage_quota > 0
                                && grant.payload.limits.max_storage_bytes <= c.storage_quota
                        }
                        _ => true,
                    };
                    if !allowed {
                        tx.execute("UPDATE grants SET revoked=1 WHERE nonce=?1", [nonce])?;
                    }
                }
                Ok(())
            })?;
            *node.config.write().unwrap() = c;
            node.event(
                "sharing.configured",
                &node.id,
                json!({"configuration_hash":digest(&canonical(&value)?)}),
            )?;
            Ok(json!({"configured":true,"advertisement":node.advertisement()?}))
        }
        "job.run" => node.run(args).await,
        "storage.put" => {
            let data = hex::decode(required("plaintext_hex")?)?;
            storage::put_local(node, &data)
        }
        "storage.get" => storage::get_local(node, &required("cid")?),
        "storage.pin" => storage::pin(
            node,
            &required("cid")?,
            args["pinned"].as_bool().unwrap_or(true),
        ),
        "storage.delete" => storage::delete_local(node, &required("cid")?),
        "storage.replicate" => {
            storage::replicate(node, &required("peer")?, &required("cid")?).await
        }
        "storage.fetch" => storage::fetch(node, &required("peer")?, &required("cid")?).await,
        "storage.delete_remote" => {
            storage::delete_remote(node, &required("peer")?, &required("cid")?).await
        }
        "message.send" => node.send_message(args).await,
        "message.retry" => node.retry_outbox().await,
        "channel.send" => {
            let channel = required("channel")?;
            let text = required("text")?;
            let porch = node.db.get("porch")?.context("NO_PORCH")?;
            ensure!(porch["id"] == channel, "CHANNEL_NOT_ACTIVE");
            let mut receipts = Vec::new();
            for peer in node
                .db
                .trust_rows()?
                .into_iter()
                .filter(|r| r["active"] == true && r["porch"] == channel)
            {
                let result=node.send_message(&json!({"peer":peer["peer"],"text":text,"channel":channel,"queue_if_offline":args["queue_if_offline"]})).await;
                receipts.push(json!({"peer":peer["peer"],"result":result.as_ref().ok(),"refusal":result.err().map(|e|e.to_string())}));
            }
            Ok(json!({"channel":channel,"deliveries":receipts}))
        }
        "address.resolve" => {
            let address = required("address")?;
            let (kind, id) = logical_address(&address)?;
            match kind {
                "peer" => {
                    let rows = node.db.trust_rows()?;
                    Ok(
                        json!({"peer":rows.into_iter().find(|r|r["peer"]==id || r["alias"]==id),"alias_authority":"operator-pinned"}),
                    )
                }
                "porch" => {
                    let current = node.db.get("porch")?.filter(|v| v["id"] == id);
                    Ok(node
                        .db
                        .get(&format!("porch:{id}"))?
                        .or(current)
                        .unwrap_or(Value::Null))
                }
                "service" => {
                    let mut found = Value::Null;
                    let mut adverts = node.db.records("advertisements")?;
                    adverts.push(serde_json::to_value(node.advertisement()?)?);
                    for ad in adverts {
                        let ad: Signed<Advertisement> = serde_json::from_value(ad)?;
                        ad.verify("porch.advertisement.v1")?;
                        if ad.signer != node.id && !node.db.trusted(&ad.signer)? {
                            continue;
                        }
                        if ad.payload.expires_at <= node.db.time()? {
                            continue;
                        }
                        for v in ad.payload.services {
                            let s: Signed<Value> = serde_json::from_value(v)?;
                            s.verify("porch.service.v1")?;
                            if s.signer == ad.signer
                                && s.payload["service_id"] == id
                                && s.payload["expires_at"].as_u64().unwrap_or(0) > now()
                            {
                                found = serde_json::to_value(s)?;
                            }
                        }
                    }
                    Ok(found)
                }
                "blob" => {
                    Ok(json!({"cid":id,"stored_locally":node.root.join("vault").join(id).exists()}))
                }
                "model" => Ok(json!({"logical_model":id,"execution_requires_grant":true})),
                _ => anyhow::bail!("ADDRESS_TYPE_UNSUPPORTED"),
            }
        }
        "relay.enable" | "federation.enable" | "compute.wasm" => {
            anyhow::bail!("EXECUTION_ENGINE_NOT_INSTALLED")
        }
        _ => anyhow::bail!("OPERATOR_OPERATION_UNSUPPORTED"),
    }
}

fn bounded_response(node: &Node, value: Value) -> Response {
    if serde_json::to_vec(&value)
        .is_ok_and(|v| v.len() <= node.config.read().unwrap().limits.response_body_bytes)
    {
        Json(value).into_response()
    } else {
        (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(json!({"status":"REFUSED","reason":"API_RESPONSE_EXCEEDS_OPERATOR_CAP"})),
        )
            .into_response()
    }
}
