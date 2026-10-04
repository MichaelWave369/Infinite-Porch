use anyhow::{Context, Result, ensure};
use libp2p::identity::Keypair;
use porch_core::{Signed, canonical, digest, now};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde_json::{Value, json};
use std::{path::Path, sync::Mutex};

pub struct Db {
    pub connection: Mutex<Connection>,
    pub events: tokio::sync::broadcast::Sender<Value>,
}
impl Db {
    pub fn open(path: &Path, owner: &str) -> Result<Self> {
        let connection = Connection::open(path)?;
        let schema: u32 = connection.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        ensure!(schema <= 2, "DATABASE_SCHEMA_UNSUPPORTED");
        // Check ownership before creating a backup or changing a known schema.
        if schema > 0 {
            let stored: Option<String> = connection
                .query_row("SELECT value FROM meta WHERE key='owner'", [], |r| r.get(0))
                .optional()?;
            ensure!(
                stored.as_deref() == Some(owner),
                "DATABASE_IDENTITY_MISMATCH"
            );
        }
        if schema == 1 {
            let backup = path.with_extension("v1-backup.sqlite");
            ensure!(
                !backup.exists(),
                "MIGRATION_BACKUP_ALREADY_EXISTS_REVIEW_REQUIRED"
            );
            connection.execute("VACUUM INTO ?1", [backup.to_string_lossy().as_ref()])?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&backup, std::fs::Permissions::from_mode(0o600))?;
            }
        }
        connection.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON; PRAGMA busy_timeout=3000; PRAGMA max_page_count=32768; BEGIN IMMEDIATE;
          CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY,value TEXT NOT NULL);
          CREATE TABLE IF NOT EXISTS trust (peer TEXT PRIMARY KEY,alias TEXT NOT NULL,porch TEXT,active INTEGER NOT NULL,record TEXT NOT NULL);
          CREATE UNIQUE INDEX IF NOT EXISTS trust_alias ON trust(alias) WHERE active=1;
          CREATE TABLE IF NOT EXISTS invites (nonce TEXT PRIMARY KEY,hash TEXT NOT NULL,used INTEGER NOT NULL DEFAULT 0,expires INTEGER NOT NULL);
          CREATE TABLE IF NOT EXISTS grants (nonce TEXT PRIMARY KEY,issuer TEXT NOT NULL,recipient TEXT NOT NULL,record TEXT NOT NULL,revoked INTEGER NOT NULL DEFAULT 0,calls INTEGER NOT NULL DEFAULT 0,window INTEGER NOT NULL DEFAULT 0,hour_calls INTEGER NOT NULL DEFAULT 0,bytes INTEGER NOT NULL DEFAULT 0,transfer INTEGER NOT NULL DEFAULT 0);
          CREATE TABLE IF NOT EXISTS replay (peer TEXT NOT NULL,nonce TEXT NOT NULL,expires INTEGER NOT NULL,PRIMARY KEY(peer,nonce));
          CREATE TABLE IF NOT EXISTS jobs (requester TEXT NOT NULL,id TEXT NOT NULL,hash TEXT NOT NULL,status TEXT NOT NULL,receipt TEXT,PRIMARY KEY(requester,id));
          CREATE TABLE IF NOT EXISTS ledger (sequence INTEGER PRIMARY KEY AUTOINCREMENT,record TEXT NOT NULL,hash TEXT NOT NULL);
          CREATE TABLE IF NOT EXISTS advertisements(peer TEXT PRIMARY KEY,record TEXT NOT NULL,expires INTEGER NOT NULL);
          CREATE TABLE IF NOT EXISTS blobs(cid TEXT PRIMARY KEY,owner TEXT NOT NULL,size INTEGER NOT NULL,key TEXT,pinned INTEGER NOT NULL DEFAULT 0,tombstone INTEGER NOT NULL DEFAULT 0);
          CREATE TABLE IF NOT EXISTS uploads(cid TEXT NOT NULL,owner TEXT NOT NULL,size INTEGER NOT NULL,offset INTEGER NOT NULL DEFAULT 0,grant_nonce TEXT NOT NULL,expires INTEGER NOT NULL,PRIMARY KEY(cid,owner));
          CREATE TABLE IF NOT EXISTS replicas(cid TEXT NOT NULL,peer TEXT NOT NULL,desired INTEGER NOT NULL DEFAULT 1,verified INTEGER NOT NULL DEFAULT 0,record TEXT,PRIMARY KEY(cid,peer));
          CREATE TABLE IF NOT EXISTS messages(sender TEXT NOT NULL,id TEXT NOT NULL,hash TEXT NOT NULL,record TEXT NOT NULL,PRIMARY KEY(sender,id));
          CREATE TABLE IF NOT EXISTS outbox(id TEXT PRIMARY KEY,peer TEXT NOT NULL,record TEXT NOT NULL,status TEXT NOT NULL,attempts INTEGER NOT NULL DEFAULT 0);
          CREATE TABLE IF NOT EXISTS outbound_jobs(id TEXT PRIMARY KEY,intent_hash TEXT NOT NULL,job TEXT NOT NULL,route TEXT NOT NULL,result TEXT);
          CREATE TABLE IF NOT EXISTS clients(nonce TEXT PRIMARY KEY,token_digest TEXT UNIQUE NOT NULL,record TEXT NOT NULL,revoked INTEGER NOT NULL DEFAULT 0,calls INTEGER NOT NULL DEFAULT 0);
          CREATE TABLE IF NOT EXISTS qualification(id TEXT PRIMARY KEY,record TEXT NOT NULL);
          PRAGMA user_version=2; COMMIT;")?;
        let old: Option<String> = connection
            .query_row("SELECT value FROM meta WHERE key='owner'", [], |r| r.get(0))
            .optional()?;
        if let Some(old) = old {
            ensure!(old == owner, "DATABASE_IDENTITY_MISMATCH");
        } else {
            connection.execute("INSERT INTO meta(key,value) VALUES('owner',?1)", [owner])?;
        }
        let (events, _) = tokio::sync::broadcast::channel(256);
        let db = Self {
            connection: Mutex::new(connection),
            events,
        };
        db.verify_ledger(owner)?;
        db.transaction(|tx| {
            tx.execute(
                "UPDATE jobs SET status='UNCERTAIN' WHERE status='RUNNING'",
                [],
            )?;
            Ok(())
        })?;
        Ok(db)
    }
    pub fn transaction<T>(&self, f: impl FnOnce(&Transaction<'_>) -> Result<T>) -> Result<T> {
        let mut conn = self
            .connection
            .lock()
            .map_err(|_| anyhow::anyhow!("DATABASE_LOCK_POISONED"))?;
        let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let before: u64 =
            tx.query_row("SELECT coalesce(max(sequence),0) FROM ledger", [], |r| {
                r.get(0)
            })?;
        let result = f(&tx)?;
        let notifications = {
            let mut stmt =
                tx.prepare("SELECT record FROM ledger WHERE sequence>?1 ORDER BY sequence")?;
            stmt.query_map([before], |r| r.get::<_, String>(0))?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };
        tx.commit()?;
        for record in notifications {
            if let Ok(value) = serde_json::from_str(&record) {
                let _ = self.events.send(value);
            }
        }
        Ok(result)
    }
    pub fn get(&self, key: &str) -> Result<Option<Value>> {
        let conn = self
            .connection
            .lock()
            .map_err(|_| anyhow::anyhow!("DATABASE_LOCK_POISONED"))?;
        let value: Option<String> = conn
            .query_row("SELECT value FROM meta WHERE key=?1", [key], |r| r.get(0))
            .optional()?;
        value.map(|v| Ok(serde_json::from_str(&v)?)).transpose()
    }
    pub fn set(&self, key: &str, value: &Value) -> Result<()> {
        self.transaction(|tx|{tx.execute("INSERT INTO meta(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",params![key,serde_json::to_string(value)?])?;Ok(())})
    }
    pub fn trusted(&self, peer: &str) -> Result<bool> {
        let conn = self
            .connection
            .lock()
            .map_err(|_| anyhow::anyhow!("DATABASE_LOCK_POISONED"))?;
        Ok(conn
            .query_row("SELECT active FROM trust WHERE peer=?1", [peer], |r| {
                r.get::<_, bool>(0)
            })
            .optional()?
            .unwrap_or(false))
    }
    pub fn trust_rows(&self) -> Result<Vec<Value>> {
        let conn = self
            .connection
            .lock()
            .map_err(|_| anyhow::anyhow!("DATABASE_LOCK_POISONED"))?;
        let mut stmt = conn
            .prepare("SELECT peer,alias,porch,active,record FROM trust ORDER BY peer LIMIT 128")?;
        let rows=stmt.query_map([],|r|Ok(json!({"peer":r.get::<_,String>(0)?,"alias":r.get::<_,String>(1)?,"porch":r.get::<_,Option<String>>(2)?,"active":r.get::<_,bool>(3)?,"record":serde_json::from_str::<Value>(&r.get::<_,String>(4)?).unwrap_or(Value::Null)})))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
    pub fn grants(&self) -> Result<Vec<Value>> {
        let conn = self
            .connection
            .lock()
            .map_err(|_| anyhow::anyhow!("DATABASE_LOCK_POISONED"))?;
        let mut stmt = conn.prepare(
            "SELECT record,revoked,calls,hour_calls,bytes FROM grants ORDER BY nonce LIMIT 1000",
        )?;
        let rows=stmt.query_map([],|r|Ok(json!({"grant":serde_json::from_str::<Value>(&r.get::<_,String>(0)?).unwrap_or(Value::Null),"revoked":r.get::<_,bool>(1)?,"consumed_calls":r.get::<_,u64>(2)?,"hour_calls":r.get::<_,u64>(3)?,"reserved_bytes":r.get::<_,u64>(4)?})))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
    pub fn records(&self, kind: &str) -> Result<Vec<Value>> {
        let sql = match kind {
            "ledger" => "SELECT record FROM ledger ORDER BY sequence DESC LIMIT 200",
            "jobs" => {
                "SELECT receipt FROM jobs WHERE receipt IS NOT NULL ORDER BY rowid DESC LIMIT 200"
            }
            "messages" => "SELECT record FROM messages ORDER BY rowid DESC LIMIT 200",
            "advertisements" => "SELECT record FROM advertisements ORDER BY peer LIMIT 128",
            "qualification" => "SELECT record FROM qualification ORDER BY rowid DESC LIMIT 64",
            _ => anyhow::bail!("UNKNOWN_RECORD_TYPE"),
        };
        let conn = self
            .connection
            .lock()
            .map_err(|_| anyhow::anyhow!("DATABASE_LOCK_POISONED"))?;
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
        rows.map(|r| Ok(serde_json::from_str(&r?)?)).collect()
    }
    pub fn ledger(
        &self,
        key: &Keypair,
        event: &str,
        subject: &str,
        hashes: Value,
    ) -> Result<Value> {
        self.transaction(|tx| Self::append(tx, key, event, subject, hashes))
    }
    pub fn append(
        tx: &Transaction<'_>,
        key: &Keypair,
        event: &str,
        subject: &str,
        hashes: Value,
    ) -> Result<Value> {
        let last: Option<(u64, String)> = tx
            .query_row(
                "SELECT sequence,hash FROM ledger ORDER BY sequence DESC LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        let (seq, previous) = last.map(|(s, h)| (s + 1, Some(h))).unwrap_or((1, None));
        let signed = Signed::new(
            key,
            "porch.ledger.v1",
            json!({"sequence":seq,"event_type":event,"timestamp":now(),"subject":subject,"actor":key.public().to_peer_id().to_string(),"hashes":hashes,"previous_hash":previous}),
        )?;
        tx.execute(
            "INSERT INTO ledger(sequence,record,hash) VALUES(?1,?2,?3)",
            params![seq, serde_json::to_string(&signed)?, signed.hash()?],
        )?;
        Ok(serde_json::to_value(signed)?)
    }
    pub fn verify_ledger(&self, owner: &str) -> Result<()> {
        let conn = self
            .connection
            .lock()
            .map_err(|_| anyhow::anyhow!("DATABASE_LOCK_POISONED"))?;
        let mut stmt = conn.prepare("SELECT sequence,record,hash FROM ledger ORDER BY sequence")?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, u64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?;
        let mut previous: Option<String> = None;
        for (expected, row) in (1u64..).zip(rows) {
            let (seq, record, hash) = row?;
            let s: Signed<Value> = serde_json::from_str(&record)?;
            s.verify("porch.ledger.v1").context("LEDGER_TAMPERED")?;
            ensure!(
                s.signer == owner
                    && seq == expected
                    && s.payload["sequence"] == seq
                    && s.payload["previous_hash"] == json!(previous)
                    && digest(&canonical(&s)?) == hash,
                "LEDGER_TAMPERED"
            );
            previous = Some(hash);
        }
        Ok(())
    }
    /// Persist a lower wall-clock bound; backward jumps cannot extend old grants.
    pub fn time(&self) -> Result<u64> {
        let time = now();
        let result = self.transaction(|tx|{
            let old:Option<String>=tx.query_row("SELECT value FROM meta WHERE key='clock'",[],|r|r.get(0)).optional()?;
            let old:u64=old.and_then(|s|s.parse().ok()).unwrap_or(0);
            ensure!(time+2>=old,"CLOCK_UNCERTAIN_BACKWARD_JUMP");
            tx.execute("INSERT INTO meta(key,value) VALUES('clock',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",[time.max(old).to_string()])?;Ok(time.max(old))
        });
        if result.is_err() {
            self.set("clock_anomaly", &json!({"observed_at":time,"reason":"CLOCK_UNCERTAIN_BACKWARD_JUMP","action":"refuse time-dependent authority; correct clock, never extend grants"}))?;
        }
        result
    }
    pub fn active_jobs(&self) -> Result<Vec<Value>> {
        self.transaction(|tx| {
            let mut s=tx.prepare("SELECT requester,id,hash,status FROM jobs WHERE status IN ('RUNNING','UNCERTAIN') ORDER BY rowid DESC LIMIT 200")?;
            Ok(s.query_map([],|r|Ok(json!({"requester":r.get::<_,String>(0)?,"id":r.get::<_,String>(1)?,"request_digest":r.get::<_,String>(2)?,"status":r.get::<_,String>(3)?})))?.collect::<rusqlite::Result<_>>()?)
        })
    }
    pub fn replay(&self, peer: &str, nonce: &str, expires: u64) -> Result<()> {
        ensure!(nonce.len() <= 128 && !nonce.is_empty(), "INVALID_NONCE");
        self.transaction(|tx| {
            tx.execute("DELETE FROM replay WHERE expires<?1", [now()])?;
            let count: u64 = tx.query_row("SELECT count(*) FROM replay", [], |r| r.get(0))?;
            ensure!(count < 4096, "REPLAY_QUEUE_FULL");
            tx.execute(
                "INSERT INTO replay(peer,nonce,expires) VALUES(?1,?2,?3)",
                params![peer, nonce, expires],
            )
            .map_err(|_| anyhow::anyhow!("REPLAY_REJECTED"))?;
            Ok(())
        })
    }
}
