// actor_ping_pong_network.rs

use std::{collections::HashMap, env, net::SocketAddr, sync::{Arc, Mutex}};
use tokio::{net::{TcpListener, TcpStream}, sync::mpsc::{unbounded_channel, UnboundedSender, UnboundedReceiver}, io::{AsyncReadExt, AsyncWriteExt}};
use serde::{Serialize, Deserialize};
use uuid::Uuid;


// ======== Network Protocol ========

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActorMeta {
    pub id: Uuid,
    pub addr: SocketAddr,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Start,
    Ping { sender: ActorMeta },
    Pong { sender: ActorMeta },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NetworkMessage {
    pub actor_id: Uuid,
    pub payload: Message,
}

// ======== Actor System definition/implementation ========

pub struct ActorSystem {
    registry: Arc<Mutex<HashMap<Uuid, UnboundedSender<Message>>>>,
    listener: TcpListener,
}

impl ActorSystem {
    pub async fn new(listen_addr: SocketAddr) -> Self {
        let listener = TcpListener::bind(listen_addr).await.expect("bind failed");
        Self { registry: Arc::new(Mutex::new(HashMap::new())), listener }
    }

    pub fn register(&self, id: Uuid, sender: UnboundedSender<Message>) {
        self.registry.lock().unwrap().insert(id, sender);
    }

    pub async fn start(self) {
        loop {
            let (mut socket, _) = self.listener.accept().await.expect("accept failed");
            let registry = self.registry.clone();
            tokio::spawn(async move {
                // Read length prefix
                let mut len_buf = [0u8; 4];
                if socket.read_exact(&mut len_buf).await.is_err() { return; }
                let len = u32::from_le_bytes(len_buf) as usize;
                let mut data = vec![0u8; len];
                if socket.read_exact(&mut data).await.is_err() { return; }
                // Deserialize and dispatch
                let net: NetworkMessage = bincode::deserialize(&data).expect("deserialize failed");
                if let Some(tx) = registry.lock().unwrap().get(&net.actor_id) {
                    let _ = tx.send(net.payload);
                }
            });
        }
    }
}

// ======== Actor definition/implementation ========

pub trait Actor: Send + 'static + Sized {
    type Msg: Send + 'static;
    fn receive(&mut self, msg: Self::Msg, ctx: &mut Context<Self>);
    fn send_message(&mut self, message:Message);
}

#[derive(Clone)]
pub struct Address<A: Actor> {
    channel_sender: UnboundedSender<A::Msg>,
}

impl<A: Actor> Address<A> {
    pub fn sender(&self) -> UnboundedSender<A::Msg> {
        self.channel_sender.clone()
    }
    pub async fn send(&self, msg: A::Msg) {
        let _ = self.channel_sender.send(msg);
    }
}

pub struct Context<A: Actor> {
    pub self_addr: Address<A>,
}

pub async fn start_actor<A: Actor>(mut actor: A) -> Address<A> {
    let (channel_sender, mut channel_receiver): (UnboundedSender<A::Msg>, UnboundedReceiver<A::Msg>) = unbounded_channel();
    let channel_sender_ctx = channel_sender.clone();
    let mut ctx = Context { self_addr: Address { channel_sender: channel_sender_ctx } };

    tokio::spawn(async move {
        while let Some(msg) = channel_receiver.recv().await {
            actor.receive(msg, &mut ctx);
        }
    });
    let addr = Address { channel_sender };
    print!("Actor running in Addr {:?}",addr.channel_sender);
    addr
}

pub struct RustActor {
    peer_meta: ActorMeta,
    self_meta: ActorMeta,
}

impl Actor for RustActor {
    type Msg = Message;
    fn receive(&mut self, msg: Self::Msg, _ctx: &mut Context<Self>) {
        match msg {
            Message::Start => {
                println!("RustActor: sending message to {}", self.peer_meta.addr);
                self.send_message(Message::Ping { sender: self.self_meta.clone() } );
            }
            Message::Ping { sender } => {
                println!("RustActor: received Ping from {}", sender.addr);
                self.send_message(Message::Pong { sender: self.self_meta.clone() } );

            }
            Message::Pong { sender } => {
                println!("RustActor: received Pong from {}", sender.addr);
                self.send_message(Message::Ping { sender: self.self_meta.clone() } );
            }
            _ => {}
        }
    }
    fn send_message(&mut self, message:Message) {
        let peer = self.peer_meta.clone();
        tokio::spawn(async move {
            let mut sock = TcpStream::connect(peer.addr).await.expect("connect failed");
            let net = NetworkMessage { actor_id: peer.id, payload: message};
            let buf = bincode::serialize(&net).expect("serialize failed");
            let len = (buf.len() as u32).to_le_bytes();
            sock.write_all(&len).await.unwrap();
            sock.write_all(&buf).await.unwrap();
        });
    }
}



// ======== Main Entrypoint ========

#[tokio::main]
async fn main() {
    let local_addr: SocketAddr =  "127.0.0.1:8000".parse().expect("invalid local_addr");
    let peer_addr: SocketAddr = "127.0.0.1:8001".parse().expect("invalid peer_addr");

    // Predefined IDs for Ping and Pong actors
    let ping_id = Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
    let pong_id = Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap();

    let actor_system = ActorSystem::new(local_addr).await;
    let ping_meta = ActorMeta { id: ping_id, addr: local_addr };
    let pong_meta = ActorMeta { id: pong_id, addr: peer_addr };
    let pong_addr = start_actor(RustActor { peer_meta: ping_meta.clone(), self_meta: pong_meta.clone() }).await;
    let ping_addr = start_actor(RustActor { peer_meta: pong_meta.clone(), self_meta: ping_meta.clone() }).await;
    actor_system.register(pong_meta.id, ping_addr.sender());
    actor_system.register(ping_meta.id, pong_addr.sender());
    tokio::spawn(actor_system.start());

    // Trigger start
    ping_addr.send(Message::Start).await;
    // Keep the program alive
    loop { tokio::time::sleep(std::time::Duration::from_secs(60)).await; }
    
    
    // // Start node to listen & dispatch
    // let node = Node::new(local_addr).await;
    // 
    // match role.as_str() {
    //     "ping" => {
    //         let ping_meta = ActorMeta { id: ping_id, addr: local_addr };
    //         let pong_meta = ActorMeta { id: pong_id, addr: peer_addr };
    //         let ping_addr = start_actor(PingActor { peer_meta: pong_meta, self_meta: ping_meta.clone() }).await;
    //         node.register(ping_meta.id, ping_addr.sender());
    //         // Spawn listener
    //         tokio::spawn(node.start());
    //         // Trigger start
    //         ping_addr.send(Message::Start).await;
    //     }
    //     "pong" => {
    //         let pong_meta = ActorMeta { id: pong_id, addr: local_addr };
    //         let pong_addr = start_actor(PongActor { self_meta: pong_meta.clone() }).await;
    //         node.register(pong_meta.id, pong_addr.sender());
    //         tokio::spawn(node.start());
    //     }
    //     _ => {
    //         eprintln!("Role must be 'ping' or 'pong'");
    //     }
    // }


}
