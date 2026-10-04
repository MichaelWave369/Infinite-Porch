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
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{mpsc, oneshot};

type Request = Signed<WireRequest>;
type Response = Signed<WireResponse>;
#[derive(NetworkBehaviour)]
struct Behaviour {
    rpc: request_response::Behaviour<crate::wire_codec::BoundedJsonCodec>,
    mdns: Toggle<mdns::tokio::Behaviour>,
    identify: identify::Behaviour,
    ping: ping::Behaviour,
    limits: connection_limits::Behaviour,
}
enum Command {
    Request(PeerId, Request, oneshot::Sender<Result<Response>>),
    Reply(request_response::ResponseChannel<Response>, Response),
    Connect(PeerId, Vec<Multiaddr>),
    Disconnect(PeerId),
}
#[derive(Clone)]
pub struct Handle {
    tx: mpsc::Sender<Command>,
    frame_limit: usize,
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
    fn connect_many(&self, addresses: &[String], expected: Option<&str>) -> Result<()> {
        for address in addresses {
            self.connect(address, expected)?;
        }
        Ok(())
    }
    fn disconnect(&self, peer: &str) -> Result<()>;
}
impl Transport for Handle {
    fn frame_limit(&self) -> usize {
        self.frame_limit
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
    fn connect_many(&self, addresses: &[String], expected: Option<&str>) -> Result<()> {
        Handle::connect_many(self, addresses, expected)
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
        self.connect_many(&[address.to_string()], expected)
    }
    pub fn connect_many(&self, addresses: &[String], expected: Option<&str>) -> Result<()> {
        ensure!(
            !addresses.is_empty() && addresses.len() <= 16,
            "ADDRESS_COUNT_LIMIT"
        );
        let mut peer = None;
        let mut parsed = Vec::new();
        for address in addresses {
            let mut addr: Multiaddr = address.parse()?;
            let Some(libp2p::multiaddr::Protocol::P2p(id)) = addr.pop() else {
                anyhow::bail!("ADDRESS_REQUIRES_PINNED_PEER_ID")
            };
            ensure!(
                expected.is_none_or(|v| id.to_string() == v) && peer.is_none_or(|v| v == id),
                "ADDRESS_IDENTITY_MISMATCH"
            );
            ensure!(
                addr.iter().any(|p| matches!(
                    p,
                    libp2p::multiaddr::Protocol::Ip4(_) | libp2p::multiaddr::Protocol::Ip6(_)
                )),
                "IP_TRANSPORT_REQUIRED"
            );
            peer = Some(id);
            if !parsed.contains(&addr) {
                parsed.push(addr);
            }
        }
        // A physical invitation can contain the creator's loopback address.
        // Prefer routable local destinations, retaining every pinned fallback.
        parsed.sort_by_key(|a| match classify_path(&a.to_string(), false) {
            "LAN_DIRECT" => 0,
            "WAN_DIRECT" => 1,
            "LOOPBACK" => 2,
            _ => 3,
        });
        self.tx
            .try_send(Command::Connect(
                peer.context("ADDRESS_COUNT_LIMIT")?,
                parsed,
            ))
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
            let codec = crate::wire_codec::BoundedJsonCodec::new(cfg.limits.peer_frame_bytes);
            Behaviour {
                rpc: request_response::Behaviour::with_codec(
                    codec,
                    [(
                        StreamProtocol::new("/infinite-porch/rpc/1"),
                        request_response::ProtocolSupport::Full,
                    )],
                    request_response::Config::default()
                        .with_request_timeout(Duration::from_secs(33))
                        .with_max_concurrent_streams(cfg.limits.ingress_requests),
                ),
                mdns: Toggle::from(mdns),
                identify: identify::Behaviour::new(
                    identify::Config::new("/infinite-porch/1".into(), key.public())
                        .with_agent_version(format!("porch-node/{}", env!("CARGO_PKG_VERSION"))),
                ),
                ping: ping::Behaviour::new(
                    ping::Config::new().with_interval(Duration::from_secs(15)),
                ),
                limits: connection_limits::Behaviour::new(
                    connection_limits::ConnectionLimits::default()
                        .with_max_pending_incoming(Some(16))
                        .with_max_pending_outgoing(Some(16))
                        .with_max_established(Some(cfg.limits.connections))
                        .with_max_established_per_peer(Some(2)),
                ),
            }
        })?
        .with_swarm_config(|c| {
            c.with_idle_connection_timeout(Duration::from_secs(cfg.limits.idle_peer_seconds))
        })
        .build();
    for address in &cfg.listen {
        swarm.listen_on(address.parse()?)?;
    }
    // Reload only operator-pinned addresses belonging to still-trusted identities.
    for row in node
        .db
        .trust_rows()?
        .into_iter()
        .filter(|r| r["active"] == true)
    {
        if let Some(peer) = row["peer"].as_str()
            && let Ok(id) = peer.parse::<PeerId>()
            && let Some(saved) = node.db.get(&format!("peer_addresses:{peer}"))?
        {
            for address in saved
                .as_array()
                .into_iter()
                .flatten()
                .take(16)
                .filter_map(Value::as_str)
            {
                if let Ok(mut addr) = address.parse::<Multiaddr>()
                    && addr.pop() == Some(libp2p::multiaddr::Protocol::P2p(id))
                {
                    swarm.add_peer_address(id, addr);
                }
            }
        }
    }
    let (tx, mut rx) = mpsc::channel(64);
    let handle = Handle {
        tx: tx.clone(),
        frame_limit: cfg.limits.peer_frame_bytes,
    };
    *node.network.write().unwrap() = Some(Arc::new(handle.clone()));
    tokio::spawn(async move {
        let mut pending: HashMap<
            request_response::OutboundRequestId,
            oneshot::Sender<Result<Response>>,
        > = HashMap::new();
        let mut connected_at: HashMap<libp2p::swarm::ConnectionId, Instant> = HashMap::new();
        let mut peer_connections: HashMap<String, HashMap<libp2p::swarm::ConnectionId, Value>> =
            HashMap::new();
        let mut age_tick = tokio::time::interval(Duration::from_secs(1));
        loop {
            tokio::select! {
                _=age_tick.tick()=>{
                    let mut o=node.observations.write().unwrap();
                    for (peer,connections) in &mut peer_connections {
                        for (id,value) in connections.iter_mut() {value["connection_age_seconds"]=json!(connected_at.get(id).map(|t|t.elapsed().as_secs()));}
                        if let Some(row)=o["peers"].get_mut(peer) {
                            if let Some(value)=connections.values().find(|v|v["connection_id"]==row["connection_id"]) {row["connection_age_seconds"]=value["connection_age_seconds"].clone();}
                            row["connections"]=json!(connections.values().collect::<Vec<_>>());
                        }
                    }
                },
                Some(command)=rx.recv()=>match command {
                    Command::Request(peer,request,reply)=>{if pending.len()>=64{let _=reply.send(Err(anyhow::anyhow!("OUTBOUND_QUEUE_FULL")));}else{let id=swarm.behaviour_mut().rpc.send_request(&peer,request);pending.insert(id,reply);}},
                    Command::Reply(channel,response)=>{let _=swarm.behaviour_mut().rpc.send_response(channel,response);},
                    Command::Connect(peer,addresses)=>{
                        if node.db.trust_rows().ok().is_some_and(|rows|rows.iter().any(|r|r["peer"].as_str()==Some(&peer.to_string()) && r["active"]==false)){continue;}
                        for address in &addresses {swarm.add_peer_address(peer,address.clone());}
                        if node.db.trusted(&peer.to_string()).unwrap_or(false) { let _=node.db.set(&format!("peer_addresses:{peer}"),&json!(addresses.iter().map(|a|a.clone().with(libp2p::multiaddr::Protocol::P2p(peer)).to_string()).collect::<Vec<_>>())); }
                        let _=swarm.dial(libp2p::swarm::dial_opts::DialOpts::peer_id(peer).condition(libp2p::swarm::dial_opts::PeerCondition::DisconnectedAndNotDialing).addresses(addresses).build());
                    },
                    Command::Disconnect(peer)=>{let _=swarm.disconnect_peer_id(peer);},
                },
                event=swarm.select_next_some()=>match event {
                    SwarmEvent::NewListenAddr{address,..}=>{
                        let full=address.with(libp2p::multiaddr::Protocol::P2p(id)).to_string();let mut o=node.observations.write().unwrap();
                        if let Some(a)=o["addresses"].as_array_mut()&& !a.contains(&json!(full)) && a.len()<32{a.push(json!(full));}
                    },
                    SwarmEvent::ConnectionEstablished{peer_id,endpoint,connection_id,..}=>{
                        let peer=peer_id.to_string();let remote=endpoint.get_remote_address().to_string();
                        let local=match &endpoint {libp2p::core::ConnectedPoint::Listener{local_addr,..}=>Some(local_addr.to_string()),_=>None};
                        let relay=remote.contains("/p2p-circuit");let path=classify_path(&remote,relay);
                        let value=json!({"connection_id":connection_id.to_string(),"connected":true,"transport":if remote.contains("quic"){"QUIC"}else{"TCP/Noise"},"encryption":if remote.contains("quic"){"QUIC TLS 1.3"}else{"Noise XX"},"encrypted":true,"authenticated":true,"authenticated_peer":peer,"local_socket":local,"remote_socket":remote,"negotiated_protocol":null,"direct":!relay,"relayed":relay,"path":path,"path_basis":"authenticated transport IP classification; LAN topology/WAN state require operator evidence","connection_age_seconds":0,"latency_ms":null});
                        connected_at.insert(connection_id,Instant::now());
                        peer_connections.entry(peer.clone()).or_default().insert(connection_id,value.clone());
                        let mut o=node.observations.write().unwrap();
                        if let Some(rows)=o["peers"].as_object_mut() {
                            if rows.len()>=128 && !rows.contains_key(&peer) && let Some(expired)=rows.iter().find(|(_,v)|v["connected"]==false).map(|(id,_)|id.clone()){rows.remove(&expired);}
                            if rows.len()<128 || rows.contains_key(&peer){rows.insert(peer.clone(),value);rows[&peer]["connections"]=json!(peer_connections[&peer].values().collect::<Vec<_>>());}
                        }drop(o);
                        let _=node.event("peer.connected",&peer,json!({}));
                    },
                    SwarmEvent::ConnectionClosed{peer_id,connection_id,num_established,..}=>{
                        connected_at.remove(&connection_id);let peer=peer_id.to_string();
                        if let Some(c)=peer_connections.get_mut(&peer){c.remove(&connection_id);}
                        if num_established==0 {peer_connections.remove(&peer);}
                        let mut o=node.observations.write().unwrap();
                        if num_established==0 { if let Some(p)=o["peers"].get_mut(&peer){p["connected"]=json!(false);p["authenticated"]=json!(false);p["connection_age_seconds"]=Value::Null;p["connections"]=json!([]);} }
                        else if let Some(v)=peer_connections.get(&peer).and_then(|c|c.values().next()){o["peers"][&peer]=v.clone();o["peers"][&peer]["connections"]=json!(peer_connections[&peer].values().collect::<Vec<_>>());}
                        drop(o);if num_established==0{let _=node.event("peer.disconnected",&peer,json!({}));}
                    },
                    SwarmEvent::ExpiredListenAddr{address,..}=>{let full=address.with(libp2p::multiaddr::Protocol::P2p(id)).to_string();if let Some(a)=node.observations.write().unwrap()["addresses"].as_array_mut(){a.retain(|v|v!=&full);}},
                    SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Discovered(peers)))=>{
                        for (peer,addr) in peers {
                            if node.db.trusted(&peer.to_string()).unwrap_or(false){swarm.add_peer_address(peer,addr.clone());let _=swarm.dial(libp2p::swarm::dial_opts::DialOpts::peer_id(peer).condition(libp2p::swarm::dial_opts::PeerCondition::DisconnectedAndNotDialing).addresses(vec![addr]).build());}
                        }
                    },
                    SwarmEvent::Behaviour(BehaviourEvent::Ping(ping::Event{peer,connection,result:Ok(rtt),..}))=>{
                        if let Some(value)=peer_connections.get_mut(&peer.to_string()).and_then(|c|c.get_mut(&connection)){value["latency_ms"]=json!(rtt.as_millis() as u64);}
                        if let Some(p)=node.observations.write().unwrap()["peers"].get_mut(peer.to_string()) && p["connection_id"]==connection.to_string(){p["latency_ms"]=json!(rtt.as_millis() as u64);}
                    },
                    SwarmEvent::Behaviour(BehaviourEvent::Rpc(request_response::Event::Message{peer,message,connection_id}))=>{
                        if let Some(connections)=peer_connections.get_mut(&peer.to_string()) {
                            if let Some(value)=connections.get_mut(&connection_id) {value["negotiated_protocol"]=json!("/infinite-porch/rpc/1");node.observations.write().unwrap()["peers"][peer.to_string()]=value.clone();}
                            node.observations.write().unwrap()["peers"][peer.to_string()]["connections"]=json!(connections.values().collect::<Vec<_>>());
                        }
                        match message {
                        request_response::Message::Request{request,channel,..}=>{
                            let Ok(permit)=node.ingress.clone().try_acquire_owned() else {let response=Signed::new(&node.key,"porch.response.v1",WireResponse{request_nonce:request.payload.nonce.clone(),request_digest:request.hash().unwrap_or_default(),status:"REFUSED".into(),code:"INGRESS_BUSY".into(),output:Value::Null,timestamp:now()});if let Ok(r)=response{let _=swarm.behaviour_mut().rpc.send_response(channel,r);}continue;};
                            let node=node.clone();let tx=tx.clone();tokio::spawn(async move{let _permit=permit;match node.receive(peer.to_string(),request).await{Ok(response)=>{let _=tx.send(Command::Reply(channel,response)).await;},Err(e)=>tracing::warn!(reason=%crate::safe_error(&e),"request verification failed")}});
                        },
                        request_response::Message::Response{request_id,response}=>{
                        if let Some(reply)=pending.remove(&request_id){let _=reply.send(Ok(response));}},
                    }},
                    SwarmEvent::Behaviour(BehaviourEvent::Rpc(request_response::Event::OutboundFailure{request_id,error,..}))=>{if let Some(reply)=pending.remove(&request_id){let _=reply.send(Err(anyhow::anyhow!("PEER_TRANSPORT_FAILURE:{error}")));}},
                    SwarmEvent::Behaviour(BehaviourEvent::Rpc(request_response::Event::InboundFailure{..}))=>{},
                    SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Received{peer_id,info,..}))
                        if node.db.trusted(&peer_id.to_string()).unwrap_or(false) && info.protocol_version=="/infinite-porch/1"=> {let mut saved=Vec::new();for address in info.listen_addrs.into_iter().take(16){if address.iter().any(|p|matches!(p,libp2p::multiaddr::Protocol::Ip4(_) | libp2p::multiaddr::Protocol::Ip6(_))){saved.push(address.clone().with(libp2p::multiaddr::Protocol::P2p(peer_id)).to_string());swarm.add_peer_address(peer_id,address);}}let _=node.db.set(&format!("peer_addresses:{peer_id}"),&json!(saved));},
                    _=>{},
                }
            }
        }
    });
    Ok(handle)
}

/// Classifies the observed direct destination, without proving physical topology.
pub fn classify_path(address: &str, relayed: bool) -> &'static str {
    if relayed {
        return "RELAY";
    }
    let Ok(addr) = address.parse::<Multiaddr>() else {
        return "UNKNOWN";
    };
    for p in addr.iter() {
        match p {
            libp2p::multiaddr::Protocol::Ip4(ip) => {
                return if ip.is_loopback() {
                    "LOOPBACK"
                } else if ip.is_unspecified() {
                    "UNKNOWN"
                } else if ip.is_private() || ip.is_link_local() {
                    "LAN_DIRECT"
                } else {
                    "WAN_DIRECT"
                };
            }
            libp2p::multiaddr::Protocol::Ip6(ip) => {
                return if ip.is_loopback() {
                    "LOOPBACK"
                } else if ip.is_unspecified() {
                    "UNKNOWN"
                } else if (ip.segments()[0] & 0xfe00) == 0xfc00
                    || (ip.segments()[0] & 0xffc0) == 0xfe80
                {
                    "LAN_DIRECT"
                } else {
                    "WAN_DIRECT"
                };
            }
            _ => {}
        }
    }
    "UNKNOWN"
}
pub fn diagnostics(node: &Node) -> Value {
    let mut o = node.observations.read().unwrap().clone();
    o["local_peer_id"] = json!(node.id);
    o["local_socket_limitation"] = json!(
        "Inbound socket observed; outbound ephemeral socket unavailable from this libp2p endpoint API."
    );
    if let Some(peers) = o["peers"].as_object_mut() {
        for (id, v) in peers {
            v["remote_peer_id"] = json!(id);
            v["trust_state"] = json!(if node.db.trusted(id).unwrap_or(false) {
                "TRUSTED"
            } else {
                "UNTRUSTED_OR_REVOKED"
            });
            v["physical_topology_verified"] = json!(false);
        }
    }
    o
}
