//! Independent Porch daemon, with effects confined to installed capabilities.
pub mod api;
pub mod clients;
pub mod db;
pub mod limits;
pub mod models;
pub mod network;
pub mod qualification;
pub mod storage;
mod wire_codec;

use anyhow::{Context, Result, ensure};
use db::Db;
use libp2p::identity::Keypair;
use models::ModelSpec;
use porch_core::*;
use rusqlite::{OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
    time::{Duration, Instant},
};
use tokio::sync::{Semaphore, broadcast};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: u32,
    pub alias: String,
    pub listen: Vec<String>,
    pub mdns: bool,
    pub api: String,
    pub ollama_url: String,
    pub models: Vec<ModelSpec>,
    pub compute_hash: bool,
    pub storage_quota: u64,
    #[serde(default)]
    pub limits: limits::LimitsConfig,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            version: VERSION,
            alias: "My Porch Node".into(),
            listen: vec![
                "/ip4/0.0.0.0/tcp/7332".into(),
                "/ip4/0.0.0.0/udp/7332/quic-v1".into(),
            ],
            mdns: true,
            api: "127.0.0.1:7331".into(),
            ollama_url: "http://127.0.0.1:11434".into(),
            models: vec![],
            compute_hash: false,
            storage_quota: 0,
            limits: limits::LimitsConfig::default(),
        }
    }
}
impl Config {
    /// Persisted operator sharing choices precede initial config.json. CLI
    /// overrides are applied by main after this load, and validated by open.
    pub fn load(root: &Path) -> Result<Self> {
        let db = root.join("porch.sqlite");
        if db.exists() {
            let conn = rusqlite::Connection::open_with_flags(
                db,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            )?;
            let saved: Option<String> = conn
                .query_row("SELECT value FROM meta WHERE key='config'", [], |r| {
                    r.get(0)
                })
                .optional()?;
            if let Some(saved) = saved {
                return Ok(serde_json::from_str(&saved)?);
            }
        }
        let file = root.join("config.json");
        if file.exists() {
            Ok(serde_json::from_slice(&std::fs::read(file)?)?)
        } else {
            Ok(Self::default())
        }
    }
}
pub struct GrantUse<'a> {
    pub peer: &'a str,
    pub cap: &'a str,
    pub resource: &'a str,
    pub action: &'a str,
    pub input: Option<&'a str>,
    pub consume: bool,
    pub bytes: u64,
    pub time: u64,
}
pub struct Node {
    pub id: String,
    pub key: Keypair,
    pub root: PathBuf,
    pub db: Db,
    pub config: RwLock<Config>,
    pub api_token: String,
    pub network: RwLock<Option<Arc<dyn network::Transport>>>,
    pub observations: RwLock<Value>,
    pub slots: Arc<Semaphore>,
    pub ingress: Arc<Semaphore>,
    pub storage_lock: Mutex<()>,
    pub events: broadcast::Sender<Value>,
    _state_lock: std::fs::File,
    pub started: Instant,
}
impl Node {
    pub fn open(root: &Path, config: Config) -> Result<Arc<Self>> {
        ensure!(
            config.version == VERSION
                && valid_label(&config.alias, 64)
                && config.models.len() + 3 <= config.limits.services
                && config.models.iter().all(|m| valid_label(&m.name, 128)
                    && matches!(m.provider.as_str(), "ollama" | "mock")),
            "INVALID_CONFIGURATION"
        );
        let addr: std::net::SocketAddr = config.api.parse()?;
        ensure!(addr.ip().is_loopback(), "CONTROL_API_MUST_BE_LOOPBACK");
        models::validate_ollama(&config.ollama_url)?;
        ensure!(
            config.storage_quota <= 1024 * 1024 * 1024 * 1024,
            "STORAGE_QUOTA_LIMIT"
        );
        config.limits.validate()?;
        ensure!(
            config.storage_quota <= config.limits.storage_total_bytes,
            "STORAGE_QUOTA_EXCEEDS_OPERATOR_CAP"
        );
        ensure!(
            !root.join("porch.sqlite").exists() || root.join("identity.key").exists(),
            "EXISTING_STATE_IDENTITY_MISSING_RESTORE_BACKUP"
        );
        std::fs::create_dir_all(root)?;
        let lock = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(root.join("node.lock"))?;
        lock.try_lock()
            .map_err(|_| anyhow::anyhow!("NODE_STATE_ALREADY_IN_USE"))?;
        let key = identity(root)?;
        let id = key.public().to_peer_id().to_string();
        let token_path = root.join("api.token");
        if !token_path.exists() {
            private_write(&token_path, hex::encode(random_bytes::<32>()).as_bytes())?;
        }
        let api_token = std::fs::read_to_string(token_path)?.trim().to_string();
        ensure!(
            api_token.len() == 64
                && api_token.bytes().all(|b| b.is_ascii_hexdigit())
                && api_token
                    .bytes()
                    .collect::<std::collections::HashSet<_>>()
                    .len()
                    >= 8,
            "INVALID_OPERATOR_TOKEN"
        );
        let db = Db::open(&root.join("porch.sqlite"), &id)?;
        let events = db.events.clone();
        let job_slots = config.limits.concurrent_jobs;
        let ingress_slots = config.limits.ingress_requests;
        let node = Arc::new(Self {
            id,
            key,
            root: root.into(),
            db,
            config: RwLock::new(config),
            api_token,
            network: RwLock::new(None),
            observations: RwLock::new(
                json!({"addresses":[],"peers":{},"outbound_job_requests":0,"dht":"disabled-private-default","relay":"disabled","nat_traversal":"not-field-qualified"}),
            ),
            slots: Arc::new(Semaphore::new(job_slots)),
            ingress: Arc::new(Semaphore::new(ingress_slots)),
            storage_lock: Mutex::new(()),
            events,
            _state_lock: lock,
            started: Instant::now(),
        });
        if let Err(e) = node.db.time() {
            node.event(
                "clock.anomaly",
                &node.id,
                json!({"reason":"CLOCK_UNCERTAIN_BACKWARD_JUMP"}),
            )?;
            return Err(e);
        }
        node.event(
            "node.started",
            &node.id,
            json!({"configuration_hash":digest(&canonical(&*node.config.read().unwrap())?)}),
        )?;
        storage::recover(&node)?;
        Ok(node)
    }
    pub fn event(&self, kind: &str, subject: &str, hashes: Value) -> Result<()> {
        self.db.ledger(&self.key, kind, subject, hashes)?;
        Ok(())
    }
    pub fn trust(
        &self,
        peer: &str,
        alias: &str,
        porch: Option<&str>,
        record: &Value,
    ) -> Result<()> {
        let _: libp2p::PeerId = peer.parse()?;
        ensure!(
            peer != self.id && valid_label(alias, 64),
            "INVALID_PEER_PROFILE"
        );
        self.db.transaction(|tx|{
            let existing:Option<String>=tx.query_row("SELECT alias FROM trust WHERE peer=?1",[peer],|r|r.get(0)).optional()?;
            if let Some(existing)=&existing {ensure!(existing==alias,"ALIAS_CHANGE_REQUIRES_OPERATOR");}
            let count:u64=tx.query_row("SELECT count(*) FROM trust",[],|r|r.get(0))?;ensure!(existing.is_some() || count<128,"PEER_LIMIT");
            tx.execute("INSERT INTO trust(peer,alias,porch,active,record) VALUES(?1,?2,?3,1,?4) ON CONFLICT(peer) DO UPDATE SET porch=excluded.porch,active=1,record=excluded.record",params![peer,alias,porch,serde_json::to_string(record)?])?;
            Db::append(tx,&self.key,"trust.established",peer,json!({"record_hash":digest(&canonical(record)?)}))?;Ok(())
        })
    }
    pub fn revoke_peer(&self, peer: &str) -> Result<Value> {
        self.db.transaction(|tx| {
            tx.execute("UPDATE trust SET active=0 WHERE peer=?1", [peer])?;
            tx.execute(
                "UPDATE grants SET revoked=1 WHERE recipient=?1 OR issuer=?1",
                [peer],
            )?;
            Db::append(tx, &self.key, "peer.revoked", peer, json!({}))?;
            Ok(())
        })?;
        if let Some(net) = self.network.read().unwrap().clone() {
            let _ = net.disconnect(peer);
        }
        Ok(json!({"revoked":peer}))
    }
    pub fn advertisement(&self) -> Result<Signed<Advertisement>> {
        let c = self.config.read().unwrap().clone();
        let time = self.db.time()?;
        let models=c.models.iter().filter(|m|m.exposed).map(|m|ModelManifest{model:m.name.clone(),provider:m.provider.clone(),version:m.version.clone(),claims:json!({"capability_evidence":"configured-provider","context_limit":null,"vision":null,"tools":null})}).collect::<Vec<_>>();
        let mut services = Vec::new();
        for m in &models {
            services.push(serde_json::to_value(Signed::new(&self.key,"porch.service.v1",json!({"service_id":digest(format!("{}:model:{}",self.id,m.model).as_bytes()),"provider_peer":self.id,"type":"model.inference","resource":m.model,"protocol_version":VERSION,"endpoint":format!("porch://peer/{}",self.id),"policy":"explicit-grant-required","expires_at":time+60}))?)?);
        }
        for (enabled, kind, resource) in [
            (c.compute_hash, "compute.hash", "sha256"),
            (c.storage_quota > 0, "blob.storage", "vault"),
            (true, "message.direct", "inbox"),
        ] {
            if enabled {
                services.push(serde_json::to_value(Signed::new(&self.key,"porch.service.v1",json!({"service_id":digest(format!("{}:{kind}:{resource}",self.id).as_bytes()),"provider_peer":self.id,"type":kind,"resource":resource,"protocol_version":VERSION,"endpoint":format!("porch://peer/{}",self.id),"policy":"explicit-grant-required","expires_at":time+60}))?)?);
            }
        }
        Signed::new(
            &self.key,
            "porch.advertisement.v1",
            Advertisement {
                peer: self.id.clone(),
                alias: c.alias,
                created_at: time,
                expires_at: time + 60,
                models,
                compute_hash: c.compute_hash,
                storage_quota: c.storage_quota,
                advertised: json!({"shareable_storage_bytes":c.storage_quota,"gpu_vram":null,"ram_bytes":null,"capacity_evidence":"operator-allocation-and-provider-claims"}),
                observed: json!({"local_cpu_parallelism":std::thread::available_parallelism().map(|n|n.get()).ok(),"active_jobs":c.limits.concurrent_jobs-self.slots.available_permits(),"queue_depth":0,"job_slots":c.limits.concurrent_jobs}),
                services,
            },
        )
    }
    pub fn issue_grant(&self, mut grant: Grant) -> Result<Signed<Grant>> {
        let time = self.db.time()?;
        ensure!(
            self.db.trusted(&grant.recipient)?,
            "UNKNOWN_OR_REVOKED_PEER"
        );
        grant.issuer = self.id.clone();
        grant.created_at = time;
        grant.validate(
            &self.id,
            &grant.recipient,
            &grant.capability,
            &grant.resource,
            &grant.action,
            time,
        )?;
        let c = self.config.read().unwrap().clone();
        let valid = match grant.capability.as_str() {
            "model.inference" => {
                grant.action == "run"
                    && c.models
                        .iter()
                        .any(|m| m.exposed && m.name == grant.resource)
            }
            "compute.hash" => grant.action == "run" && grant.resource == "sha256" && c.compute_hash,
            "blob.storage" => {
                grant.action == "store"
                    && grant.resource == "vault"
                    && c.storage_quota > 0
                    && grant.limits.max_storage_bytes <= c.storage_quota
            }
            "message.direct" => grant.action == "send" && grant.resource == "inbox",
            _ => false,
        };
        ensure!(valid, "CAPABILITY_NOT_SHARED");
        let signed = Signed::new(&self.key, "porch.grant.v1", grant)?;
        self.db.transaction(|tx| {
            let count: u64 = tx.query_row("SELECT count(*) FROM grants", [], |r| r.get(0))?;
            ensure!(count < c.limits.grants, "GRANT_TABLE_FULL");
            tx.execute(
                "INSERT INTO grants(nonce,issuer,recipient,record) VALUES(?1,?2,?3,?4)",
                params![
                    signed.payload.nonce,
                    signed.signer,
                    signed.payload.recipient,
                    serde_json::to_string(&signed)?
                ],
            )?;
            Db::append(
                tx,
                &self.key,
                "grant.issued",
                &signed.payload.recipient,
                json!({"grant_hash":signed.hash()?}),
            )?;
            Ok(())
        })?;
        Ok(signed)
    }
    pub fn import_grant(&self, grant: Signed<Grant>) -> Result<Value> {
        grant.verify("porch.grant.v1")?;
        grant.payload.validate(
            &grant.signer,
            &self.id,
            &grant.payload.capability,
            &grant.payload.resource,
            &grant.payload.action,
            self.db.time()?,
        )?;
        ensure!(
            grant.payload.issuer == grant.signer && self.db.trusted(&grant.signer)?,
            "UNTRUSTED_GRANT_ISSUER"
        );
        self.db.transaction(|tx| {
            let old: Option<String> = tx
                .query_row(
                    "SELECT record FROM grants WHERE nonce=?1",
                    [&grant.payload.nonce],
                    |r| r.get(0),
                )
                .optional()?;
            if let Some(old) = old {
                ensure!(
                    old == serde_json::to_string(&grant)?,
                    "GRANT_NONCE_CONFLICT"
                );
                return Ok(());
            }
            let count: u64 = tx.query_row("SELECT count(*) FROM grants", [], |r| r.get(0))?;
            ensure!(
                count < self.config.read().unwrap().limits.grants,
                "GRANT_TABLE_FULL"
            );
            tx.execute(
                "INSERT INTO grants(nonce,issuer,recipient,record) VALUES(?1,?2,?3,?4)",
                params![
                    grant.payload.nonce,
                    grant.signer,
                    grant.payload.recipient,
                    serde_json::to_string(&grant)?
                ],
            )?;
            Db::append(
                tx,
                &self.key,
                "grant.imported",
                &grant.signer,
                json!({"grant_hash":grant.hash()?}),
            )?;
            Ok(())
        })?;
        Ok(json!({"imported":grant.payload.nonce}))
    }
    pub fn revoke_grant(&self, nonce: &str) -> Result<Value> {
        self.db.transaction(|tx| {
            let changed = tx.execute("UPDATE grants SET revoked=1 WHERE nonce=?1", [nonce])?;
            ensure!(changed == 1, "GRANT_NOT_FOUND");
            Db::append(tx, &self.key, "grant.revoked", nonce, json!({}))?;
            Ok(())
        })?;
        Ok(json!({"revoked":nonce}))
    }
    pub fn check_grant(
        &self,
        tx: &Transaction<'_>,
        grant: &Signed<Grant>,
        check: GrantUse<'_>,
    ) -> Result<()> {
        let GrantUse {
            peer,
            cap,
            resource,
            action,
            input,
            consume,
            bytes,
            time,
        } = check;
        grant.verify("porch.grant.v1")?;
        ensure!(grant.signer == self.id, "WRONG_EXECUTOR");
        grant
            .payload
            .validate(&self.id, peer, cap, resource, action, time)?;
        let active: bool = tx
            .query_row("SELECT active FROM trust WHERE peer=?1", [peer], |r| {
                r.get(0)
            })
            .optional()?
            .unwrap_or(false);
        ensure!(active, "UNKNOWN_OR_REVOKED_PEER");
        if let Some(porch) = &grant.payload.porch {
            let member: Option<String> = tx
                .query_row("SELECT porch FROM trust WHERE peer=?1", [peer], |r| {
                    r.get(0)
                })
                .optional()?
                .flatten();
            ensure!(member.as_ref() == Some(porch), "GRANT_PORCH_SCOPE_MISMATCH");
            let current: Option<String> = tx
                .query_row("SELECT value FROM meta WHERE key='porch'", [], |r| r.get(0))
                .optional()?;
            let current: Value = current
                .map(|v| serde_json::from_str(&v))
                .transpose()?
                .unwrap_or(Value::Null);
            ensure!(current["id"].as_str() == Some(porch), "PORCH_NOT_ACTIVE");
        }
        if let Some(exact) = &grant.payload.exact_input_digest {
            ensure!(input == Some(exact.as_str()), "EXACT_INPUT_MISMATCH");
        }
        let (record, revoked, calls, window, hour_calls, used_bytes): (
            String,
            bool,
            u64,
            u64,
            u64,
            u64,
        ) = tx
            .query_row(
                "SELECT record,revoked,calls,window,hour_calls,bytes FROM grants WHERE nonce=?1",
                [&grant.payload.nonce],
                |r| {
                    Ok((
                        r.get(0)?,
                        r.get(1)?,
                        r.get(2)?,
                        r.get(3)?,
                        r.get(4)?,
                        r.get(5)?,
                    ))
                },
            )
            .context("GRANT_NOT_ISSUED")?;
        ensure!(
            !revoked && record == serde_json::to_string(grant)?,
            "GRANT_REVOKED_OR_CHANGED"
        );
        if consume {
            let (window, hour_calls) = if time >= window.saturating_add(3600) {
                (time, 0)
            } else {
                (window, hour_calls)
            };
            ensure!(
                calls < grant.payload.limits.max_calls
                    && hour_calls < grant.payload.limits.calls_per_hour,
                "GRANT_QUOTA_EXHAUSTED"
            );
            ensure!(
                used_bytes
                    .checked_add(bytes)
                    .is_some_and(|b| b <= grant.payload.limits.max_storage_bytes),
                "GRANT_STORAGE_QUOTA_EXHAUSTED"
            );
            tx.execute("UPDATE grants SET calls=calls+1,window=?2,hour_calls=?3,bytes=bytes+?4 WHERE nonce=?1",params![grant.payload.nonce,window,hour_calls+1,bytes])?;
        }
        Ok(())
    }
    pub fn remote_grant(
        &self,
        peer: &str,
        cap: &str,
        resource: &str,
        action: &str,
    ) -> Result<Option<Signed<Grant>>> {
        let time = self.db.time()?;
        for row in self.db.grants()? {
            if row["revoked"] == true {
                continue;
            }
            let g: Signed<Grant> = serde_json::from_value(row["grant"].clone())?;
            if g.signer == peer
                && g.payload
                    .validate(peer, &self.id, cap, resource, action, time)
                    .is_ok()
            {
                return Ok(Some(g));
            }
        }
        Ok(None)
    }
    pub async fn rpc(
        &self,
        peer: &str,
        operation: &str,
        args: Value,
    ) -> Result<Signed<WireResponse>> {
        if operation == "job.run" {
            let job: Job = serde_json::from_value(args.clone())?;
            ensure!(
                job.privacy != Privacy::LocalOnly,
                "LOCAL_ONLY_INPUT_CANNOT_LEAVE_ORIGIN"
            );
            ensure!(
                job.requester == self.id,
                "LOCAL_REQUESTER_IDENTITY_MISMATCH"
            );
        }
        let time = self.db.time()?;
        let request = Signed::new(
            &self.key,
            "porch.request.v1",
            WireRequest {
                id: nonce(),
                nonce: nonce(),
                created_at: time,
                expires_at: time + REQUEST_TTL,
                operation: operation.into(),
                args,
            },
        )?;
        ensure!(
            canonical(&request)?.len() <= self.config.read().unwrap().limits.peer_frame_bytes,
            "REQUEST_TOO_LARGE"
        );
        let hash = request.hash()?;
        let req_nonce = request.payload.nonce.clone();
        let net = self
            .network
            .read()
            .unwrap()
            .clone()
            .context("NETWORK_NOT_STARTED")?;
        ensure!(
            canonical(&request)?.len() <= net.frame_limit(),
            "TRANSPORT_FRAME_LIMIT"
        );
        if operation == "job.run" {
            let mut o = self.observations.write().unwrap();
            let n = o["outbound_job_requests"].as_u64().unwrap_or(0);
            o["outbound_job_requests"] = json!(n + 1);
        }
        let response = if operation == "job.run" {
            let deadline = request.payload.args["timeout_ms"]
                .as_u64()
                .unwrap_or(30000)
                .min(30000)
                + 2000;
            match tokio::time::timeout(Duration::from_millis(deadline), net.request(peer, request))
                .await
            {
                Ok(Ok(response)) => response,
                Ok(Err(e)) => return Err(e),
                Err(_) => {
                    let _ = net.disconnect(peer);
                    anyhow::bail!("PEER_JOB_DEADLINE");
                }
            }
        } else {
            net.request(peer, request).await?
        };
        response
            .verify("porch.response.v1")
            .context("RESPONSE_SIGNATURE_INVALID")?;
        ensure!(
            response.signer == peer
                && response.payload.request_nonce == req_nonce
                && response.payload.request_digest == hash,
            "RESPONSE_BINDING_MISMATCH"
        );
        Ok(response)
    }
    pub async fn receive(
        self: &Arc<Self>,
        peer: String,
        request: Signed<WireRequest>,
    ) -> Result<Signed<WireResponse>> {
        let hash = request.hash()?;
        let result = async {
            ensure!(
                canonical(&request)?.len() <= self.config.read().unwrap().limits.peer_frame_bytes,
                "REQUEST_TOO_LARGE"
            );
            request.verify("porch.request.v1")?;
            ensure!(request.signer == peer, "TRANSPORT_IDENTITY_MISMATCH");
            let t = self.db.time()?;
            let r = &request.payload;
            ensure!(
                r.created_at <= t + 2
                    && t < r.expires_at
                    && r.expires_at <= r.created_at + REQUEST_TTL,
                "REQUEST_EXPIRED_OR_FUTURE"
            );
            self.db.replay(&peer, &r.nonce, r.expires_at)?;
            if r.operation != "porch.join" {
                ensure!(self.db.trusted(&peer)?, "UNKNOWN_OR_REVOKED_PEER");
            }
            match r.operation.as_str() {
                "resources" => Ok(serde_json::to_value(self.advertisement()?)?),
                "porch.join" => self.accept_join(&peer, &r.args),
                "job.run" => {
                    let job: Job = serde_json::from_value(r.args.clone())?;
                    ensure!(job.requester == peer, "JOB_REQUESTER_TRANSPORT_MISMATCH");
                    Ok(serde_json::to_value(self.execute_job(job, false).await?)?)
                }
                "blob.begin" | "blob.chunk" | "blob.get" | "blob.delete" => {
                    storage::receive(self, &peer, &r.operation, &r.args)
                }
                "message.send" => self.accept_message(&peer, &r.args),
                _ => anyhow::bail!("PROTOCOL_OPERATION_UNSUPPORTED"),
            }
        }
        .await;
        let (status, code, output) = match result {
            Ok(v) => ("OK", "OK".to_string(), v),
            Err(e) => {
                self.event(
                    "request.refused",
                    &peer,
                    json!({"request_digest":hash,"reason":safe_error(&e)}),
                )?;
                ("REFUSED", safe_error(&e), Value::Null)
            }
        };
        Signed::new(
            &self.key,
            "porch.response.v1",
            WireResponse {
                request_nonce: request.payload.nonce,
                request_digest: hash,
                status: status.into(),
                code,
                output,
                timestamp: now(),
            },
        )
    }
    pub async fn execute_job(self: &Arc<Self>, job: Job, local: bool) -> Result<Signed<Receipt>> {
        let time = self.db.time()?;
        let started = Instant::now();
        let request_hash = digest(&canonical(&job)?);
        let prior: Option<(String, String, Option<String>)> = self.db.transaction(|tx| {
            Ok(tx
                .query_row(
                    "SELECT hash,status,receipt FROM jobs WHERE requester=?1 AND id=?2",
                    params![job.requester, job.id],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )
                .optional()?)
        })?;
        if let Some((hash, status, receipt)) = prior {
            ensure!(hash == request_hash, "JOB_ID_CONFLICT");
            if let Some(receipt) = receipt {
                let receipt: Signed<Receipt> = serde_json::from_str(&receipt)?;
                receipt.verify("porch.job.receipt.v1")?;
                ensure!(
                    receipt.signer == self.id && receipt.payload.request_digest == request_hash,
                    "CACHED_RECEIPT_BINDING_MISMATCH"
                );
                return Ok(receipt);
            }
            anyhow::bail!("DUPLICATE_JOB_{status}");
        }
        let validation = (|| -> Result<_> {
            job.validate(if local { &self.id } else { &job.requester })?;
            if !local {
                ensure!(
                    job.privacy != Privacy::LocalOnly,
                    "LOCAL_ONLY_CANNOT_EXECUTE_REMOTELY"
                );
                ensure!(job.execution_mode == "REMOTE_NODE", "REMOTE_MODE_REQUIRED");
            }
            if let Some(preferred) = &job.preferred_peer {
                ensure!(preferred == &self.id, "WRONG_EXECUTOR");
            }
            ensure!(
                job.input.len() <= self.config.read().unwrap().limits.input_bytes
                    && job.timeout_ms <= self.config.read().unwrap().limits.model_timeout_ms,
                "JOB_EXCEEDS_OPERATOR_CAP"
            );
            let permit = self
                .slots
                .clone()
                .try_acquire_owned()
                .context("EXECUTOR_BUSY")?;
            let c = self.config.read().unwrap().clone();
            let model = c
                .models
                .iter()
                .find(|m| m.name == job.resource && (local || m.exposed))
                .cloned();
            match job.capability.as_str() {
                "model.inference" => ensure!(model.is_some(), "MODEL_UNAVAILABLE"),
                "compute.hash" => ensure!(
                    job.resource == "sha256" && (local || c.compute_hash),
                    "COMPUTE_NOT_SHARED"
                ),
                _ => anyhow::bail!("CAPABILITY_NOT_INSTALLED"),
            };
            self.db.transaction(|tx| {
                if !local {
                    let grant = job.grant.as_ref().context("AUTHORITY_REQUIRED")?;
                    ensure!(
                        job.input.len() as u64 <= grant.payload.limits.max_input_bytes
                            && job.max_output_tokens <= grant.payload.limits.max_output_tokens
                            && job.timeout_ms <= grant.payload.limits.max_duration_ms,
                        "JOB_EXCEEDS_GRANT_LIMITS"
                    );
                    self.check_grant(
                        tx,
                        grant,
                        crate::GrantUse {
                            peer: &job.requester,
                            cap: &job.capability,
                            resource: &job.resource,
                            action: "run",
                            input: Some(&job.input_digest),
                            consume: true,
                            bytes: 0,
                            time,
                        },
                    )?;
                }
                tx.execute(
                    "INSERT INTO jobs(requester,id,hash,status) VALUES(?1,?2,?3,'RUNNING')",
                    params![job.requester, job.id, request_hash],
                )?;
                Db::append(
                    tx,
                    &self.key,
                    "job.accepted",
                    &job.id,
                    json!({"input_digest":job.input_digest,"request_digest":request_hash}),
                )?;
                Ok(())
            })?;
            Ok((permit, model, c.ollama_url))
        })();
        let admitted = validation.is_ok();
        let outcome = match validation {
            Ok((_permit, model, url)) => {
                let effect = models::execute(&job, model.as_ref(), &url);
                tokio::pin!(effect);
                let mut tick = tokio::time::interval(Duration::from_millis(100));
                let mut effect_result;
                loop {
                    tokio::select! {
                        r=&mut effect=>{effect_result=r;break;},
                        _=tick.tick()=>{
                            if started.elapsed().as_millis()>=job.timeout_ms as u128 {effect_result=Err(anyhow::anyhow!("JOB_TIMEOUT"));break;}
                            if !local {
                                let allowed=self.db.time().and_then(|t|self.db.transaction(|tx|self.check_grant(tx,job.grant.as_ref().unwrap(),crate::GrantUse{peer:&job.requester,cap:&job.capability,resource:&job.resource,action:"run",input:Some(&job.input_digest),consume:false,bytes:0,time:t})));
                                if let Err(e)=allowed {effect_result=Err(e);break;}
                            }
                        }
                    }
                }
                // Revalidate before releasing provider output to a remote requester.
                if !local
                    && effect_result.is_ok()
                    && let Err(e) = self.db.time().and_then(|t| {
                        self.db.transaction(|tx| {
                            self.check_grant(
                                tx,
                                job.grant.as_ref().unwrap(),
                                crate::GrantUse {
                                    peer: &job.requester,
                                    cap: &job.capability,
                                    resource: &job.resource,
                                    action: "run",
                                    input: Some(&job.input_digest),
                                    consume: false,
                                    bytes: 0,
                                    time: t,
                                },
                            )
                        })
                    })
                {
                    effect_result = Err(e);
                }
                if let Some(m) = model.as_ref() {
                    models::record_execution(self, m, &effect_result)?;
                }
                effect_result
            }
            Err(e) => Err(e),
        };
        let (status, reason, output, provider, model_version, measured) = match outcome {
            Ok((o, p, v, m)) => ("COMPLETED", None, Some(o), p, v, m),
            Err(e) => (
                "REFUSED",
                Some(safe_error(&e)),
                None,
                "none".into(),
                None,
                json!({}),
            ),
        };
        let receipt = Signed::new(
            &self.key,
            "porch.job.receipt.v1",
            Receipt {
                version: VERSION,
                job_id: job.id.clone(),
                requester: job.requester.clone(),
                executor: self.id.clone(),
                capability: job.capability.clone(),
                resource: job.resource.clone(),
                provider,
                model_version,
                input_digest: job.input_digest,
                output_digest: digest(&canonical(&output)?),
                request_digest: request_hash.clone(),
                status: status.into(),
                reason,
                output,
                started_at: time,
                finished_at: now(),
                duration_ms: started.elapsed().as_millis() as u64,
                grant_nonce: job.grant.map(|g| g.payload.nonce),
                measured,
            },
        )?;
        self.db.transaction(|tx|{
            if !admitted {
                let prior:Option<(String,String,Option<String>)>=tx.query_row("SELECT hash,status,receipt FROM jobs WHERE requester=?1 AND id=?2",params![job.requester,job.id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional()?;
                if let Some((hash,status,prior))=prior {ensure!(hash==request_hash,"JOB_ID_CONFLICT");if let Some(prior)=prior{let original:Signed<Receipt>=serde_json::from_str(&prior)?;original.verify("porch.job.receipt.v1")?;return Ok(original);}anyhow::bail!("DUPLICATE_JOB_{status}");}
            }
            tx.execute("INSERT INTO jobs(requester,id,hash,status,receipt) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(requester,id) DO UPDATE SET status=excluded.status,receipt=excluded.receipt",params![job.requester,job.id,request_hash,status,serde_json::to_string(&receipt)?])?;
            Db::append(tx,&self.key,if status=="COMPLETED"{"job.completed"}else{"job.refused"},&job.id,json!({"receipt_hash":receipt.hash()?}))?;Ok(receipt)
        })
    }
    pub async fn refresh(self: &Arc<Self>) -> Result<Value> {
        let peers = self
            .db
            .trust_rows()?
            .into_iter()
            .filter(|r| r["active"] == true)
            .map(|r| r["peer"].as_str().unwrap_or("").to_string())
            .collect::<Vec<_>>();
        use futures::StreamExt;
        let results=futures::stream::iter(peers).map(|peer|{let node=self.clone();async move {
            let result=tokio::time::timeout(Duration::from_secs(3),node.rpc(&peer,"resources",json!({}))).await;
            let outcome=(||->Result<()> {let r=result.context("PEER_TIMEOUT")??;ensure!(r.payload.status=="OK","{}",r.payload.code);
                let ad:Signed<Advertisement>=serde_json::from_value(r.payload.output)?;ad.verify("porch.advertisement.v1").context("ADVERTISEMENT_SIGNATURE_INVALID")?;
                ensure!(ad.signer==peer && ad.payload.peer==peer && ad.payload.created_at<=now()+2 && ad.payload.expires_at>now() && ad.payload.expires_at<=now()+REQUEST_TTL+2
                    && valid_label(&ad.payload.alias,64) && ad.payload.models.len() <= 125 && ad.payload.services.len() <= node.config.read().unwrap().limits.services
                    && ad.payload.models.iter().all(|m|valid_label(&m.model,128)),"INVALID_ADVERTISEMENT");
                ensure!(node.db.trusted(&peer)?,"PEER_REVOKED_DURING_REFRESH");
                node.db.transaction(|tx|{tx.execute("INSERT INTO advertisements(peer,record,expires) VALUES(?1,?2,?3) ON CONFLICT(peer) DO UPDATE SET record=excluded.record,expires=excluded.expires",params![peer,serde_json::to_string(&ad)?,ad.payload.expires_at])?;Ok(())})?;Ok(())})();
            json!({"peer":peer,"updated":outcome.is_ok(),"reason":outcome.err().map(|e|e.to_string())})
        }}).buffer_unordered(8).collect::<Vec<_>>().await;
        Ok(json!(results))
    }
    pub async fn run(self: &Arc<Self>, args: &Value) -> Result<Value> {
        let input = args["input"]
            .as_str()
            .context("INPUT_REQUIRED")?
            .to_string();
        ensure!(
            input.len() <= self.config.read().unwrap().limits.input_bytes,
            "INPUT_TOO_LARGE"
        );
        let cap = args["capability"]
            .as_str()
            .unwrap_or("model.inference")
            .to_string();
        let resource = args["resource"]
            .as_str()
            .context("RESOURCE_REQUIRED")?
            .to_string();
        let privacy: Privacy =
            serde_json::from_value(args.get("privacy").cloned().unwrap_or(json!("LOCAL_ONLY")))?;
        let preferred = args["preferred_peer"].as_str().map(str::to_string);
        let request_id = args["id"]
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(nonce);
        ensure!(
            !request_id.is_empty() && request_id.len() <= 128,
            "INVALID_JOB_ID"
        );
        let max_output_tokens = u32::try_from(args["max_output_tokens"].as_u64().unwrap_or(512))
            .context("OUTPUT_LIMIT_INVALID")?;
        let timeout_cap = self.config.read().unwrap().limits.model_timeout_ms;
        let timeout_ms = args["timeout_ms"].as_u64().unwrap_or(timeout_cap);
        ensure!(
            timeout_ms > 0
                && timeout_ms <= timeout_cap
                && max_output_tokens > 0
                && max_output_tokens <= 4096,
            "JOB_LIMITS_INVALID"
        );
        let intent_hash = digest(&canonical(
            &json!({"requester":self.id,"capability":cap,"resource":resource,"input_digest":digest(input.as_bytes()),"privacy":privacy,"preferred_peer":preferred,"max_output_tokens":max_output_tokens,"timeout_ms":timeout_ms}),
        )?);
        let previous = self.db.transaction(|tx| {
            Ok(tx
                .query_row(
                    "SELECT intent_hash,job,route,result FROM outbound_jobs WHERE id=?1",
                    [&request_id],
                    |r| {
                        Ok((
                            r.get::<_, String>(0)?,
                            r.get::<_, String>(1)?,
                            r.get::<_, String>(2)?,
                            r.get::<_, Option<String>>(3)?,
                        ))
                    },
                )
                .optional()?)
        })?;
        if let Some((intent, job, route, result)) = previous {
            ensure!(intent == intent_hash, "JOB_ID_CONFLICT");
            if let Some(result) = result {
                let value: Value = serde_json::from_str(&result)?;
                let receipt: Signed<Receipt> = serde_json::from_value(value["receipt"].clone())?;
                receipt.verify("porch.job.receipt.v1")?;
                return Ok(value);
            }
            return self
                .dispatch_remote(serde_json::from_str(&job)?, serde_json::from_str(&route)?)
                .await;
        }
        if privacy != Privacy::LocalOnly {
            self.refresh().await?;
        }
        let c = self.config.read().unwrap().clone();
        let observations = self.observations.read().unwrap().clone();
        let time = self.db.time()?;
        let matches_local = if cap == "compute.hash" {
            resource == "sha256"
        } else {
            c.models.iter().any(|m| m.name == resource)
        };
        let mut candidates = vec![Candidate {
            peer: self.id.clone(),
            local: true,
            trusted: true,
            same_porch: true,
            model_available: matches_local,
            authorized: true,
            fresh: true,
            has_capacity: self.slots.available_permits() > 0,
            queue_depth: 0,
            latency_ms: 0,
        }];
        let trust = self.db.trust_rows()?;
        let current = self.db.get("porch")?.unwrap_or(Value::Null);
        for value in self.db.records("advertisements")? {
            let ad: Signed<Advertisement> = serde_json::from_value(value)?;
            ad.verify("porch.advertisement.v1")?;
            let g = self.remote_grant(&ad.signer, &cap, &resource, "run")?;
            let row = trust
                .iter()
                .find(|r| r["peer"].as_str() == Some(&ad.signer));
            let available = if cap == "compute.hash" {
                ad.payload.compute_hash && resource == "sha256"
            } else {
                ad.payload.models.iter().any(|m| m.model == resource)
            };
            let authority = g.as_ref().is_some_and(|g| {
                g.payload.limits.max_input_bytes >= input.len() as u64
                    && g.payload.limits.max_output_tokens >= max_output_tokens
                    && g.payload.limits.max_duration_ms >= timeout_ms
                    && g.payload
                        .porch
                        .as_ref()
                        .is_none_or(|scope| current["id"].as_str() == Some(scope))
                    && g.payload
                        .exact_input_digest
                        .as_ref()
                        .is_none_or(|v| v == &digest(input.as_bytes()))
            });
            candidates.push(Candidate {
                peer: ad.signer.clone(),
                local: false,
                trusted: row.is_some_and(|r| r["active"] == true),
                same_porch: row
                    .is_some_and(|r| r["porch"] == current["id"] && !current["id"].is_null()),
                model_available: available,
                authorized: authority,
                fresh: ad.payload.expires_at > time,
                has_capacity: ad.payload.observed["active_jobs"]
                    .as_u64()
                    .unwrap_or(u64::MAX)
                    < ad.payload.observed["job_slots"].as_u64().unwrap_or(2),
                queue_depth: ad.payload.observed["queue_depth"].as_u64().unwrap_or(0) as u32,
                latency_ms: observations["peers"][&ad.signer]["latency_ms"]
                    .as_u64()
                    .unwrap_or(10000),
            });
        }
        if let Some(peer) = &preferred {
            candidates.retain(|c| &c.peer == peer);
        }
        let mut route = schedule(&privacy, &candidates);
        route["capability"] = json!(cap);
        route["resource"] = json!(resource);
        route["privacy"] = json!(privacy);
        route["federation"] = json!("NOT_INSTALLED");
        if privacy == Privacy::FederatedAllowed {
            route["effective_remote_scope"] = json!("TRUSTED_PEERS");
        }
        route["candidates"]=json!(candidates.iter().map(|c|json!({"peer":c.peer,"local":c.local,"trusted":c.trusted,"same_porch":c.same_porch,"capability_available":c.model_available,"authorized":c.authorized,"fresh":c.fresh,"has_capacity":c.has_capacity,"queue_depth":c.queue_depth,"observed_latency_ms":if c.latency_ms==10000{None}else{Some(c.latency_ms)}})).collect::<Vec<_>>());
        self.db.set("latest_route", &route)?;
        self.event(
            "route.decided",
            &resource,
            json!({"decision_hash":digest(&canonical(&route)?)}),
        )?;
        let Some(chosen) = route["chosen"].as_str() else {
            return Ok(json!({"status":"REFUSED","reason":"NO_ELIGIBLE_EXECUTOR","route":route}));
        };
        let local = chosen == self.id;
        let grant = if local {
            None
        } else {
            self.remote_grant(chosen, &cap, &resource, "run")?
        };
        let job = Job {
            version: VERSION,
            id: request_id,
            requester: self.id.clone(),
            capability: cap,
            resource,
            input_digest: digest(input.as_bytes()),
            input,
            max_output_tokens,
            timeout_ms,
            privacy,
            execution_mode: if local { "SINGLE_NODE" } else { "REMOTE_NODE" }.into(),
            preferred_peer: Some(chosen.into()),
            grant,
        };
        if local {
            let receipt = self.execute_job(job, true).await?;
            return Ok(json!({"status":receipt.payload.status,"route":route,"receipt":receipt}));
        }
        let (job, route) = self.db.transaction(|tx| {
            let existing: Option<(String, String, String)> = tx
                .query_row(
                    "SELECT intent_hash,job,route FROM outbound_jobs WHERE id=?1",
                    [&job.id],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )
                .optional()?;
            if let Some((intent, job, route)) = existing {
                ensure!(intent == intent_hash, "JOB_ID_CONFLICT");
                return Ok((serde_json::from_str(&job)?, serde_json::from_str(&route)?));
            }
            let local_hash: Option<String> = tx
                .query_row(
                    "SELECT hash FROM jobs WHERE requester=?1 AND id=?2",
                    params![self.id, job.id],
                    |r| r.get(0),
                )
                .optional()?;
            ensure!(local_hash.is_none(), "JOB_ID_ALREADY_USED_LOCALLY");
            let pending: u64 = tx.query_row(
                "SELECT count(*) FROM outbound_jobs WHERE result IS NULL",
                [],
                |r| r.get(0),
            )?;
            ensure!(
                pending < self.config.read().unwrap().limits.queued_jobs as u64,
                "OUTBOUND_QUEUE_FULL"
            );
            tx.execute(
                "INSERT INTO outbound_jobs(id,intent_hash,job,route) VALUES(?1,?2,?3,?4)",
                params![
                    job.id,
                    intent_hash,
                    serde_json::to_string(&job)?,
                    serde_json::to_string(&route)?
                ],
            )?;
            Db::append(
                tx,
                &self.key,
                "job.submitted",
                &job.id,
                json!({"manifest_digest":digest(&canonical(&job)?),"intent_hash":intent_hash}),
            )?;
            Ok((job, route))
        })?;
        self.dispatch_remote(job, route).await
    }
    async fn dispatch_remote(self: &Arc<Self>, job: Job, route: Value) -> Result<Value> {
        let chosen = job
            .preferred_peer
            .as_deref()
            .context("PINNED_EXECUTOR_REQUIRED")?;
        ensure!(self.db.trusted(chosen)?, "EXECUTOR_TRUST_REVOKED");
        let response = self
            .rpc(chosen, "job.run", serde_json::to_value(&job)?)
            .await?;
        ensure!(response.payload.status == "OK", "{}", response.payload.code);
        let receipt: Signed<Receipt> = serde_json::from_value(response.payload.output)?;
        receipt.verify("porch.job.receipt.v1")?;
        ensure!(
            receipt.signer == chosen
                && receipt.payload.executor == chosen
                && receipt.payload.requester == self.id
                && receipt.payload.request_digest == digest(&canonical(&job)?)
                && receipt.payload.output_digest == digest(&canonical(&receipt.payload.output)?),
            "RECEIPT_BINDING_MISMATCH"
        );
        let result = json!({"status":receipt.payload.status,"route":route,"receipt":receipt});
        self.db.transaction(|tx|{tx.execute("UPDATE outbound_jobs SET result=?2 WHERE id=?1",params![job.id,serde_json::to_string(&result)?])?;tx.execute("INSERT INTO jobs(requester,id,hash,status,receipt) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(requester,id) DO UPDATE SET status=excluded.status,receipt=excluded.receipt",params![self.id,job.id,digest(&canonical(&job)?),receipt.payload.status,serde_json::to_string(&receipt)?])?;Db::append(tx,&self.key,"job.remote.received",&job.id,json!({"receipt_hash":receipt.hash()?}))?;Ok(())})?;
        Ok(result)
    }
    pub fn create_porch(&self, name: &str) -> Result<Value> {
        ensure!(valid_label(name, 64), "INVALID_PORCH_NAME");
        let manifest =
            json!({"issuer":self.id,"name":name,"nonce":nonce(),"created_at":self.db.time()?});
        let id = digest(&canonical(&manifest)?);
        let signed = Signed::new(&self.key, "porch.community.v1", manifest)?;
        let record = json!({"id":id,"name":name,"issuer":self.id,"manifest":signed,"role":"owner"});
        self.db.set(&format!("porch:{id}"), &record)?;
        self.db.set("porch", &record)?;
        self.event(
            "porch.created",
            &id,
            json!({"manifest_hash":signed.hash()?}),
        )?;
        Ok(record)
    }
    pub fn invite(&self, args: &Value) -> Result<Signed<Invite>> {
        let porch = self.db.get("porch")?.context("CREATE_PORCH_FIRST")?;
        ensure!(porch["issuer"] == self.id, "ONLY_PORCH_OWNER_CAN_INVITE");
        let time = self.db.time()?;
        let ttl = args["ttl_seconds"].as_u64().unwrap_or(600);
        ensure!(ttl > 0 && ttl <= 3600, "INVALID_INVITE_TTL");
        let addresses = self.observations.read().unwrap()["addresses"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|a| a.as_str().map(str::to_string))
            .take(16)
            .collect();
        let invite = Signed::new(
            &self.key,
            "porch.invite.v1",
            Invite {
                porch: porch["id"].as_str().context("INVALID_PORCH")?.into(),
                name: porch["name"].as_str().unwrap_or("").into(),
                issuer: self.id.clone(),
                addresses,
                nonce: nonce(),
                recipient: args["recipient"].as_str().map(str::to_string),
                created_at: time,
                expires_at: time + ttl,
            },
        )?;
        self.db.transaction(|tx| {
            tx.execute(
                "INSERT INTO invites(nonce,hash,expires) VALUES(?1,?2,?3)",
                params![
                    invite.payload.nonce,
                    invite.hash()?,
                    invite.payload.expires_at
                ],
            )?;
            Db::append(
                tx,
                &self.key,
                "invite.created",
                &invite.payload.porch,
                json!({"invite_hash":invite.hash()?}),
            )?;
            Ok(())
        })?;
        Ok(invite)
    }
    fn validate_invite(&self, invite: &Signed<Invite>) -> Result<()> {
        invite.verify("porch.invite.v1")?;
        let t = self.db.time()?;
        ensure!(
            invite.signer == invite.payload.issuer
                && invite.payload.created_at <= t + 2
                && t < invite.payload.expires_at
                && invite.payload.expires_at <= invite.payload.created_at + 3600
                && invite.payload.addresses.len() <= 16
                && valid_label(&invite.payload.name, 64),
            "INVITE_EXPIRED_OR_INVALID"
        );
        Ok(())
    }
    fn accept_join(&self, peer: &str, args: &Value) -> Result<Value> {
        let invite: Signed<Invite> = serde_json::from_value(args["invite"].clone())?;
        self.validate_invite(&invite)?;
        ensure!(
            invite.signer == self.id && invite.payload.recipient.as_ref().is_none_or(|p| p == peer),
            "INVITE_RECIPIENT_MISMATCH"
        );
        let alias = args["alias"].as_str().context("ALIAS_REQUIRED")?;
        ensure!(valid_label(alias, 64), "INVALID_ALIAS");
        let current = self.db.get("porch")?.context("PORCH_NOT_ACTIVE")?;
        ensure!(current["id"] == invite.payload.porch, "PORCH_NOT_ACTIVE");
        let membership = Signed::new(
            &self.key,
            "porch.membership.v1",
            json!({"porch":invite.payload.porch,"peer":peer,"alias":alias,"invite_hash":invite.hash()?,"joined_at":self.db.time()?,"effect_authority":false}),
        )?;
        self.db.transaction(|tx| {
            let (hash, used): (String, bool) = tx
                .query_row(
                    "SELECT hash,used FROM invites WHERE nonce=?1",
                    [&invite.payload.nonce],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .context("INVITE_NOT_ISSUED")?;
            ensure!(
                hash == invite.hash()? && !used,
                "INVITE_ALREADY_USED_OR_CHANGED"
            );
            let count: u64 = tx.query_row("SELECT count(*) FROM trust", [], |r| r.get(0))?;
            ensure!(count < 128, "PEER_LIMIT");
            tx.execute(
                "INSERT INTO trust(peer,alias,porch,active,record) VALUES(?1,?2,?3,1,?4)",
                params![
                    peer,
                    alias,
                    invite.payload.porch,
                    serde_json::to_string(&membership)?
                ],
            )?;
            tx.execute(
                "UPDATE invites SET used=1 WHERE nonce=?1",
                [&invite.payload.nonce],
            )?;
            Db::append(
                tx,
                &self.key,
                "invite.accepted",
                peer,
                json!({"membership_hash":membership.hash()?}),
            )?;
            Ok(())
        })?;
        Ok(
            json!({"membership":membership,"porch":current,"owner_alias":self.config.read().unwrap().alias}),
        )
    }
    pub async fn join(self: &Arc<Self>, invite: Signed<Invite>) -> Result<Value> {
        self.validate_invite(&invite)?;
        if let Some(peer) = &invite.payload.recipient {
            ensure!(peer == &self.id, "INVITE_RECIPIENT_MISMATCH");
        }
        let net = self
            .network
            .read()
            .unwrap()
            .clone()
            .context("NETWORK_NOT_STARTED")?;
        net.connect_many(&invite.payload.addresses, Some(&invite.signer))?;
        let alias = self.config.read().unwrap().alias.clone();
        let response = self
            .rpc(
                &invite.signer,
                "porch.join",
                json!({"invite":invite,"alias":alias}),
            )
            .await?;
        ensure!(response.payload.status == "OK", "{}", response.payload.code);
        let m: Signed<Value> =
            serde_json::from_value(response.payload.output["membership"].clone())?;
        m.verify("porch.membership.v1")?;
        ensure!(
            m.signer == invite.signer
                && m.payload["peer"] == self.id
                && m.payload["porch"] == invite.payload.porch,
            "MEMBERSHIP_BINDING_MISMATCH"
        );
        self.trust(
            &invite.signer,
            response.payload.output["owner_alias"]
                .as_str()
                .context("OWNER_ALIAS_REQUIRED")?,
            Some(&invite.payload.porch),
            &serde_json::to_value(&m)?,
        )?;
        self.db.set(
            &format!("peer_addresses:{}", invite.signer),
            &json!(invite.payload.addresses),
        )?;
        let mut porch = response.payload.output["porch"].clone();
        porch["role"] = json!("member");
        self.db.set("porch", &porch)?;
        self.event(
            "porch.joined",
            &invite.payload.porch,
            json!({"membership_hash":m.hash()?}),
        )?;
        Ok(porch)
    }
    fn accept_message(&self, peer: &str, args: &Value) -> Result<Value> {
        let grant: Signed<Grant> = serde_json::from_value(args["grant"].clone())?;
        let id = args["id"].as_str().context("MESSAGE_ID_REQUIRED")?;
        ensure!(id.len() <= 128, "MESSAGE_ID_TOO_LONG");
        let text = args["text"].as_str().context("MESSAGE_TEXT_REQUIRED")?;
        ensure!(
            text.len() <= self.config.read().unwrap().limits.message_bytes
                && text.len() as u64 <= grant.payload.limits.max_input_bytes,
            "MESSAGE_TOO_LARGE"
        );
        let channel = args["channel"].as_str();
        let time = self.db.time()?;
        let input_hash = digest(text.as_bytes());
        let hash = digest(&canonical(&json!({"text":text,"channel":channel}))?);
        let receipt = Signed::new(
            &self.key,
            "porch.message.receipt.v1",
            json!({"id":id,"sender":peer,"recipient":self.id,"message_digest":hash,"received_at":time}),
        )?;
        self.db.transaction(|tx|{
            if let Some(old)=tx.query_row("SELECT hash FROM messages WHERE sender=?1 AND id=?2",params![peer,id],|r|r.get::<_,String>(0)).optional()?{ensure!(old==hash,"MESSAGE_ID_CONFLICT");return Ok(());}
            self.check_grant(tx,&grant,crate::GrantUse{peer,cap:"message.direct",resource:"inbox",action:"send",input:Some(&input_hash),consume:true,bytes:0,time})?;
            if let Some(channel)=channel {
                let member:Option<String>=tx.query_row("SELECT porch FROM trust WHERE peer=?1",[peer],|r|r.get(0)).optional()?.flatten();
                ensure!(member.as_deref()==Some(channel),"CHANNEL_MEMBERSHIP_REQUIRED");
            }
            tx.execute("INSERT INTO messages(sender,id,hash,record) VALUES(?1,?2,?3,?4)",params![peer,id,hash,serde_json::to_string(&json!({"id":id,"sender":peer,"text":text,"channel":channel,"receipt":receipt}))?])?;
            Db::append(tx,&self.key,"message.received",peer,json!({"message_digest":hash,"receipt_hash":receipt.hash()?}))?;Ok(())
        })?;
        Ok(serde_json::to_value(receipt)?)
    }
    pub async fn send_message(self: &Arc<Self>, args: &Value) -> Result<Value> {
        let peer = args["peer"].as_str().context("PEER_REQUIRED")?;
        ensure!(self.db.trusted(peer)?, "UNTRUSTED_PEER");
        let text = args["text"].as_str().context("TEXT_REQUIRED")?;
        ensure!(
            text.len() <= self.config.read().unwrap().limits.message_bytes,
            "MESSAGE_TOO_LARGE"
        );
        let grant = self
            .remote_grant(peer, "message.direct", "inbox", "send")?
            .context("AUTHORITY_REQUIRED")?;
        let id = args["id"]
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(nonce);
        let request = json!({"id":id,"text":text,"channel":args["channel"],"grant":grant});
        match self.rpc(peer, "message.send", request.clone()).await {
            Ok(reply) => {
                ensure!(reply.payload.status == "OK", "{}", reply.payload.code);
                self.event(
                    "message.sent",
                    peer,
                    json!({"message_digest":digest(text.as_bytes())}),
                )?;
                Ok(json!({"status":"DELIVERED","receipt":reply.payload.output}))
            }
            Err(e) if args["queue_if_offline"] == true => {
                self.db.transaction(|tx| {
                    let count: u64 = tx.query_row(
                        "SELECT count(*) FROM outbox WHERE status='QUEUED'",
                        [],
                        |r| r.get(0),
                    )?;
                    ensure!(
                        count < self.config.read().unwrap().limits.queued_jobs as u64,
                        "OUTBOX_FULL"
                    );
                    tx.execute(
                        "INSERT INTO outbox(id,peer,record,status) VALUES(?1,?2,?3,'QUEUED')",
                        params![id, peer, serde_json::to_string(&request)?],
                    )?;
                    Ok(())
                })?;
                Ok(json!({"status":"QUEUED","id":id,"reason":e.to_string()}))
            }
            Err(e) => Err(e),
        }
    }
    pub async fn retry_outbox(self: &Arc<Self>) -> Result<Value> {
        let rows = self.db.transaction(|tx| {
            let mut s = tx.prepare(
                "SELECT id,peer,record,attempts FROM outbox WHERE status='QUEUED' LIMIT 64",
            )?;
            Ok(s.query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, u32>(3)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?)
        })?;
        let mut results = Vec::new();
        for (id, peer, record, attempts) in rows {
            let result = if self.db.trusted(&peer)? && attempts < 10 {
                self.rpc(&peer, "message.send", serde_json::from_str(&record)?)
                    .await
            } else {
                Err(anyhow::anyhow!("OUTBOX_REFUSED"))
            };
            let status = match &result {
                Ok(r) if r.payload.status == "OK" => "DELIVERED",
                Ok(_) => "REFUSED",
                Err(_) if attempts >= 9 || !self.db.trusted(&peer)? => "REFUSED",
                Err(_) => "QUEUED",
            };
            self.db.transaction(|tx| {
                tx.execute(
                    "UPDATE outbox SET status=?2,attempts=attempts+1 WHERE id=?1",
                    params![id, status],
                )?;
                Ok(())
            })?;
            results.push(json!({"id":id,"status":status}));
        }
        Ok(json!(results))
    }
    pub fn snapshot(&self) -> Result<Value> {
        let c = self.config.read().unwrap().clone();
        let observations = self.observations.read().unwrap().clone();
        let jobs = self.db.records("jobs")?;
        let grants = self.db.grants()?;
        Ok(
            json!({"version":VERSION,"candidate":env!("CARGO_PKG_VERSION"),"node":{"id":self.id,"alias":c.alias,"address":format!("porch://peer/{}",self.id),"status":"RUNNING"},"porch":self.db.get("porch")?,"peers":self.db.trust_rows()?,"resources":self.advertisement()?,"models":{"local":c.models,"remote":self.db.records("advertisements")?},"network":observations,"grants":grants,"jobs":jobs,"active_jobs":self.db.active_jobs()?,"qualification":qualification::status(self)?,"storage":storage::list(self)?,"messages":self.db.records("messages")?,"ledger":self.db.records("ledger")?,"accounting":{"model_calls":jobs.iter().filter(|j|j["payload"]["status"]=="COMPLETED" && j["payload"]["capability"]=="model.inference").count(),"jobs_refused":jobs.iter().filter(|j|j["payload"]["status"]=="REFUSED").count(),"duration_ms":jobs.iter().map(|j|j["payload"]["duration_ms"].as_u64().unwrap_or(0)).sum::<u64>()},"claims":{"wan_nat":"unverified","gpu_pooling":false,"relay_server":false,"federation":false,"telemetry":false}}),
        )
    }
}

/// Display labels cannot inject control sequences or bidirectional impersonation.
pub fn valid_label(s: &str, max: usize) -> bool {
    !s.is_empty()
        && s.len() <= max
        && !s.chars().any(|c| {
            c.is_control()
                || ('\u{202a}'..='\u{202e}').contains(&c)
                || ('\u{2066}'..='\u{2069}').contains(&c)
        })
}
/// Peer-visible failures use fixed codes; parser/provider messages may contain private data.
pub fn safe_error(e: &anyhow::Error) -> String {
    let s = e.to_string();
    if s.len() <= 96
        && !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
    {
        s
    } else {
        "MALFORMED_OR_PROVIDER_TRANSPORT_FAILURE".into()
    }
}
