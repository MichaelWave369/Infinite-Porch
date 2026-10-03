use crate::Node;
use anyhow::{Context, Result, ensure};
use futures::StreamExt;
use libp2p::{
    Multiaddr, PeerId, StreamProtocol, SwarmBuilder, connection_limits, identify, mdns, noise,
    ping, request_response,
    swarm::{NetworkBehaviour, SwarmEvent, behaviour::toggle::Toggle},
    tcp, yamux,
};
use porch_core::*;
use serde_json::{Value, json};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::{mpsc, oneshot};

type Request = Signed<WireRequest>;
type Response = Signed<WireResponse>;
#[derive(NetworkBehaviour)]
struct Behaviour {
    rpc: request_response::json::Behaviour<Request, Response>,
    mdns: Toggle<mdns::tokio::Behaviour>,
    identify: identify::Behaviour,
    ping: ping::Behaviour,
    limits: connection_limits::Behaviour,
}
enum Command {
    Request(PeerId, Request, oneshot::Sender<Result<Response>>),
    Reply(request_response::ResponseChannel<Response>, Response),
    Connect(PeerId, Multiaddr),
    Disconnect(PeerId),
}
#[derive(Clone)]
pub struct Handle {
    tx: mpsc::Sender<Command>,
}
/// An installed transport supplies authenticated peer paths, not authority.
/// Alternative drivers still exchange the same signed, versioned Porch frames.
pub trait Transport: Send + Sync {
    fn frame_limit(&self) -> usize;
    fn request<'a>(
        &'a self,
        peer: &'a str,
        request: Request,
    ) -> futures::future::BoxFuture<'a, Result<Response>>;
    fn connect(&self, address: &str, expected: Option<&str>) -> Result<()>;
    fn disconnect(&self, peer: &str) -> Result<()>;
}
impl Transport for Handle {
    fn frame_limit(&self) -> usize {
        MAX_FRAME
    }
    fn request<'a>(
        &'a self,
        peer: &'a str,
        request: Request,
    ) -> futures::future::BoxFuture<'a, Result<Response>> {
        Box::pin(Handle::request(self, peer, request))
    }
    fn connect(&self, address: &str, expected: Option<&str>) -> Result<()> {
        Handle::connect(self, address, expected)
    }
    fn disconnect(&self, peer: &str) -> Result<()> {
        Handle::disconnect(self, peer)
    }
}
impl Handle {
    pub async fn request(&self, peer: &str, request: Request) -> Result<Response> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .try_send(Command::Request(peer.parse()?, request, tx))
            .context("NETWORK_QUEUE_FULL")?;
        tokio::time::timeout(Duration::from_secs(35), rx)
            .await
            .context("PEER_REQUEST_TIMEOUT")?
            .context("NETWORK_STOPPED")?
    }
    pub fn connect(&self, address: &str, expected: Option<&str>) -> Result<()> {
        let mut addr: Multiaddr = address.parse()?;
        let Some(libp2p::multiaddr::Protocol::P2p(peer)) = addr.pop() else {
            anyhow::bail!("ADDRESS_REQUIRES_PINNED_PEER_ID");
        };
        if let Some(expected) = expected {
            ensure!(peer.to_string() == expected, "ADDRESS_IDENTITY_MISMATCH");
        }
        ensure!(
            addr.iter().any(|p| matches!(
                p,
                libp2p::multiaddr::Protocol::Ip4(_) | libp2p::multiaddr::Protocol::Ip6(_)
            )),
            "IP_TRANSPORT_REQUIRED"
        );
        self.tx
            .try_send(Command::Connect(peer, addr))
            .context("NETWORK_QUEUE_FULL")?;
        Ok(())
    }
    pub fn disconnect(&self, peer: &str) -> Result<()> {
        self.tx
            .try_send(Command::Disconnect(peer.parse()?))
            .context("NETWORK_QUEUE_FULL")?;
        Ok(())
    }
}
pub async fn start(node: Arc<Node>) -> Result<Handle> {
    let cfg = node.config.read().unwrap().clone();
    let key = node.key.clone();
    let id = node.key.public().to_peer_id();
    let mdns = if cfg.mdns {
        Some(mdns::tokio::Behaviour::new(mdns::Config::default(), id)?)
    } else {
        None
    };
    let mut swarm = SwarmBuilder::with_existing_identity(key)
        .with_tokio()
        .with_tcp(
            tcp::Config::default().nodelay(true),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|key| {
            let codec = request_response::json::codec::Codec::<Request, Response>::default()
                .set_request_size_maximum(MAX_FRAME as u64)
                .set_response_size_maximum(MAX_FRAME as u64);
            Behaviour {
                rpc: request_response::Behaviour::with_codec(
                    codec,
                    [(
                        StreamProtocol::new("/infinite-porch/rpc/1"),
                        request_response::ProtocolSupport::Full,
                    )],
                    request_response::Config::default()
                        .with_request_timeout(Duration::from_secs(33))
                        .with_max_concurrent_streams(32),
                ),
                mdns: Toggle::from(mdns),
                identify: identify::Behaviour::new(
                    identify::Config::new("/infinite-porch/1".into(), key.public())
                        .with_agent_version("porch-node/0.1.0".into()),
                ),
                ping: ping::Behaviour::new(
                    ping::Config::new().with_interval(Duration::from_secs(15)),
                ),
                limits: connection_limits::Behaviour::new(
                    connection_limits::ConnectionLimits::default()
                        .with_max_pending_incoming(Some(16))
                        .with_max_pending_outgoing(Some(16))
                        .with_max_established(Some(64))
                        .with_max_established_per_peer(Some(2)),
                ),
            }
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(120)))
        .build();
    for address in &cfg.listen {
        swarm.listen_on(address.parse()?)?;
    }
    let (tx, mut rx) = mpsc::channel(64);
    let handle = Handle { tx: tx.clone() };
    *node.network.write().unwrap() = Some(Arc::new(handle.clone()));
    tokio::spawn(async move {
        let mut pending: HashMap<
            request_response::OutboundRequestId,
            oneshot::Sender<Result<Response>>,
        > = HashMap::new();
        loop {
            tokio::select! {
                Some(command)=rx.recv()=>match command {
                    Command::Request(peer,request,reply)=>{if pending.len()>=64{let _=reply.send(Err(anyhow::anyhow!("OUTBOUND_QUEUE_FULL")));}else{let id=swarm.behaviour_mut().rpc.send_request(&peer,request);pending.insert(id,reply);}},
                    Command::Reply(channel,response)=>{let _=swarm.behaviour_mut().rpc.send_response(channel,response);},
                    Command::Connect(peer,address)=>{
                        if node.db.trust_rows().ok().is_some_and(|rows|rows.iter().any(|r|r["peer"].as_str()==Some(&peer.to_string()) && r["active"]==false)){continue;}
                        swarm.add_peer_address(peer,address.clone());
                        let _=swarm.dial(address.with(libp2p::multiaddr::Protocol::P2p(peer)));
                    },
                    Command::Disconnect(peer)=>{let _=swarm.disconnect_peer_id(peer);},
                },
                event=swarm.select_next_some()=>match event {
                    SwarmEvent::NewListenAddr{address,..}=>{
                        let full=address.with(libp2p::multiaddr::Protocol::P2p(id)).to_string();let mut o=node.observations.write().unwrap();
                        if let Some(a)=o["addresses"].as_array_mut()&& !a.contains(&json!(full)) && a.len()<32{a.push(json!(full));}
                    },
                    SwarmEvent::ConnectionEstablished{peer_id,endpoint,..}=>{
                        let peer=peer_id.to_string();let mut o=node.observations.write().unwrap();
                        if o["peers"].as_object().is_some_and(|p|p.len()<128){o["peers"][&peer]=json!({"connected":true,"transport":if endpoint.get_remote_address().to_string().contains("quic"){"QUIC"}else{"TCP/Noise"},"latency_ms":null});}drop(o);
                        let _=node.event("peer.connected",&peer,json!({}));
                    },
                    SwarmEvent::ConnectionClosed{peer_id,num_established,..}=>{if num_established==0{if let Some(p)=node.observations.write().unwrap()["peers"].get_mut(peer_id.to_string()){p["connected"]=json!(false);}let _=node.event("peer.disconnected",&peer_id.to_string(),json!({}));}},
                    SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Discovered(peers)))=>{
                        for (peer,addr) in peers {
                            if node.db.trusted(&peer.to_string()).unwrap_or(false){swarm.add_peer_address(peer,addr.clone());let _=swarm.dial(addr.with(libp2p::multiaddr::Protocol::P2p(peer)));}
                        }
                    },
                    SwarmEvent::Behaviour(BehaviourEvent::Ping(ping::Event{peer,result:Ok(rtt),..}))=>{if let Some(p)=node.observations.write().unwrap()["peers"].get_mut(peer.to_string()){p["latency_ms"]=json!(rtt.as_millis() as u64);}},
                    SwarmEvent::Behaviour(BehaviourEvent::Rpc(request_response::Event::Message{peer,message,..}))=>match message {
                        request_response::Message::Request{request,channel,..}=>{
                            let Ok(permit)=node.ingress.clone().try_acquire_owned() else {let response=Signed::new(&node.key,"porch.response.v1",WireResponse{request_nonce:request.payload.nonce.clone(),request_digest:request.hash().unwrap_or_default(),status:"REFUSED".into(),code:"INGRESS_BUSY".into(),output:Value::Null,timestamp:now()});if let Ok(r)=response{let _=swarm.behaviour_mut().rpc.send_response(channel,r);}continue;};
                            let node=node.clone();let tx=tx.clone();tokio::spawn(async move{let _permit=permit;match node.receive(peer.to_string(),request).await{Ok(response)=>{let _=tx.send(Command::Reply(channel,response)).await;},Err(e)=>tracing::warn!(reason=%e,"request verification failed")}});
                        },
                        request_response::Message::Response{request_id,response}=>{if let Some(reply)=pending.remove(&request_id){let _=reply.send(Ok(response));}},
                    },
                    SwarmEvent::Behaviour(BehaviourEvent::Rpc(request_response::Event::OutboundFailure{request_id,error,..}))=>{if let Some(reply)=pending.remove(&request_id){let _=reply.send(Err(anyhow::anyhow!("PEER_TRANSPORT_FAILURE:{error}")));}},
                    SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Received{peer_id,info,..}))
                        if node.db.trusted(&peer_id.to_string()).unwrap_or(false) && info.protocol_version=="/infinite-porch/1"=> {for address in info.listen_addrs.into_iter().take(16){swarm.add_peer_address(peer_id,address);}},
                    _=>{},
                }
            }
        }
    });
    Ok(handle)
}
