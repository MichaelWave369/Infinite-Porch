//! Local applications receive bounded credentials, never the operator root token.
use crate::{Node, db::Db};
use anyhow::{Context, Result, ensure};
use porch_core::*;
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientGrant {
    pub issuer: String,
    pub holder: String,
    pub nonce: String,
    pub token_digest: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub max_calls: u64,
    pub reads: Vec<String>,
    pub operations: Vec<String>,
    pub resources: Vec<String>,
    pub privacy: Vec<Privacy>,
}
pub fn issue(node: &Node, args: &Value) -> Result<Value> {
    let holder = args["holder"].as_str().context("HOLDER_REQUIRED")?;
    ensure!(!holder.is_empty() && holder.len() <= 64, "INVALID_HOLDER");
    let reads: Vec<String> =
        serde_json::from_value(args.get("reads").cloned().unwrap_or(json!([
            "models",
            "peers",
            "resources",
            "network"
        ])))?;
    let operations: Vec<String> = serde_json::from_value(
        args.get("operations")
            .cloned()
            .unwrap_or(json!(["job.run"])),
    )?;
    ensure!(
        reads
            .iter()
            .all(|r| matches!(r.as_str(), "models" | "peers" | "resources" | "network")),
        "CLIENT_READ_SCOPE_INVALID"
    );
    ensure!(
        operations
            .iter()
            .all(|r| matches!(r.as_str(), "job.run" | "message.send" | "resources.refresh")),
        "CLIENT_ADMIN_AUTHORITY_FORBIDDEN"
    );
    let resources: Vec<String> =
        serde_json::from_value(args.get("resources").cloned().unwrap_or(json!([])))?;
    ensure!(
        resources.len() <= 32 && resources.iter().all(|r| r.len() <= 128),
        "CLIENT_RESOURCE_SCOPE_INVALID"
    );
    let privacy: Vec<Privacy> = serde_json::from_value(
        args.get("privacy")
            .cloned()
            .unwrap_or(json!(["LOCAL_ONLY"])),
    )?;
    let ttl = args["ttl_seconds"].as_u64().unwrap_or(3600);
    let max_calls = args["max_calls"].as_u64().unwrap_or(100);
    ensure!(
        ttl > 0 && ttl <= 86400 && max_calls > 0 && max_calls <= 10000,
        "CLIENT_LIMITS_INVALID"
    );
    let time = node.db.time()?;
    let secret = hex::encode(random_bytes::<32>());
    let grant = Signed::new(
        &node.key,
        "porch.api.client.v1",
        ClientGrant {
            issuer: node.id.clone(),
            holder: holder.into(),
            nonce: nonce(),
            token_digest: digest(secret.as_bytes()),
            created_at: time,
            expires_at: time + ttl,
            max_calls,
            reads,
            operations,
            resources,
            privacy,
        },
    )?;
    node.db.transaction(|tx| {
        let count: u64 = tx.query_row("SELECT count(*) FROM clients", [], |r| r.get(0))?;
        ensure!(count < 128, "CLIENT_CREDENTIAL_LIMIT");
        tx.execute(
            "INSERT INTO clients(nonce,token_digest,record) VALUES(?1,?2,?3)",
            params![
                grant.payload.nonce,
                grant.payload.token_digest,
                serde_json::to_string(&grant)?
            ],
        )?;
        Db::append(
            tx,
            &node.key,
            "client.grant.issued",
            holder,
            json!({"grant_hash":grant.hash()?}),
        )?;
        Ok(())
    })?;
    Ok(
        json!({"token":secret,"grant":grant,"operator_authority":false,"note":"Token is returned once. Give applications this token, never the operator token."}),
    )
}
pub fn lookup(node: &Node, secret: &str) -> Result<Signed<ClientGrant>> {
    ensure!(secret.len() == 64, "OPERATOR_AUTHENTICATION_REQUIRED");
    let hash = digest(secret.as_bytes());
    let record: Option<String> = node.db.transaction(|tx| {
        Ok(tx
            .query_row(
                "SELECT record FROM clients WHERE token_digest=?1 AND revoked=0",
                [hash],
                |r| r.get(0),
            )
            .optional()?)
    })?;
    let grant: Signed<ClientGrant> =
        serde_json::from_str(&record.context("CLIENT_AUTHENTICATION_REQUIRED")?)?;
    grant.verify("porch.api.client.v1")?;
    ensure!(
        grant.signer == node.id
            && grant.payload.issuer == node.id
            && grant.payload.created_at <= node.db.time()?
            && node.db.time()? < grant.payload.expires_at,
        "CLIENT_GRANT_EXPIRED"
    );
    Ok(grant)
}
pub fn authorize(
    node: &Node,
    grant: &Signed<ClientGrant>,
    read: Option<&str>,
    operation: Option<&str>,
    args: &Value,
) -> Result<()> {
    if let Some(read) = read {
        ensure!(
            grant.payload.reads.iter().any(|r| r == read),
            "CLIENT_READ_NOT_GRANTED"
        );
    }
    if let Some(op) = operation {
        ensure!(
            grant.payload.operations.iter().any(|r| r == op),
            "CLIENT_OPERATION_NOT_GRANTED"
        );
        if op == "job.run" {
            let resource = args["resource"].as_str().context("RESOURCE_REQUIRED")?;
            ensure!(
                grant.payload.resources.iter().any(|r| r == resource),
                "CLIENT_RESOURCE_NOT_GRANTED"
            );
            let privacy: Privacy = serde_json::from_value(
                args.get("privacy").cloned().unwrap_or(json!("LOCAL_ONLY")),
            )?;
            ensure!(
                grant.payload.privacy.contains(&privacy),
                "CLIENT_PRIVACY_NOT_GRANTED"
            );
            let duration = args["timeout_ms"].as_u64().unwrap_or(30000);
            ensure!(
                duration > 0 && duration <= 30000,
                "CLIENT_JOB_DURATION_INVALID"
            );
            ensure!(
                node.db.time()? + duration.div_ceil(1000) < grant.payload.expires_at,
                "CLIENT_AUTHORITY_ENDS_BEFORE_JOB_DEADLINE"
            );
        }
    }
    node.db.transaction(|tx|{let count=tx.execute("UPDATE clients SET calls=calls+1 WHERE nonce=?1 AND revoked=0 AND calls<?2",params![grant.payload.nonce,grant.payload.max_calls])?;ensure!(count==1,"CLIENT_GRANT_REVOKED_OR_EXHAUSTED");Db::append(tx,&node.key,"client.request.authorized",&grant.payload.holder,json!({"credential_nonce":grant.payload.nonce,"operation":operation,"read":read,"input_digest":digest(&canonical(args)?)}))?;Ok(())})?;
    Ok(())
}
pub fn revoke(node: &Node, nonce: &str) -> Result<Value> {
    node.db.transaction(|tx| {
        let n = tx.execute("UPDATE clients SET revoked=1 WHERE nonce=?1", [nonce])?;
        ensure!(n == 1, "CLIENT_NOT_FOUND");
        Db::append(tx, &node.key, "client.grant.revoked", nonce, json!({}))?;
        Ok(())
    })?;
    Ok(json!({"revoked":nonce}))
}
