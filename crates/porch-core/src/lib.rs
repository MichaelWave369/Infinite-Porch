//! Versioned, domain-separated contracts shared by the daemon and CLI.
use anyhow::{Context, Result, ensure};
use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit, Payload},
};
use libp2p_identity::{Keypair, PublicKey};
use rand::{RngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

pub const VERSION: u32 = 1;
pub const MAX_INPUT: usize = 64 * 1024;
pub const MAX_BLOB: usize = 8 * 1024 * 1024;
pub const CHUNK: usize = 64 * 1024;
pub const MAX_FRAME: usize = 256 * 1024;
pub const REQUEST_TTL: u64 = 60;

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
pub fn digest(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
pub fn nonce() -> String {
    uuid::Uuid::new_v4().to_string()
}
pub fn random_bytes<const N: usize>() -> [u8; N] {
    let mut b = [0; N];
    OsRng.fill_bytes(&mut b);
    b
}

/// A protocol-specific canonical JSON format, not a claim of RFC 8785 support.
/// No floats, because their cross-language representation is not a signing contract.
pub fn canonical<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    fn sort(v: Value, depth: u32) -> Result<Value> {
        ensure!(depth <= 32, "NESTING_LIMIT");
        Ok(match v {
            Value::Number(n) => {
                ensure!(n.is_i64() || n.is_u64(), "NON_INTEGER_NUMBER");
                Value::Number(n)
            }
            Value::Array(a) => Value::Array(
                a.into_iter()
                    .map(|v| sort(v, depth + 1))
                    .collect::<Result<_>>()?,
            ),
            Value::Object(o) => {
                let sorted = o.into_iter().collect::<std::collections::BTreeMap<_, _>>();
                Value::Object(
                    sorted
                        .into_iter()
                        .map(|(k, v)| Ok((k, sort(v, depth + 1)?)))
                        .collect::<Result<_>>()?,
                )
            }
            x => x,
        })
    }
    Ok(serde_json::to_vec(&sort(serde_json::to_value(value)?, 0)?)?)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Signed<T> {
    pub version: u32,
    pub domain: String,
    pub signer: String,
    pub public_key: String,
    pub payload: T,
    pub signature: String,
}
impl<T: Serialize> Signed<T> {
    pub fn new(key: &Keypair, domain: &str, payload: T) -> Result<Self> {
        let signer = key.public().to_peer_id().to_string();
        let bytes = canonical(
            &json!({"version": VERSION, "domain": domain, "signer": signer, "payload": &payload}),
        )?;
        Ok(Self {
            version: VERSION,
            domain: domain.into(),
            signer,
            public_key: hex::encode(key.public().encode_protobuf()),
            payload,
            signature: hex::encode(key.sign(&bytes)?),
        })
    }
    pub fn verify(&self, domain: &str) -> Result<()> {
        ensure!(
            self.version == VERSION && self.domain == domain,
            "PROTOCOL_MISMATCH"
        );
        let public = PublicKey::try_decode_protobuf(&hex::decode(&self.public_key)?)?;
        ensure!(
            public.to_peer_id().to_string() == self.signer,
            "IDENTITY_MISMATCH"
        );
        let bytes = canonical(
            &json!({"version": self.version, "domain": self.domain, "signer": self.signer, "payload": &self.payload}),
        )?;
        ensure!(
            public.verify(&bytes, &hex::decode(&self.signature)?),
            "INVALID_SIGNATURE"
        );
        Ok(())
    }
    pub fn hash(&self) -> Result<String> {
        Ok(digest(&canonical(self)?))
    }
}

pub fn private_write(path: &Path, data: &[u8]) -> Result<()> {
    use std::io::Write;
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut f = opts
        .open(path)
        .with_context(|| format!("create {}", path.display()))?;
    f.write_all(data)?;
    f.sync_all()?;
    Ok(())
}
pub fn identity(root: &Path) -> Result<Keypair> {
    std::fs::create_dir_all(root)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(root, std::fs::Permissions::from_mode(0o700))?;
    }
    let p = root.join("identity.key");
    if p.exists() {
        return Ok(Keypair::from_protobuf_encoding(&std::fs::read(p)?)?);
    }
    let key = Keypair::generate_ed25519();
    private_write(&p, &key.to_protobuf_encoding()?)?;
    Ok(key)
}

/// nonce || authenticated ciphertext. The owner is additional authenticated data.
pub fn seal(plain: &[u8], key: &[u8; 32], owner: &str) -> Result<Vec<u8>> {
    ensure!(plain.len() <= MAX_BLOB - 40, "BLOB_TOO_LARGE");
    let n = random_bytes::<24>();
    let cipher = XChaCha20Poly1305::new(key.into());
    let mut out = n.to_vec();
    out.extend(
        cipher
            .encrypt(
                XNonce::from_slice(&n),
                Payload {
                    msg: plain,
                    aad: owner.as_bytes(),
                },
            )
            .map_err(|_| anyhow::anyhow!("ENCRYPTION_FAILED"))?,
    );
    Ok(out)
}
pub fn unseal(ciphertext: &[u8], key: &[u8; 32], owner: &str) -> Result<Vec<u8>> {
    ensure!(
        ciphertext.len() >= 40 && ciphertext.len() <= MAX_BLOB,
        "INVALID_CIPHERTEXT"
    );
    XChaCha20Poly1305::new(key.into())
        .decrypt(
            XNonce::from_slice(&ciphertext[..24]),
            Payload {
                msg: &ciphertext[24..],
                aad: owner.as_bytes(),
            },
        )
        .map_err(|_| anyhow::anyhow!("INTEGRITY_FAILURE"))
}
pub fn protected_backup(key: &Keypair, password: &str) -> Result<Value> {
    ensure!(password.len() >= 12, "BACKUP_PASSPHRASE_TOO_SHORT");
    let salt = random_bytes::<16>();
    let mut derived = [0u8; 32];
    argon2::Argon2::default()
        .hash_password_into(password.as_bytes(), &salt, &mut derived)
        .map_err(|_| anyhow::anyhow!("KDF_FAILED"))?;
    Ok(
        json!({"version":VERSION,"kdf":"argon2id-v19-m19456-t2-p1","salt":hex::encode(salt),"cipher":"xchacha20poly1305","owner":key.public().to_peer_id().to_string(),"data":hex::encode(seal(&key.to_protobuf_encoding()?,&derived,"porch.identity.backup.v1")?)}),
    )
}
pub fn restore_backup(value: &Value, password: &str) -> Result<Keypair> {
    ensure!(
        value["version"] == VERSION
            && value["kdf"] == "argon2id-v19-m19456-t2-p1"
            && value["cipher"] == "xchacha20poly1305",
        "BACKUP_FORMAT_UNSUPPORTED"
    );
    let salt = hex::decode(value["salt"].as_str().context("MISSING_SALT")?)?;
    ensure!(salt.len() == 16, "INVALID_SALT");
    let mut derived = [0u8; 32];
    argon2::Argon2::default()
        .hash_password_into(password.as_bytes(), &salt, &mut derived)
        .map_err(|_| anyhow::anyhow!("KDF_FAILED"))?;
    let key = Keypair::from_protobuf_encoding(&unseal(
        &hex::decode(value["data"].as_str().context("MISSING_CIPHERTEXT")?)?,
        &derived,
        "porch.identity.backup.v1",
    )?)?;
    ensure!(
        key.public().to_peer_id().to_string() == value["owner"].as_str().unwrap_or(""),
        "BACKUP_IDENTITY_MISMATCH"
    );
    Ok(key)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Privacy {
    LocalOnly,
    TrustedPeers,
    PorchAllowed,
    FederatedAllowed,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    pub max_calls: u64,
    pub calls_per_hour: u64,
    pub max_input_bytes: u64,
    pub max_output_tokens: u32,
    pub max_duration_ms: u64,
    pub max_storage_bytes: u64,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_calls: 20,
            calls_per_hour: 20,
            max_input_bytes: MAX_INPUT as u64,
            max_output_tokens: 512,
            max_duration_ms: 30000,
            max_storage_bytes: 0,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Grant {
    pub issuer: String,
    pub recipient: String,
    pub capability: String,
    pub resource: String,
    pub action: String,
    pub porch: Option<String>,
    pub limits: Limits,
    pub created_at: u64,
    pub expires_at: u64,
    pub nonce: String,
    pub exact_input_digest: Option<String>,
}
impl Grant {
    pub fn validate(
        &self,
        issuer: &str,
        peer: &str,
        cap: &str,
        resource: &str,
        action: &str,
        time: u64,
    ) -> Result<()> {
        ensure!(
            self.issuer == issuer && self.recipient == peer,
            "GRANT_IDENTITY_MISMATCH"
        );
        ensure!(
            self.capability == cap && self.resource == resource && self.action == action,
            "GRANT_SCOPE_MISMATCH"
        );
        ensure!(
            self.created_at <= time && time < self.expires_at,
            "GRANT_EXPIRED_OR_FUTURE"
        );
        ensure!(
            self.expires_at - self.created_at <= 86400,
            "GRANT_LIFETIME_LIMIT"
        );
        ensure!(
            self.limits.max_calls > 0
                && self.limits.calls_per_hour > 0
                && self.limits.max_duration_ms <= 30000
                && self.limits.max_input_bytes <= MAX_INPUT as u64
                && self.limits.max_output_tokens <= 4096,
            "INVALID_GRANT_LIMITS"
        );
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Job {
    pub version: u32,
    pub id: String,
    pub requester: String,
    pub capability: String,
    pub resource: String,
    pub input: String,
    pub input_digest: String,
    pub max_output_tokens: u32,
    pub timeout_ms: u64,
    pub privacy: Privacy,
    pub execution_mode: String,
    pub preferred_peer: Option<String>,
    pub grant: Option<Signed<Grant>>,
}
impl Job {
    pub fn validate(&self, peer: &str) -> Result<()> {
        ensure!(
            self.version == VERSION && self.requester == peer,
            "JOB_IDENTITY_OR_VERSION_MISMATCH"
        );
        ensure!(
            !self.id.is_empty() && self.id.len() <= 128,
            "INVALID_JOB_ID"
        );
        ensure!(self.input.len() <= MAX_INPUT, "INPUT_TOO_LARGE");
        ensure!(
            digest(self.input.as_bytes()) == self.input_digest,
            "INPUT_DIGEST_MISMATCH"
        );
        ensure!(
            self.timeout_ms > 0
                && self.timeout_ms <= 30000
                && self.max_output_tokens > 0
                && self.max_output_tokens <= 4096,
            "JOB_LIMITS_INVALID"
        );
        ensure!(
            self.execution_mode == "SINGLE_NODE" || self.execution_mode == "REMOTE_NODE",
            "EXECUTION_ENGINE_UNSUPPORTED"
        );
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub version: u32,
    pub job_id: String,
    pub requester: String,
    pub executor: String,
    pub capability: String,
    pub resource: String,
    pub provider: String,
    pub model_version: Option<String>,
    pub input_digest: String,
    pub output_digest: String,
    pub request_digest: String,
    pub status: String,
    pub reason: Option<String>,
    pub output: Option<Value>,
    pub started_at: u64,
    pub finished_at: u64,
    pub duration_ms: u64,
    pub grant_nonce: Option<String>,
    pub measured: Value,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireRequest {
    pub id: String,
    pub nonce: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub operation: String,
    pub args: Value,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireResponse {
    pub request_nonce: String,
    pub request_digest: String,
    pub status: String,
    pub code: String,
    pub output: Value,
    pub timestamp: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Invite {
    pub porch: String,
    pub name: String,
    pub issuer: String,
    pub addresses: Vec<String>,
    pub nonce: String,
    pub recipient: Option<String>,
    pub created_at: u64,
    pub expires_at: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelManifest {
    pub model: String,
    pub provider: String,
    pub version: Option<String>,
    pub claims: Value,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Advertisement {
    pub peer: String,
    pub alias: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub models: Vec<ModelManifest>,
    pub compute_hash: bool,
    pub storage_quota: u64,
    pub advertised: Value,
    pub observed: Value,
    pub services: Vec<Value>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    pub peer: String,
    pub local: bool,
    pub trusted: bool,
    pub same_porch: bool,
    pub model_available: bool,
    pub authorized: bool,
    pub fresh: bool,
    pub has_capacity: bool,
    pub queue_depth: u32,
    pub latency_ms: u64,
}

/// Hard eligibility before deterministic scoring. Model suggestions do not enter it.
pub fn schedule(privacy: &Privacy, candidates: &[Candidate]) -> Value {
    let mut eligible = Vec::new();
    let mut excluded = Vec::new();
    for c in candidates {
        let reason = if *privacy == Privacy::LocalOnly && !c.local {
            Some("LOCAL_ONLY")
        } else if *privacy == Privacy::FederatedAllowed && !c.local && !c.trusted {
            Some("FEDERATION_NOT_ENABLED")
        } else if !c.local && !c.trusted {
            Some("UNTRUSTED")
        } else if *privacy == Privacy::PorchAllowed && !c.local && !c.same_porch {
            Some("PORCH_SCOPE")
        } else if !c.authorized {
            Some("NO_AUTHORITY")
        } else if !c.model_available {
            Some("CAPABILITY_UNAVAILABLE")
        } else if !c.fresh {
            Some("STALE_ADVERTISEMENT")
        } else if !c.has_capacity {
            Some("INSUFFICIENT_RESOURCES")
        } else {
            None
        };
        if let Some(reason) = reason {
            excluded.push(json!({"peer":c.peer,"reason":reason}));
        } else {
            eligible.push(c);
        }
    }
    eligible.sort_by_key(|c| (!c.local, c.queue_depth, c.latency_ms, c.peer.clone()));
    json!({"version":VERSION,"chosen":eligible.first().map(|c|c.peer.clone()),"eligible":eligible.iter().map(|c|json!({"peer":c.peer,"score_order":[!c.local as u8,c.queue_depth as u64,c.latency_ms],"local_executor":c.local,"resource_claims_verified":false})).collect::<Vec<_>>(),"excluded":excluded,"policy":"hard constraints, then local, queue, observed latency, peer-id"})
}
pub fn validate_cid(cid: &str) -> Result<()> {
    ensure!(
        cid.len() == 64
            && cid
                .bytes()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
        "INVALID_CONTENT_ID"
    );
    Ok(())
}
pub fn logical_address(address: &str) -> Result<(&str, &str)> {
    let rest = address
        .strip_prefix("porch://")
        .context("INVALID_PORCH_ADDRESS")?;
    let (kind, id) = rest.split_once('/').context("INVALID_PORCH_ADDRESS")?;
    ensure!(
        matches!(kind, "peer" | "service" | "blob" | "porch" | "model")
            && !id.is_empty()
            && id.len() <= 256
            && !id.contains(".."),
        "INVALID_PORCH_ADDRESS"
    );
    if kind == "blob" {
        validate_cid(id)?;
    }
    Ok((kind, id))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn signatures_reject_tamper_and_cross_protocol_replay() {
        let k = Keypair::generate_ed25519();
        let mut s = Signed::new(&k, "porch.grant.v1", json!({"b":2,"a":1})).unwrap();
        s.verify("porch.grant.v1").unwrap();
        assert!(s.verify("porch.receipt.v1").is_err());
        s.payload["b"] = json!(3);
        assert!(s.verify("porch.grant.v1").is_err());
        assert!(canonical(&json!({"float":0.3})).is_err());
    }
    #[test]
    fn ciphertext_is_private_integrity_bound_and_owner_bound() {
        let k = random_bytes();
        let p = b"private prompts stay private";
        let mut e = seal(p, &k, "owner").unwrap();
        assert_ne!(&e[..p.len()], p);
        assert_eq!(unseal(&e, &k, "owner").unwrap(), p);
        assert!(unseal(&e, &k, "other").is_err());
        e[26] ^= 1;
        assert!(unseal(&e, &k, "owner").is_err());
    }
    #[test]
    fn router_cannot_trade_privacy_or_authority_for_speed() {
        let mut c = Candidate {
            peer: "fast".into(),
            local: false,
            trusted: true,
            same_porch: true,
            model_available: true,
            authorized: true,
            fresh: true,
            has_capacity: true,
            queue_depth: 0,
            latency_ms: 0,
        };
        assert_eq!(
            schedule(&Privacy::LocalOnly, &[c.clone()])["chosen"],
            Value::Null
        );
        c.authorized = false;
        assert_eq!(
            schedule(&Privacy::TrustedPeers, &[c])["chosen"],
            Value::Null
        );
    }
    #[test]
    fn expired_grant_is_not_permission() {
        let g = Grant {
            issuer: "a".into(),
            recipient: "b".into(),
            capability: "c".into(),
            resource: "r".into(),
            action: "run".into(),
            porch: None,
            limits: Limits::default(),
            created_at: 1,
            expires_at: 2,
            nonce: nonce(),
            exact_input_digest: None,
        };
        assert!(g.validate("a", "b", "c", "r", "run", 2).is_err());
    }
    #[test]
    fn version_and_future_engine_boundaries_refuse_instead_of_fallback() {
        let mut job = Job {
            version: VERSION,
            id: nonce(),
            requester: "origin".into(),
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
        job.validate("origin").unwrap();
        for mode in ["ENSEMBLE", "PIPELINE", "SHARDED_MODEL"] {
            job.execution_mode = mode.into();
            assert!(
                job.validate("origin")
                    .unwrap_err()
                    .to_string()
                    .contains("EXECUTION_ENGINE_UNSUPPORTED")
            );
        }
        job.execution_mode = "SINGLE_NODE".into();
        job.version = VERSION + 1;
        assert!(
            job.validate("origin")
                .unwrap_err()
                .to_string()
                .contains("VERSION_MISMATCH")
        );
        assert!(validate_cid("../../identity.key").is_err());
    }
}
