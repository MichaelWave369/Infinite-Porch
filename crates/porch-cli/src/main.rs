use anyhow::{Context, Result, ensure};
use clap::{Parser, Subcommand};
use porch_core::*;
use porch_node::Config;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(version, about = "Infinite Porch operator CLI")]
struct Args {
    #[arg(long, default_value = "state", global = true)]
    data: PathBuf,
    #[arg(long, default_value = "http://127.0.0.1:7331", global = true)]
    api: String,
    #[arg(long, global = true)]
    token_file: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Init {
        #[arg(long, default_value = "My Porch Node")]
        alias: String,
    },
    /// Update safe operator caps while the daemon is stopped; restart required.
    ConfigureLimits {
        file: PathBuf,
    },
    Status,
    Peers,
    Resources,
    Jobs,
    Ledger,
    Grants,
    Network,
    Doctor,
    Qualify {
        #[command(subcommand)]
        command: QualifyCommand,
    },
    Identity {
        #[command(subcommand)]
        command: IdentityCommand,
    },
    Create {
        name: String,
    },
    Invite {
        #[arg(long)]
        recipient: Option<String>,
        #[arg(long, default_value_t = 600)]
        ttl: u64,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Join {
        invite: PathBuf,
    },
    Leave,
    Peer {
        #[command(subcommand)]
        command: PeerCommand,
    },
    Share {
        #[command(subcommand)]
        command: ShareCommand,
    },
    #[command(alias = "models")]
    Model {
        #[command(subcommand)]
        command: ModelCommand,
    },
    Grant {
        #[command(subcommand)]
        command: GrantCommand,
    },
    Storage {
        #[command(subcommand)]
        command: StorageCommand,
    },
    Message {
        peer: String,
        text: String,
        #[arg(long)]
        queue: bool,
    },
    Channel {
        channel: String,
        text: String,
        #[arg(long)]
        queue: bool,
    },
    RetryMessages,
    Resolve {
        address: String,
    },
    Call {
        operation: String,
        #[arg(long, default_value = "{}")]
        json: String,
    },
}
#[derive(Subcommand)]
enum QualifyCommand {
    Host {
        name: String,
        #[arg(long)]
        recipient: String,
        #[arg(long)]
        out: PathBuf,
    },
    Join {
        invite: PathBuf,
        #[arg(long)]
        fingerprint: String,
    },
    Status,
    Run {
        #[arg(long)]
        peer: Option<String>,
        #[arg(long)]
        model: Option<String>,
        #[arg(long,default_value="LOOPBACK",value_parser=["SIMULATED","LOOPBACK","PHYSICAL"])]
        environment: String,
        #[arg(long,default_value="baseline",value_parser=["baseline","revoked","offline","restored","restart"])]
        phase: String,
        #[arg(long)]
        separate_machines_confirmed: bool,
        #[arg(long)]
        wan_condition_confirmed: bool,
        #[arg(long)]
        message: bool,
    },
    Export {
        directory: PathBuf,
    },
    Validate {
        path: PathBuf,
    },
}
#[derive(Subcommand)]
enum IdentityCommand {
    Show,
    Backup {
        #[arg(long)]
        out: PathBuf,
        #[arg(long, default_value = "PORCH_BACKUP_PASSWORD")]
        password_env: String,
    },
    Restore {
        file: PathBuf,
        #[arg(long, default_value = "PORCH_BACKUP_PASSWORD")]
        password_env: String,
    },
    Rotate {
        #[arg(long)]
        new_data: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
}
#[derive(Subcommand)]
enum PeerCommand {
    Approve { peer: String, alias: String },
    Revoke { peer: String },
    Connect { address: String },
    Rotate { proof: PathBuf },
}
#[derive(Subcommand)]
enum ShareCommand {
    Model {
        name: String,
        #[arg(long)]
        private: bool,
    },
    Mock {
        #[arg(default_value = "porch-mock")]
        name: String,
    },
    Storage {
        #[arg(long,value_parser=parse_bytes)]
        limit: u64,
    },
    Compute {
        #[arg(long)]
        disable: bool,
    },
    Refresh,
}
#[derive(Subcommand)]
enum ModelCommand {
    List,
    Discover,
    Scan,
    Refresh,
    Verify {
        model: String,
    },
    Run {
        model: String,
        prompt: String,
        #[arg(long, default_value = "LOCAL_ONLY")]
        privacy: String,
        #[arg(long)]
        peer: Option<String>,
        #[arg(long, default_value_t = 512)]
        max_tokens: u32,
    },
}
#[derive(Subcommand)]
enum GrantCommand {
    Issue {
        #[arg(long)]
        recipient: String,
        #[arg(long)]
        capability: String,
        #[arg(long)]
        resource: String,
        #[arg(long, default_value = "run")]
        action: String,
        #[arg(long, default_value_t = 3600)]
        ttl: u64,
        #[arg(long, default_value_t = 20)]
        max_calls: u64,
        #[arg(long, default_value_t = 20)]
        calls_per_hour: u64,
        #[arg(long, default_value_t = 65536)]
        max_input_bytes: u64,
        #[arg(long, default_value_t = 512)]
        max_output_tokens: u32,
        #[arg(long, default_value_t = 30000)]
        max_duration_ms: u64,
        #[arg(long,value_parser=parse_bytes,default_value="0")]
        max_storage_bytes: u64,
        #[arg(long)]
        porch: Option<String>,
        #[arg(long)]
        exact_input_digest: Option<String>,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Import {
        file: PathBuf,
    },
    Revoke {
        nonce: String,
    },
    List,
}
#[derive(Subcommand)]
enum StorageCommand {
    List,
    Put {
        file: PathBuf,
    },
    Get {
        cid: String,
        #[arg(long)]
        out: PathBuf,
    },
    Replicate {
        cid: String,
        peer: String,
    },
    Fetch {
        cid: String,
        peer: String,
        #[arg(long)]
        out: PathBuf,
    },
    Pin {
        cid: String,
        #[arg(long)]
        unpin: bool,
    },
    Delete {
        cid: String,
    },
    DeleteRemote {
        cid: String,
        peer: String,
    },
}
fn parse_bytes(s: &str) -> std::result::Result<u64, String> {
    let split = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    let n: u64 = s[..split]
        .parse()
        .map_err(|_| "Use an integer byte count or units such as 500MiB")?;
    let mult = match s[split..].to_ascii_lowercase().as_str() {
        "" | "b" => 1,
        "kb" => 1000,
        "mb" => 1000000,
        "gb" => 1000000000,
        "tb" => 1000000000000,
        "kib" => 1024,
        "mib" => 1024 * 1024,
        "gib" => 1024 * 1024 * 1024,
        _ => return Err("Unknown byte unit".into()),
    };
    n.checked_mul(mult)
        .ok_or_else(|| "Byte count overflow".into())
}
fn read_json(path: &PathBuf) -> Result<Value> {
    Ok(serde_json::from_slice(&std::fs::read(path)?)?)
}
fn write_json(path: &Path, value: &Value) -> Result<()> {
    private_write(path, &serde_json::to_vec_pretty(value)?)
}
async fn http(a: &Args, read: Option<&str>, operation: &str, args: Value) -> Result<Value> {
    let url = reqwest::Url::parse(&a.api)?;
    ensure!(
        url.scheme() == "http"
            && matches!(url.host_str(), Some("127.0.0.1" | "[::1]" | "::1"))
            && url.username().is_empty()
            && url.password().is_none()
            && url.path() == "/"
            && url.query().is_none()
            && url.fragment().is_none(),
        "CLI_API_MUST_BE_LOOPBACK"
    );
    let token = std::fs::read_to_string(
        a.token_file
            .clone()
            .unwrap_or_else(|| a.data.join("api.token")),
    )?;
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(60))
        .build()?;
    let response = if let Some(kind) = read {
        client.get(format!("{}/v1/{kind}", a.api.trim_end_matches('/')))
    } else {
        client
            .post(format!("{}/v1/control", a.api.trim_end_matches('/')))
            .json(&json!({"operation":operation,"args":args}))
    }
    .bearer_auth(token.trim())
    .send()
    .await?;
    let status = response.status();
    use futures::StreamExt;
    let mut stream = response.bytes_stream();
    let mut bytes = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        ensure!(
            bytes.len() + chunk.len() <= 32 * 1024 * 1024,
            "API_RESPONSE_TOO_LARGE"
        );
        bytes.extend(chunk);
    }
    let value: Value = serde_json::from_slice(&bytes)?;
    ensure!(
        status.is_success(),
        "{}",
        value["reason"].as_str().unwrap_or("API_REFUSED")
    );
    Ok(value)
}
#[tokio::main]
async fn main() -> Result<()> {
    let a = Args::parse();
    let mut export_dir = None;
    let mut save = None;
    let mut binary = None;
    let mut op = String::new();
    let mut args = json!({});
    let mut read = None;
    match &a.command {
        Command::ConfigureLimits { file } => {
            ensure!(
                a.data.join("identity.key").exists(),
                "INITIALIZE_NODE_FIRST"
            );
            let caps: porch_node::limits::LimitsConfig = serde_json::from_value(read_json(file)?)?;
            caps.validate()?;
            let mut c = Config::load(&a.data)?;
            c.limits = caps;
            let node = porch_node::Node::open(&a.data, c.clone())?;
            node.db.set("config", &serde_json::to_value(&c)?)?;
            write_json(&a.data.join("config.json"), &serde_json::to_value(&c)?)?;
            node.event(
                "operator.limits.changed",
                &node.id,
                json!({"config_digest":digest(&canonical(&c)?)}),
            )?;
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &json!({"saved":true,"restart_required":true,"limits":c.limits})
                )?
            );
            return Ok(());
        }
        Command::Qualify { command } => match command {
            QualifyCommand::Status => read = Some("qualification"),
            QualifyCommand::Host {
                name,
                recipient,
                out,
            } => {
                op = "qualification.host".into();
                args = json!({"name":name,"recipient":recipient});
                save = Some(out.clone());
            }
            QualifyCommand::Join {
                invite,
                fingerprint,
            } => {
                op = "qualification.join".into();
                args = json!({"invite":read_json(invite)?,"fingerprint":fingerprint});
            }
            QualifyCommand::Run {
                peer,
                model,
                environment,
                phase,
                separate_machines_confirmed,
                wan_condition_confirmed,
                message,
            } => {
                op = "qualification.run".into();
                args = json!({"peer":peer,"model":model,"environment":environment,"phase":phase,"separate_machines_confirmed":separate_machines_confirmed,"wan_condition_confirmed":wan_condition_confirmed,"message":message});
            }
            QualifyCommand::Export { directory } => {
                op = "qualification.export".into();
                export_dir = Some(directory.clone());
            }
            QualifyCommand::Validate { path } => {
                let value = porch_node::qualification::validate_export(
                    &porch_node::qualification::read_export(path)?,
                )?;
                println!("{}", serde_json::to_string_pretty(&value)?);
                return Ok(());
            }
        },
        Command::Init { alias } => {
            ensure!(
                !a.data.join("porch.sqlite").exists() || a.data.join("identity.key").exists(),
                "EXISTING_STATE_IDENTITY_MISSING_RESTORE_BACKUP"
            );
            let key = identity(&a.data)?;
            let c = Config {
                alias: alias.clone(),
                ..Config::default()
            };
            if !a.data.join("config.json").exists() {
                write_json(&a.data.join("config.json"), &serde_json::to_value(c)?)?;
            }
            if !a.data.join("api.token").exists() {
                private_write(
                    &a.data.join("api.token"),
                    hex::encode(random_bytes::<32>()).as_bytes(),
                )?;
            }
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &json!({"initialized":true,"peer":key.public().to_peer_id().to_string(),"fingerprint_sha256":digest(&key.public().encode_protobuf()),"next":"porch-node --data <this-directory>"})
                )?
            );
            return Ok(());
        }
        Command::Identity {
            command: IdentityCommand::Backup { out, password_env },
        } => {
            let password = std::env::var(password_env).context("SET_BACKUP_PASSWORD_ENV_FIRST")?;
            write_json(out, &protected_backup(&identity(&a.data)?, &password)?)?;
            println!("Protected backup created: {}", out.display());
            return Ok(());
        }
        Command::Identity {
            command: IdentityCommand::Restore { file, password_env },
        } => {
            ensure!(
                !a.data.join("identity.key").exists(),
                "RESTORE_REQUIRES_NEW_DATA_DIRECTORY"
            );
            std::fs::create_dir_all(&a.data)?;
            let key = restore_backup(&read_json(file)?, &std::env::var(password_env)?)?;
            private_write(&a.data.join("identity.key"), &key.to_protobuf_encoding()?)?;
            println!("Identity restored; network grants must be revalidated.");
            return Ok(());
        }
        Command::Identity {
            command: IdentityCommand::Rotate { new_data, out },
        } => {
            ensure!(
                !new_data.join("identity.key").exists(),
                "ROTATION_REQUIRES_NEW_DATA_DIRECTORY"
            );
            let old = identity(&a.data)?;
            let new = identity(new_data)?;
            let payload = json!({"old_peer":old.public().to_peer_id().to_string(),"new_peer":new.public().to_peer_id().to_string(),"nonce":nonce(),"created_at":now(),"expires_at":now()+86400,"authority_carried":false});
            let proof = json!({"old":Signed::new(&old,"porch.rotation.v1",payload.clone())?,"new":Signed::new(&new,"porch.rotation.v1",payload)?});
            write_json(out, &proof)?;
            println!(
                "Rotation proof created. Approve it on each peer with porch peer rotate; no grants transfer."
            );
            return Ok(());
        }
        Command::Status => read = Some("status"),
        Command::Peers => read = Some("peers"),
        Command::Resources => read = Some("resources"),
        Command::Jobs => read = Some("jobs"),
        Command::Ledger => read = Some("ledger"),
        Command::Grants => read = Some("grants"),
        Command::Network => read = Some("network"),
        Command::Doctor => read = Some("doctor"),
        Command::Identity {
            command: IdentityCommand::Show,
        } => read = Some("identity"),
        Command::Create { name } => {
            op = "porch.create".into();
            args = json!({"name":name});
        }
        Command::Invite {
            recipient,
            ttl,
            out,
        } => {
            op = "porch.invite".into();
            args = json!({"recipient":recipient,"ttl_seconds":ttl});
            save = out.clone();
        }
        Command::Join { invite } => {
            op = "porch.join".into();
            args = json!({"invite":read_json(invite)?});
        }
        Command::Leave => op = "porch.leave".into(),
        Command::Peer { command } => match command {
            PeerCommand::Approve { peer, alias } => {
                op = "peer.approve".into();
                args = json!({"peer":peer,"alias":alias});
            }
            PeerCommand::Revoke { peer } => {
                op = "peer.revoke".into();
                args = json!({"peer":peer});
            }
            PeerCommand::Connect { address } => {
                op = "peer.connect".into();
                args = json!({"address":address});
            }
            PeerCommand::Rotate { proof } => {
                op = "peer.rotate".into();
                args = read_json(proof)?;
            }
        },
        Command::Share { command } => {
            op = "share.configure".into();
            args = match command {
                ShareCommand::Model { name, private } => {
                    json!({"model":{"name":name,"provider":"ollama","exposed":!private}})
                }
                ShareCommand::Mock { name } => {
                    json!({"model":{"name":name,"provider":"mock","exposed":true}})
                }
                ShareCommand::Storage { limit } => json!({"storage_quota":limit}),
                ShareCommand::Compute { disable } => json!({"compute_hash":!disable}),
                ShareCommand::Refresh => {
                    op = "resources.refresh".into();
                    json!({})
                }
            };
        }
        Command::Model { command } => match command {
            ModelCommand::List => read = Some("models"),
            ModelCommand::Discover => op = "model.discover".into(),
            ModelCommand::Scan => op = "model.scan".into(),
            ModelCommand::Refresh => op = "model.refresh".into(),
            ModelCommand::Verify { model } => {
                op = "model.verify".into();
                args = json!({"model":model});
            }
            ModelCommand::Run {
                model,
                prompt,
                privacy,
                peer,
                max_tokens,
            } => {
                let _: Privacy = serde_json::from_value(json!(privacy))?;
                op = "job.run".into();
                args = json!({"resource":model,"input":prompt,"privacy":privacy,"preferred_peer":peer,"max_output_tokens":max_tokens});
            }
        },
        Command::Grant { command } => match command {
            GrantCommand::List => read = Some("grants"),
            GrantCommand::Import { file } => {
                op = "grant.import".into();
                args = json!({"grant":read_json(file)?});
            }
            GrantCommand::Revoke { nonce } => {
                op = "grant.revoke".into();
                args = json!({"nonce":nonce});
            }
            GrantCommand::Issue {
                recipient,
                capability,
                resource,
                action,
                ttl,
                max_calls,
                calls_per_hour,
                max_input_bytes,
                max_output_tokens,
                max_duration_ms,
                max_storage_bytes,
                porch,
                exact_input_digest,
                out,
            } => {
                op = "grant.issue".into();
                args = json!({"recipient":recipient,"capability":capability,"resource":resource,"action":action,"ttl_seconds":ttl,"porch":porch,"exact_input_digest":exact_input_digest,"limits":{"max_calls":max_calls,"calls_per_hour":calls_per_hour,"max_input_bytes":max_input_bytes,"max_output_tokens":max_output_tokens,"max_duration_ms":max_duration_ms,"max_storage_bytes":max_storage_bytes}});
                save = out.clone();
            }
        },
        Command::Storage { command } => match command {
            StorageCommand::List => read = Some("storage"),
            StorageCommand::Put { file } => {
                ensure!(
                    std::fs::metadata(file)?.len() <= (MAX_BLOB - 40) as u64,
                    "FILE_TOO_LARGE"
                );
                op = "storage.put".into();
                args = json!({"plaintext_hex":hex::encode(std::fs::read(file)?)});
            }
            StorageCommand::Get { cid, out } => {
                op = "storage.get".into();
                args = json!({"cid":cid});
                binary = Some(out.clone());
            }
            StorageCommand::Replicate { cid, peer } => {
                op = "storage.replicate".into();
                args = json!({"cid":cid,"peer":peer});
            }
            StorageCommand::Fetch { cid, peer, out } => {
                op = "storage.fetch".into();
                args = json!({"cid":cid,"peer":peer});
                binary = Some(out.clone());
            }
            StorageCommand::Pin { cid, unpin } => {
                op = "storage.pin".into();
                args = json!({"cid":cid,"pinned":!unpin});
            }
            StorageCommand::Delete { cid } => {
                op = "storage.delete".into();
                args = json!({"cid":cid});
            }
            StorageCommand::DeleteRemote { cid, peer } => {
                op = "storage.delete_remote".into();
                args = json!({"cid":cid,"peer":peer});
            }
        },
        Command::Message { peer, text, queue } => {
            op = "message.send".into();
            args = json!({"peer":peer,"text":text,"queue_if_offline":queue});
        }
        Command::Channel {
            channel,
            text,
            queue,
        } => {
            op = "channel.send".into();
            args = json!({"channel":channel,"text":text,"queue_if_offline":queue});
        }
        Command::RetryMessages => op = "message.retry".into(),
        Command::Resolve { address } => {
            op = "address.resolve".into();
            args = json!({"address":address});
        }
        Command::Call { operation, json } => {
            op = operation.clone();
            args = serde_json::from_str(json)?;
        }
    }
    let value = http(&a, read, &op, args).await?;
    if let Some(directory) = export_dir {
        porch_node::qualification::write_export(&directory, &value)?;
        println!(
            "{}",
            serde_json::to_string_pretty(
                &json!({"exported":directory,"validation":porch_node::qualification::validate_export(&value)?})
            )?
        );
    } else if let Some(out) = save {
        write_json(&out, &value)?;
        println!("Saved {}", out.display());
    } else if let Some(out) = binary {
        let bytes = hex::decode(
            value["plaintext_hex"]
                .as_str()
                .context("NO_PLAINTEXT_RESULT")?,
        )?;
        private_write(&out, &bytes)?;
        println!(
            "Verified and saved {} bytes to {}",
            bytes.len(),
            out.display()
        );
    } else {
        println!("{}", serde_json::to_string_pretty(&value)?);
    }
    if value["status"] == "REFUSED" {
        std::process::exit(2);
    }
    Ok(())
}
