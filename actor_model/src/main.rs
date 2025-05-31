use std::{collections::HashMap, env, net::SocketAddr, sync::{Arc, Mutex}};
use tokio::{net::{TcpListener, TcpStream}, sync::mpsc::{unbounded_channel, UnboundedSender, UnboundedReceiver}, io::{AsyncReadExt, AsyncWriteExt}};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

// ======== Actor System with TCP Networking (Rust, Tokio) ========
/// This code implements a basic actor system with message passing over TCP.
/// Actors are registered with unique IDs, can send/receive messages, and communicate locally or across the network.
/// PingActor and PongActor demonstrate the message loop between two actors.

// ======== Network Protocol ========

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActorMeta {
    pub id: Uuid,
    pub addr: SocketAddr,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Ping { sender: ActorMeta },
    Pong { sender: ActorMeta },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NetworkMessage {
    pub actor_id: Uuid,
    pub payload: Message,
}

// ======== Actor System definition/implementation ========

/// Core struct that maintains all registered actors and listens for incoming TCP messages.
/// Dispatches messages to local actors by their UUID.
pub struct ActorSystem {
    actors: Arc<Mutex<HashMap<Uuid, UnboundedSender<Message>>>>,
    listener: TcpListener,
}

impl ActorSystem {
    /// Creates a new ActorSystem, binding the TCP listener to the provided address.
    pub async fn new(listen_addr: SocketAddr) -> Self {
        let listener = TcpListener::bind(listen_addr).await.expect("bind failed");
        Self { actors: Arc::new(Mutex::new(HashMap::new())), listener }
    }

    /// Registers an actor with its UUID and a sender channel for messages.
    pub fn register(&self, id: Uuid, sender: UnboundedSender<Message>) {
        self.actors.lock().unwrap().insert(id, sender);
    }

    /// Starts accepting TCP connections and dispatching incoming messages
    /// to the appropriate local actor by UUID.
    pub async fn start(self) {
        loop {
            let (mut socket, _) = self.listener.accept().await.expect("accept failed");
            let registry = self.actors.clone();
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

/// Spawns an async task that processes messages for the actor.
/// Returns the address to send messages to this actor.
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
    print!("Actor running in Addr {:?} \n", addr.channel_sender);
    addr
}

pub struct PingActor {
    actor_meta: ActorMeta,
}

pub struct PongActor {
    actor_meta: ActorMeta,
}

impl Actor for PingActor {
    type Msg = Message;
    /// On receiving Ping, print sender and reply with Pong.
    fn receive(&mut self, msg: Self::Msg, _ctx: &mut Context<Self>) {
        match msg {
            Message::Ping { mut sender } => {
                println!("RustActor: received Ping from {}", sender.addr);
                let message = Message::Pong { sender: self.actor_meta.clone() };
                send_message(&mut sender, message);
            }
            _ => {}
        }
    }
}

impl Actor for PongActor {
    type Msg = Message;
    /// On receiving Pong, print sender and reply with Ping.
    fn receive(&mut self, msg: Self::Msg, _ctx: &mut Context<Self>) {
        match msg {
            Message::Pong { mut sender } => {
                println!("RustActor: received Pong from {}", sender.addr);
                let message = Message::Ping { sender: self.actor_meta.clone() };
                send_message(&mut sender, message);
            }
            _ => {}
        }
    }
}

/// Sends a message to another actor via TCP.
/// Used by both PingActor and PongActor for remote calls.
fn send_message(actor_meta: &mut ActorMeta, message: Message) {
    let peer = actor_meta.clone();
    tokio::spawn(async move {
        let mut sock = TcpStream::connect(peer.addr).await.expect("connect failed");
        let net = NetworkMessage { actor_id: peer.id, payload: message };
        let buf = bincode::serialize(&net).expect("serialize failed");
        let len = (buf.len() as u32).to_le_bytes();
        sock.write_all(&len).await.unwrap();
        sock.write_all(&buf).await.unwrap();
    });
}


// ======== Main Entrypoint ========

/// Bootstraps the actor system, registers Ping and Pong actors,
/// starts the listener, and triggers the first Ping message.
/// Keeps the process alive indefinitely.
#[tokio::main]
async fn main() {
    let local_addr: SocketAddr = "127.0.0.1:8000".parse().expect("invalid local_addr");

    // Predefined IDs for Ping and Pong actors
    let ping_id = Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
    let pong_id = Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap();

    let actor_system = ActorSystem::new(local_addr).await;

    //Ping actor
    let ping_meta = ActorMeta { id: ping_id, addr: local_addr };
    let ping_actor = PingActor { actor_meta: ping_meta.clone()  };
    let ping_addr = start_actor(ping_actor).await;
    actor_system.register(ping_meta.id, ping_addr.sender());

    //Pong actor
    let pong_meta = ActorMeta { id: pong_id, addr: local_addr };
    let pong_actor = PongActor { actor_meta: pong_meta.clone() };
    let pong_addr = start_actor(pong_actor).await;
    actor_system.register(pong_meta.id, pong_addr.sender());

    tokio::spawn(actor_system.start());

    // Trigger start
    let message = Message::Ping { sender: pong_meta.clone() };
    ping_addr.send(message).await;
    // Keep the program alive
    loop { tokio::time::sleep(std::time::Duration::from_secs(60)).await; }
    
}
