use crate::{Node, db::Db};
use anyhow::{Context, Result, ensure};
use porch_core::*;
use rusqlite::{OptionalExtension, params};
use serde_json::{Value, json};
use std::{
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
    sync::Arc,
};

fn path(node: &Node, cid: &str) -> Result<PathBuf> {
    validate_cid(cid)?;
    let root = node.root.join("vault");
    std::fs::create_dir_all(&root)?;
    Ok(root.join(cid))
}
fn temporary(node: &Node, cid: &str, owner: &str) -> Result<PathBuf> {
    validate_cid(cid)?;
    let root = node.root.join("transfers");
    std::fs::create_dir_all(&root)?;
    Ok(root.join(format!("{cid}-{}", digest(owner.as_bytes()))))
}
pub fn list(node: &Node) -> Result<Value> {
    node.db.transaction(|tx|{
    let mut s=tx.prepare("SELECT cid,owner,size,pinned,tombstone FROM blobs ORDER BY cid LIMIT 1000")?;
    let blobs=s.query_map([],|r|Ok(json!({"cid":r.get::<_,String>(0)?,"owner":r.get::<_,String>(1)?,"ciphertext_bytes":r.get::<_,u64>(2)?,"pinned":r.get::<_,bool>(3)?,"tombstone":r.get::<_,bool>(4)?})))?.collect::<rusqlite::Result<Vec<_>>>()?;
    let mut s=tx.prepare("SELECT cid,peer,desired,verified FROM replicas ORDER BY cid,peer LIMIT 1000")?;
    let replicas=s.query_map([],|r|Ok(json!({"cid":r.get::<_,String>(0)?,"peer":r.get::<_,String>(1)?,"desired":r.get::<_,u64>(2)?,"verified":r.get::<_,u64>(3)?})))?.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(json!({"blobs":blobs,"replicas":replicas,"algorithm":"XChaCha20-Poly1305","content_id":"SHA-256(ciphertext)","maximum_blob_bytes":MAX_BLOB,"replication_is_backup":false}))
})
}
pub fn recover(node: &Node) -> Result<()> {
    let _guard = node.storage_lock.lock().unwrap();
    let rows = node.db.transaction(|tx| {
        let mut s = tx.prepare("SELECT cid,owner,offset,size FROM uploads")?;
        Ok(s.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, u64>(2)?,
                r.get::<_, u64>(3)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?)
    })?;
    for (cid, owner, offset, size) in rows {
        let p = temporary(node, &cid, &owner)?;
        if p.exists() {
            let f = std::fs::OpenOptions::new().write(true).open(&p)?;
            let actual = f.metadata()?.len();
            ensure!(actual >= offset, "TRANSFER_RECOVERY_TRUNCATED");
            f.set_len(offset)?;
        } else {
            let complete = path(node, &cid)?;
            if complete.exists() {
                let bytes = std::fs::read(&complete)?;
                ensure!(
                    bytes.len() as u64 == size && digest(&bytes) == cid,
                    "CORRUPTED_RECOVERED_BLOB"
                );
                // The final rename may have reached disk before the metadata
                // transaction. Finish that transaction without rerunning input.
                node.db.transaction(|tx| {
                    tx.execute("INSERT INTO blobs(cid,owner,size) VALUES(?1,?2,?3) ON CONFLICT(cid) DO NOTHING", params![cid,owner,size])?;
                    let saved:(String,u64,bool)=tx.query_row("SELECT owner,size,tombstone FROM blobs WHERE cid=?1",[&cid],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
                    ensure!(saved==(owner.clone(),size,false),"RECOVERED_BLOB_METADATA_CONFLICT");
                    tx.execute("DELETE FROM uploads WHERE cid=?1 AND owner=?2",params![cid,owner])?;
                    Db::append(tx,&node.key,"storage.completion.recovered",&cid,json!({"ciphertext_digest":cid,"ciphertext_bytes":size,"owner":owner}))?;
                    Ok(())
                })?;
            } else {
                node.db.transaction(|tx| {
                    tx.execute(
                        "DELETE FROM uploads WHERE cid=?1 AND owner=?2",
                        params![cid, owner],
                    )?;
                    Ok(())
                })?;
            }
        }
    }
    Ok(())
}
fn expire_uploads(node: &Node, time: u64) -> Result<()> {
    let rows = node.db.transaction(|tx| {
        let mut s = tx.prepare("SELECT cid,owner FROM uploads WHERE expires<=?1")?;
        Ok(s.query_map([time], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?)
    })?;
    for (cid, owner) in rows {
        let p = temporary(node, &cid, &owner)?;
        if p.exists() {
            std::fs::remove_file(p)?;
        }
        node.db.transaction(|tx| {
            tx.execute(
                "DELETE FROM uploads WHERE cid=?1 AND owner=?2",
                params![cid, owner],
            )?;
            Db::append(
                tx,
                &node.key,
                "storage.transfer.expired",
                &cid,
                json!({"owner":owner}),
            )?;
            Ok(())
        })?;
    }
    Ok(())
}
pub fn put_local(node: &Node, plaintext: &[u8]) -> Result<Value> {
    let _guard = node.storage_lock.lock().unwrap();
    let key = random_bytes::<32>();
    let ciphertext = seal(plaintext, &key, &node.id)?;
    ensure!(
        ciphertext.len() <= node.config.read().unwrap().limits.blob_bytes,
        "BLOB_EXCEEDS_OPERATOR_CAP"
    );
    let cid = digest(&ciphertext);
    let p = path(node, &cid)?;
    let used: u64 = node.db.transaction(|tx| {
        Ok(tx.query_row(
            "SELECT (SELECT coalesce(sum(size),0) FROM blobs WHERE tombstone=0)+(SELECT coalesce(sum(size),0) FROM uploads)",
            [],
            |r| r.get(0),
        )?)
    })?;
    ensure!(
        used + ciphertext.len() as u64 <= node.config.read().unwrap().limits.storage_total_bytes,
        "LOCAL_VAULT_QUOTA_EXHAUSTED"
    );
    private_write(&p, &ciphertext)?;
    node.db.transaction(|tx|{tx.execute("INSERT INTO blobs(cid,owner,size,key) VALUES(?1,?2,?3,?4)",params![cid,node.id,ciphertext.len() as u64,hex::encode(key)])?;Db::append(tx,&node.key,"blob.sealed",&cid,json!({"ciphertext_digest":cid,"plaintext_digest":digest(plaintext),"ciphertext_bytes":ciphertext.len()}))?;Ok(())})?;
    Ok(
        json!({"cid":cid,"address":format!("porch://blob/{cid}"),"ciphertext_bytes":ciphertext.len(),"plaintext_bytes":plaintext.len()}),
    )
}
pub fn get_local(node: &Node, cid: &str) -> Result<Value> {
    let (owner, key, tombstone): (String, Option<String>, bool) = node.db.transaction(|tx| {
        Ok(tx.query_row(
            "SELECT owner,key,tombstone FROM blobs WHERE cid=?1",
            [cid],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?)
    })?;
    ensure!(
        owner == node.id && !tombstone,
        "NOT_LOCAL_OWNER_OR_TOMBSTONED"
    );
    let ciphertext = std::fs::read(path(node, cid)?)?;
    ensure!(digest(&ciphertext) == cid, "CORRUPTED_BLOB");
    let key: [u8; 32] = hex::decode(key.context("MISSING_LOCAL_DECRYPTION_KEY")?)?
        .try_into()
        .map_err(|_| anyhow::anyhow!("INVALID_LOCAL_KEY"))?;
    let plaintext = unseal(&ciphertext, &key, &node.id)?;
    Ok(
        json!({"cid":cid,"plaintext_hex":hex::encode(&plaintext),"plaintext_bytes":plaintext.len(),"integrity":"VERIFIED"}),
    )
}
pub fn pin(node: &Node, cid: &str, pinned: bool) -> Result<Value> {
    validate_cid(cid)?;
    node.db.transaction(|tx| {
        let count = tx.execute(
            "UPDATE blobs SET pinned=?2 WHERE cid=?1 AND owner=?3 AND tombstone=0",
            params![cid, pinned, node.id],
        )?;
        ensure!(count == 1, "BLOB_NOT_OWNED");
        Db::append(
            tx,
            &node.key,
            "blob.pin.changed",
            cid,
            json!({"pinned":pinned}),
        )?;
        Ok(())
    })?;
    Ok(json!({"cid":cid,"pinned":pinned}))
}
pub fn delete_local(node: &Node, cid: &str) -> Result<Value> {
    let _guard = node.storage_lock.lock().unwrap();
    validate_cid(cid)?;
    node.db.transaction(|tx| {
        let (owner, pinned): (String, bool) =
            tx.query_row("SELECT owner,pinned FROM blobs WHERE cid=?1", [cid], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })?;
        ensure!(owner == node.id && !pinned, "BLOB_NOT_OWNED_OR_PINNED");
        tx.execute("UPDATE blobs SET tombstone=1,key=NULL WHERE cid=?1", [cid])?;
        tx.execute("UPDATE replicas SET verified=0 WHERE cid=?1", [cid])?;
        Db::append(
            tx,
            &node.key,
            "blob.tombstoned",
            cid,
            json!({"remote_deletion":"separately-requested"}),
        )?;
        Ok(())
    })?;
    let p = path(node, cid)?;
    if p.exists() {
        std::fs::remove_file(p)?;
    }
    Ok(json!({"cid":cid,"tombstoned":true,"secure_erasure_guaranteed":false}))
}
pub fn receive(node: &Node, peer: &str, op: &str, args: &Value) -> Result<Value> {
    let cid = args["cid"].as_str().context("CID_REQUIRED")?;
    validate_cid(cid)?;
    let grant: Signed<Grant> = serde_json::from_value(args["grant"].clone())?;
    let time = node.db.time()?;
    let _guard = node.storage_lock.lock().unwrap();
    let quota = node.config.read().unwrap().storage_quota;
    ensure!(quota > 0, "STORAGE_NOT_SHARED");
    match op {
        "blob.begin" => {
            expire_uploads(node, time)?;
            let size = args["size"].as_u64().context("SIZE_REQUIRED")?;
            ensure!(
                size >= 40
                    && size <= node.config.read().unwrap().limits.blob_bytes as u64
                    && args["encryption"] == "xchacha20poly1305",
                "INVALID_ENCRYPTED_BLOB"
            );
            let offset=node.db.transaction(|tx|{
                node.check_grant(tx,&grant,crate::GrantUse{peer,cap:"blob.storage",resource:"vault",action:"store",input:None,consume:false,bytes:0,time})?;
                if let Some((owner,old_size,tombstone))=tx.query_row("SELECT owner,size,tombstone FROM blobs WHERE cid=?1",[cid],|r|Ok((r.get::<_,String>(0)?,r.get::<_,u64>(1)?,r.get::<_,bool>(2)?))).optional()?{
                    ensure!(owner==peer && old_size==size && !tombstone,"BLOB_OWNER_SIZE_OR_TOMBSTONE_CONFLICT");return Ok(size);
                }
                if let Some((old_size,offset,nonce,expires))=tx.query_row("SELECT size,offset,grant_nonce,expires FROM uploads WHERE cid=?1 AND owner=?2",params![cid,peer],|r|Ok((r.get::<_,u64>(0)?,r.get::<_,u64>(1)?,r.get::<_,String>(2)?,r.get::<_,u64>(3)?))).optional()?{
                    ensure!(old_size==size && nonce==grant.payload.nonce && time<expires,"TRANSFER_CONFLICT_OR_EXPIRED");return Ok(offset);
                }
                let existing:u64=tx.query_row("SELECT count(*) FROM uploads",[],|r|r.get(0))?;ensure!(existing<16,"TRANSFER_QUEUE_FULL");
                let used:u64=tx.query_row("SELECT coalesce(sum(size),0) FROM blobs WHERE owner!=?1 AND tombstone=0",[&node.id],|r|r.get(0))?;
                let reserved:u64=tx.query_row("SELECT coalesce(sum(size),0) FROM uploads",[],|r|r.get(0))?;
                ensure!(used+reserved+size<=quota,"STORAGE_QUOTA_EXHAUSTED");
                let total:u64=tx.query_row("SELECT (SELECT coalesce(sum(size),0) FROM blobs WHERE tombstone=0)+(SELECT coalesce(sum(size),0) FROM uploads)",[],|r|r.get(0))?;ensure!(total+size<=node.config.read().unwrap().limits.storage_total_bytes,"STORAGE_TOTAL_CAP_EXHAUSTED");
                node.check_grant(tx,&grant,crate::GrantUse{peer,cap:"blob.storage",resource:"vault",action:"store",input:None,consume:true,bytes:size,time})?;
                tx.execute("INSERT INTO uploads(cid,owner,size,grant_nonce,expires) VALUES(?1,?2,?3,?4,?5)",params![cid,peer,size,grant.payload.nonce,grant.payload.expires_at])?;Ok(0)
            })?;
            let p = temporary(node, cid, peer)?;
            if offset < size && !p.exists() {
                private_write(&p, &[])?;
            }
            Ok(json!({"cid":cid,"offset":offset,"size":size,"resume":offset>0}))
        }
        "blob.chunk" => {
            let bytes = hex::decode(args["data_hex"].as_str().context("DATA_REQUIRED")?)?;
            ensure!(
                bytes.len() <= CHUNK && !bytes.is_empty(),
                "CHUNK_TOO_LARGE_OR_EMPTY"
            );
            let wanted = args["offset"].as_u64().context("OFFSET_REQUIRED")?;
            let (size,offset)=node.db.transaction(|tx|{
                node.check_grant(tx,&grant,crate::GrantUse{peer,cap:"blob.storage",resource:"vault",action:"store",input:None,consume:false,bytes:0,time})?;
                let (size,offset,nonce,expires):(u64,u64,String,u64)=tx.query_row("SELECT size,offset,grant_nonce,expires FROM uploads WHERE cid=?1 AND owner=?2",params![cid,peer],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).context("TRANSFER_NOT_STARTED")?;
                ensure!(nonce==grant.payload.nonce && time<expires && wanted==offset && offset+bytes.len() as u64<=size,"TRANSFER_OFFSET_OR_SCOPE_MISMATCH");Ok((size,offset))
            })?;
            let p = temporary(node, cid, peer)?;
            let mut f = std::fs::OpenOptions::new().write(true).open(&p)?;
            f.seek(SeekFrom::Start(offset))?;
            f.write_all(&bytes)?;
            f.sync_all()?;
            let next = offset + bytes.len() as u64;
            if next == size {
                let cipher = std::fs::read(&p)?;
                if digest(&cipher) != cid {
                    std::fs::remove_file(&p)?;
                    node.db.transaction(|tx| {
                        tx.execute(
                            "DELETE FROM uploads WHERE cid=?1 AND owner=?2",
                            params![cid, peer],
                        )?;
                        Db::append(tx, &node.key, "blob.integrity.refused", cid, json!({}))?;
                        Ok(())
                    })?;
                    anyhow::bail!("CORRUPTED_BLOB");
                }
                std::fs::rename(&p, path(node, cid)?)?;
            }
            let receipt = Signed::new(
                &node.key,
                "porch.blob.receipt.v1",
                json!({"cid":cid,"owner":peer,"executor":node.id,"stored_bytes":next,"complete":next==size,"integrity_verified":next==size,"grant_nonce":grant.payload.nonce,"timestamp":time}),
            )?;
            node.db.transaction(|tx| {
                if next == size {
                    tx.execute(
                        "INSERT INTO blobs(cid,owner,size) VALUES(?1,?2,?3)",
                        params![cid, peer, size],
                    )?;
                    tx.execute(
                        "DELETE FROM uploads WHERE cid=?1 AND owner=?2",
                        params![cid, peer],
                    )?;
                } else {
                    tx.execute(
                        "UPDATE uploads SET offset=?3 WHERE cid=?1 AND owner=?2",
                        params![cid, peer, next],
                    )?;
                }
                Db::append(
                    tx,
                    &node.key,
                    if next == size {
                        "storage.replicated"
                    } else {
                        "storage.chunk.received"
                    },
                    cid,
                    json!({"bytes":bytes.len(),"receipt_hash":receipt.hash()?}),
                )?;
                Ok(())
            })?;
            Ok(json!({"offset":next,"receipt":receipt}))
        }
        "blob.get" => {
            let offset = args["offset"].as_u64().unwrap_or(0);
            let size = node.db.transaction(|tx| {
                node.check_grant(
                    tx,
                    &grant,
                    crate::GrantUse {
                        peer,
                        cap: "blob.storage",
                        resource: "vault",
                        action: "store",
                        input: None,
                        consume: false,
                        bytes: 0,
                        time,
                    },
                )?;
                let (owner, size, tombstone): (String, u64, bool) = tx.query_row(
                    "SELECT owner,size,tombstone FROM blobs WHERE cid=?1",
                    [cid],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )?;
                ensure!(
                    owner == peer && !tombstone && offset <= size,
                    "BLOB_NOT_OWNED_OR_TOMBSTONED"
                );
                Ok(size)
            })?;
            let p = path(node, cid)?;
            let content = std::fs::read(&p)?;
            ensure!(digest(&content) == cid, "CORRUPTED_BLOB");
            let mut f = std::fs::File::open(p)?;
            f.seek(SeekFrom::Start(offset))?;
            let mut data = vec![0u8; (size - offset).min(CHUNK as u64) as usize];
            f.read_exact(&mut data)?;
            node.db.transaction(|tx| {
                let used: u64 = tx.query_row(
                    "SELECT transfer FROM grants WHERE nonce=?1",
                    [&grant.payload.nonce],
                    |r| r.get(0),
                )?;
                ensure!(
                    used + data.len() as u64
                        <= grant.payload.limits.max_storage_bytes.saturating_mul(4),
                    "TRANSFER_BUDGET_EXHAUSTED"
                );
                tx.execute(
                    "UPDATE grants SET transfer=transfer+?2 WHERE nonce=?1",
                    params![grant.payload.nonce, data.len() as u64],
                )?;
                Db::append(
                    tx,
                    &node.key,
                    "storage.chunk.sent",
                    cid,
                    json!({"bytes":data.len()}),
                )?;
                Ok(())
            })?;
            Ok(json!({"cid":cid,"offset":offset,"size":size,"data_hex":hex::encode(data)}))
        }
        "blob.delete" => {
            node.db.transaction(|tx| {
                node.check_grant(
                    tx,
                    &grant,
                    crate::GrantUse {
                        peer,
                        cap: "blob.storage",
                        resource: "vault",
                        action: "store",
                        input: None,
                        consume: false,
                        bytes: 0,
                        time,
                    },
                )?;
                let (owner, pinned): (String, bool) =
                    tx.query_row("SELECT owner,pinned FROM blobs WHERE cid=?1", [cid], |r| {
                        Ok((r.get(0)?, r.get(1)?))
                    })?;
                ensure!(owner == peer && !pinned, "BLOB_NOT_OWNED_OR_PINNED");
                tx.execute("UPDATE blobs SET tombstone=1 WHERE cid=?1", [cid])?;
                Db::append(
                    tx,
                    &node.key,
                    "blob.tombstoned",
                    cid,
                    json!({"requester":peer}),
                )?;
                Ok(())
            })?;
            let p = path(node, cid)?;
            if p.exists() {
                std::fs::remove_file(p)?;
            }
            Ok(json!({"cid":cid,"tombstoned":true,"secure_erasure_guaranteed":false}))
        }
        _ => anyhow::bail!("STORAGE_OPERATION_UNSUPPORTED"),
    }
}
async fn call(node: &Node, peer: &str, op: &str, args: Value) -> Result<Value> {
    let r = node.rpc(peer, op, args).await?;
    ensure!(r.payload.status == "OK", "{}", r.payload.code);
    Ok(r.payload.output)
}
pub async fn retrieve_cipher(
    node: &Node,
    peer: &str,
    cid: &str,
    grant: &Signed<Grant>,
) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    loop {
        let out = call(
            node,
            peer,
            "blob.get",
            json!({"cid":cid,"offset":bytes.len(),"grant":grant}),
        )
        .await?;
        let size = out["size"].as_u64().context("INVALID_REMOTE_SIZE")?;
        ensure!(
            size <= node.config.read().unwrap().limits.blob_bytes as u64
                && out["cid"] == cid
                && out["offset"] == bytes.len(),
            "INVALID_REMOTE_BLOB"
        );
        let chunk = hex::decode(out["data_hex"].as_str().context("INVALID_CHUNK")?)?;
        ensure!(chunk.len() <= CHUNK && !chunk.is_empty(), "INVALID_CHUNK");
        ensure!(
            bytes.len() + chunk.len() <= size as usize,
            "INVALID_REMOTE_SIZE"
        );
        bytes.extend(chunk);
        if bytes.len() == size as usize {
            break;
        }
    }
    ensure!(digest(&bytes) == cid, "CORRUPTED_REMOTE_BLOB");
    Ok(bytes)
}
pub async fn replicate(node: &Arc<Node>, peer: &str, cid: &str) -> Result<Value> {
    ensure!(node.db.trusted(peer)?, "UNTRUSTED_PEER");
    let grant = node
        .remote_grant(peer, "blob.storage", "vault", "store")?
        .context("AUTHORITY_REQUIRED")?;
    validate_cid(cid)?;
    let ciphertext = std::fs::read(path(node, cid)?)?;
    ensure!(digest(&ciphertext) == cid, "CORRUPTED_LOCAL_BLOB");
    node.db.transaction(|tx|{let (owner,tombstone):(String,bool)=tx.query_row("SELECT owner,tombstone FROM blobs WHERE cid=?1",[cid],|r|Ok((r.get(0)?,r.get(1)?)))?;ensure!(owner==node.id && !tombstone,"BLOB_NOT_OWNED_OR_TOMBSTONED");
        tx.execute("INSERT INTO replicas(cid,peer,desired,verified) VALUES(?1,?2,1,0) ON CONFLICT(cid,peer) DO UPDATE SET desired=1,verified=0",params![cid,peer])?;Ok(())})?;
    let reply = call(
        node,
        peer,
        "blob.begin",
        json!({"cid":cid,"size":ciphertext.len(),"encryption":"xchacha20poly1305","grant":grant}),
    )
    .await?;
    let mut offset = reply["offset"].as_u64().context("INVALID_RESUME_OFFSET")? as usize;
    ensure!(offset <= ciphertext.len(), "INVALID_RESUME_OFFSET");
    let mut receipt = Value::Null;
    while offset < ciphertext.len() {
        let end = (offset + CHUNK).min(ciphertext.len());
        let r=call(node,peer,"blob.chunk",json!({"cid":cid,"offset":offset,"data_hex":hex::encode(&ciphertext[offset..end]),"grant":grant})).await?;
        ensure!(r["offset"] == end, "INVALID_RESUME_OFFSET");
        let signed: Signed<Value> = serde_json::from_value(r["receipt"].clone())?;
        signed.verify("porch.blob.receipt.v1")?;
        ensure!(
            signed.signer == peer
                && signed.payload["cid"] == cid
                && signed.payload["owner"] == node.id,
            "BLOB_RECEIPT_BINDING_MISMATCH"
        );
        receipt = serde_json::to_value(signed)?;
        offset = end;
    }
    let fetched = retrieve_cipher(node, peer, cid, &grant).await?;
    ensure!(fetched == ciphertext, "REPLICA_VERIFICATION_FAILED");
    node.db.transaction(|tx| {
        tx.execute(
            "UPDATE replicas SET verified=1,record=?3 WHERE cid=?1 AND peer=?2",
            params![cid, peer, serde_json::to_string(&receipt)?],
        )?;
        Db::append(
            tx,
            &node.key,
            "storage.replica.verified",
            cid,
            json!({"peer":peer,"ciphertext_digest":cid,"verified_at":now()}),
        )?;
        Ok(())
    })?;
    Ok(
        json!({"cid":cid,"peer":peer,"desired_replicas":1,"verified_replicas":1,"verification":"full ciphertext retrieval and digest","receipt":receipt}),
    )
}
pub async fn fetch(node: &Arc<Node>, peer: &str, cid: &str) -> Result<Value> {
    ensure!(node.db.trusted(peer)?, "UNTRUSTED_PEER");
    validate_cid(cid)?;
    node.db.transaction(|tx| {
        let (owner, tombstone): (String, bool) = tx.query_row(
            "SELECT owner,tombstone FROM blobs WHERE cid=?1",
            [cid],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        ensure!(
            owner == node.id && !tombstone,
            "BLOB_NOT_OWNED_OR_TOMBSTONED"
        );
        Ok(())
    })?;
    let grant = node
        .remote_grant(peer, "blob.storage", "vault", "store")?
        .context("AUTHORITY_REQUIRED")?;
    let bytes = retrieve_cipher(node, peer, cid, &grant).await?;
    let _guard = node.storage_lock.lock().unwrap();
    // A local tombstone could have been issued during the network request.
    node.db.transaction(|tx| {
        let deleted: bool =
            tx.query_row("SELECT tombstone FROM blobs WHERE cid=?1", [cid], |r| {
                r.get(0)
            })?;
        ensure!(!deleted, "BLOB_TOMBSTONED");
        Ok(())
    })?;
    let p = path(node, cid)?;
    let replacement = node
        .root
        .join("transfers")
        .join(format!("fetch-{}", nonce()));
    std::fs::create_dir_all(node.root.join("transfers"))?;
    private_write(&replacement, &bytes)?;
    // POSIX rename replaces atomically. Windows requires removing the existing
    // target; the caller can retry retrieval if interrupted in that short gap.
    #[cfg(windows)]
    if p.exists() {
        std::fs::remove_file(&p)?;
    }
    std::fs::rename(&replacement, p)?;
    get_local(node, cid)
}
pub async fn delete_remote(node: &Arc<Node>, peer: &str, cid: &str) -> Result<Value> {
    let grant = node
        .remote_grant(peer, "blob.storage", "vault", "store")?
        .context("AUTHORITY_REQUIRED")?;
    let result = call(node, peer, "blob.delete", json!({"cid":cid,"grant":grant})).await?;
    node.db.transaction(|tx| {
        tx.execute(
            "UPDATE replicas SET desired=0,verified=0 WHERE cid=?1 AND peer=?2",
            params![cid, peer],
        )?;
        Ok(())
    })?;
    Ok(result)
}
