use std::any::{Any, TypeId};
use std::collections::{HashMap};
use std::io::Cursor;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use bevy::app::App;
use bevy::prelude::{Event, Resource, World};
use bincode::config::standard;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex as TokioMutex;
use typetag::__private::once_cell::sync::Lazy;
use uuid::Uuid;
use crate::NetworkSide;
use crate::systems::client_connection::ClientComponent;

#[derive(PartialEq)]
pub enum StreamType{
    Tcp,
    Udp,
}

#[typetag::serde]
pub trait MessageTrait: Send + Sync + Any {
    fn as_any(&self) -> &dyn Any;
}

pub enum ConnectionType{
    Tcp{
        listener: Option<Arc<TcpListener>>,
        receiver_connection_stabilized: Option<UnboundedReceiver<Arc<TcpListener>>>,
        local_client_connection: Option<Arc<Mutex<ClientComponent>>>,
        started: bool,
        runtime: Option<Runtime>,
        address: String,
        network_side: NetworkSide,
        connection_name: String,
        clients_connected: Arc<Mutex<HashMap<Uuid,ClientComponent>>>,
        messages_receiver: Option<UnboundedReceiver<(Box<dyn MessageTrait>, Option<Uuid>)>>,
        messages_sender: Option<Arc<UnboundedSender<(Box<dyn MessageTrait>, Option<Uuid>)>>>,
        receiver_local_tcp_connected: Option<UnboundedReceiver<Option<(Arc<TokioMutex<OwnedReadHalf>>,Arc<TokioMutex<OwnedWriteHalf>>, SocketAddr)>>>,
        client_connected_receiver: Option<UnboundedReceiver<(Arc<SocketAddr>,String)>>,
    },
    Udp{
        udp_socket: Option<Arc<UdpSocket>>,
        receiver_connection_stabilized: Option<UnboundedReceiver<Arc<UdpSocket>>>,
        started: bool,
        runtime: Option<Runtime>,
        address: String,
        network_side: NetworkSide,
        connection_name: String
    },
}

pub trait ConnectionTrait{
    fn start(&mut self);
    fn can_start(&mut self) -> bool;
}

pub trait ConnectionsTrait{
    fn make_tcp_connection(&mut self, address: String, connection_name: String);
    fn remove_connection(&mut self, connection_name: String);
    fn can_connect(&mut self, connection_name: &String) -> bool;
}

#[derive(Event)]
pub struct MessageReceived<T: MessageTrait>{
    pub message: T,
    pub uuid: Option<Uuid>,
}

#[derive(Event)]
pub struct ClientConnectedEvent(pub (Arc<SocketAddr>,String));

#[derive(Resource)]
pub struct ConnectionsServer(pub HashMap<String, ConnectionType>);

#[derive(Resource)]
pub struct ConnectionsClient(pub HashMap<String, ConnectionType>);

pub type DispatcherFn = Box<dyn Fn(Box<dyn Any>, &mut World, Option<Uuid>) + Send + Sync>;

pub static DISPATCHERS: Lazy<Mutex<HashMap<TypeId, DispatcherFn>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn deserialize_message(buf: &[u8]) -> Option<Box<dyn MessageTrait>> {
    let config = standard();
    let mut cursor = Cursor::new(buf);

    match bincode::serde::decode_from_std_read::<Box<dyn MessageTrait>, _, _>(&mut cursor, config) {
        Ok(msg) => Some(msg),
        Err(_) => None,
    }
}

#[macro_export]
macro_rules! register_message_type {
    ($type:ty, $dispatcher_map:expr) => {{
        use std::any::TypeId;
        let dispatcher: DispatcherFn = Box::new(|boxed, world, uuid_opt| {
            let msg = boxed.downcast::<$type>().expect("Failed to downcast");
            world.send_event(MessageReceived {
                message: *msg,
                uuid: uuid_opt,
            });
        });
        $dispatcher_map.lock().unwrap().insert(TypeId::of::<$type>(), dispatcher);
    }};
}

pub fn register_message_type<T: MessageTrait>(app: &mut App){
    app.add_event::<MessageReceived<T>>();
    register_message_type!(T, &DISPATCHERS);
}

impl Drop for ConnectionType{
    fn drop(&mut self){
        match self {
            ConnectionType::Tcp {
                listener: _listener,
                local_client_connection: _local_client_connection,
                started: _started,
                runtime,
                ..
            } => {
                if let Some(runtime) = runtime.take(){
                    runtime.shutdown_background()
                }
            }
            ConnectionType::Udp {
                udp_socket: _udp_socket,
                started: _started,
                runtime,
                ..
            } => {
                if let Some(runtime) = runtime.take(){
                    runtime.shutdown_background()
                }
            }
        }
    }
}

impl ConnectionTrait for ConnectionType{
    #[tokio::main]
    async fn start(&mut self) {
        if !self.can_start() {return;}

        match self {
            ConnectionType::Tcp {
                listener: _listener,
                local_client_connection: _local_client_connection,
                started,
                runtime,
                address,
                network_side,
                receiver_connection_stabilized,
                connection_name,
                clients_connected,
                messages_receiver,
                messages_sender,
                receiver_local_tcp_connected,
                client_connected_receiver
            } => {
                *started = true;

                if network_side == &NetworkSide::Server {
                    let (sender, receiver) = unbounded_channel::<Arc<TcpListener>>();
                    let (sender_connected, mut receiver_connected) = unbounded_channel::<(TcpStream, SocketAddr)>();
                    let (sender_client_connected,receiver_client_connected) = unbounded_channel::<(Arc<SocketAddr>,String)>();
                    let (sender_message, receiver_message) = unbounded_channel::<(Box<dyn MessageTrait>, Option<Uuid>)>();
                    let new_runtime = Runtime::new().unwrap();
                    let address_clone = address.clone();
                    let clients_connected_arc = Arc::downgrade(&clients_connected);
                    let connection_name_clone = connection_name.clone();

                    *receiver_connection_stabilized = Some(receiver);
                    *runtime = Some(new_runtime);
                    *messages_receiver = Some(receiver_message);
                    *messages_sender = Some(Arc::new(sender_message));
                    *client_connected_receiver = Some(receiver_client_connected);

                    runtime.as_ref().unwrap().spawn(async move {
                        let listener_arc = Arc::new(match TcpListener::bind(address_clone).await {
                            Ok(l) => l,
                            Err(_) => return
                        });
                        let weak_listener = Arc::downgrade(&listener_arc);

                        match sender.send(listener_arc) {
                            Ok(_) => {},
                            Err(_) => return
                        };

                        loop {
                            if let Some(listener) = weak_listener.upgrade() {
                                match listener.accept().await {
                                    Ok((stream,socket)) => {
                                        match sender_connected.send((stream,socket)) {
                                            Ok(_) => {},
                                            Err(_) => (),
                                        };
                                    }
                                    Err(e) => {
                                        print!("Error: {}", e);
                                    }
                                }
                            }else {
                                println!("Disconnected");
                                break;
                            }
                        }
                    });

                    runtime.as_ref().unwrap().spawn(async move {
                        loop {
                            if let Some(clients_connected) = clients_connected_arc.upgrade() {
                                match receiver_connected.recv().await {
                                    Some((stream,socket)) => {
                                        let mut queue = clients_connected.lock().unwrap();
                                        let uuid = Uuid::new_v4();

                                        let (read_half, write_half) = stream.into_split();

                                        let client_component = ClientComponent{
                                            write_half: Arc::new(TokioMutex::new(write_half)),
                                            read_half: Arc::new(TokioMutex::new(read_half)),
                                            socket_addr: Arc::new(socket),
                                            connection_name: connection_name_clone.clone(),
                                            network_side: NetworkSide::Server,
                                            listening: false,
                                            runtime: Some(Runtime::new().unwrap()),
                                            connection_dropped: Arc::new(AtomicBool::new(false)),
                                            uuid
                                        };

                                        match sender_client_connected.send((Arc::clone(&client_component.socket_addr),connection_name_clone.clone())) {
                                            Ok(_) => {},
                                            Err(_) => (),
                                        };

                                        queue.insert(uuid, client_component);
                                    },
                                    None => {
                                        break
                                    }
                                }
                            }else {
                                break;
                            }
                        }
                    });
                }else {
                    let (sender, receiver) = unbounded_channel::<Option<(Arc<TokioMutex<OwnedReadHalf>>,Arc<TokioMutex<OwnedWriteHalf>>, SocketAddr)>>();
                    let (sender_message, receiver_message) = unbounded_channel::<(Box<dyn MessageTrait>,Option<Uuid>)>();
                    let new_runtime = Runtime::new().unwrap();
                    let address_clone = address.clone();

                    *receiver_local_tcp_connected = Some(receiver);
                    *runtime = Some(new_runtime);
                    *messages_receiver = Some(receiver_message);

                    runtime.as_ref().unwrap().spawn(async move {
                        let tcp_stream;

                        loop {
                            tcp_stream = match TcpStream::connect(&address_clone).await {
                                Ok(s) => { println!("Connected to server: {}", &s.peer_addr().unwrap()); s},
                                Err(e) => { println!("Error trying connect: {} ", e);
                                    continue;
                                }
                            };

                            break
                        }

                        let socket_address = match tcp_stream.local_addr() {
                            Ok(a) => a,
                            Err(_) => {return;}
                        };

                        let (read_half, write_half) = tcp_stream.into_split();

                        let read_half = Arc::new(TokioMutex::new(read_half));

                        sender.send(Some((Arc::clone(&read_half), Arc::new(TokioMutex::new(write_half)), socket_address))).unwrap();

                        loop {
                            let mut read_half_mutex = read_half.lock().await;

                            match read_half_mutex.read_u32().await {
                                Ok(length) => {
                                    let mut buf = vec![0u8; length as usize];

                                    match read_half_mutex.read_exact(&mut buf).await {
                                        Ok(_) => {
                                            if let Some(message) = deserialize_message(&buf) {
                                                match sender_message.send((message,None)) {
                                                    Ok(_) => {
                                                        println!("Message sent");
                                                    },
                                                    Err(e) => println!("Error sending message: {:?}", e)
                                                };
                                            } else {
                                                println!("Mensagem não registrada ou falha na desserialização");
                                            }
                                        },
                                        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                                            break;
                                        }
                                        Err(other_e) => {
                                            println!("Error: {:?}", other_e);
                                        }
                                    }
                                }
                                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                                    break;
                                }
                                Err(other_e) => {
                                    println!("Error: {:?}", other_e);
                                }
                            }
                        }
                    });
                }
            }
            ConnectionType::Udp {
                ..
            } => {

            }
        }
    }

    fn can_start(&mut self) -> bool {
        match self {
            ConnectionType::Tcp {
                listener: _listener,
                local_client_connection: _local_client_connection,
                started,
                runtime: _runtime,
                address: _address,
                network_side: _network_side,
                receiver_connection_stabilized: _receiver_connection_stabilized,
                ..
            } => {
               !*started
            }
            ConnectionType::Udp {
                udp_socket: _udp_socket,
                started,
                runtime: _runtime,
                address: _address,
                network_side: _network_side,
                receiver_connection_stabilized: _receiver_connection_stabilized,
                ..
            } => {
                !*started
            }
        }
    }
}

impl ConnectionsTrait for ConnectionsServer{
    fn make_tcp_connection(&mut self, address: String, connection_name: String){
        if !self.can_connect(&connection_name) {return;}

        let connection_name_clone = connection_name.clone();

        self.0.insert(connection_name, ConnectionType::Tcp{
            listener: None,
            receiver_connection_stabilized: None,
            local_client_connection: None,
            started: false,
            runtime: None,
            address,
            network_side: NetworkSide::Server,
            connection_name: connection_name_clone,
            clients_connected: Arc::new(Mutex::new(HashMap::new())),
            messages_receiver: None,
            messages_sender: None,
            receiver_local_tcp_connected: None,
            client_connected_receiver: None
        });
    }

    fn remove_connection(&mut self, connection_name: String){
        let _ = match self.0.get_mut(&connection_name) {
            Some(connection) => connection,
            None => {return;}
        };

        self.0.remove(&connection_name);
    }

    fn can_connect(&mut self, connection_name: &String) -> bool {
        !self.0.contains_key(connection_name)
    }
}

impl ConnectionsTrait for ConnectionsClient{
    #[tokio::main]
    async fn make_tcp_connection(&mut self, address: String, connection_name: String){
        if !self.can_connect(&connection_name) {return;}

        let connection_name_clone = connection_name.clone();

        self.0.insert(connection_name,ConnectionType::Tcp{
            listener: None,
            receiver_connection_stabilized: None,
            local_client_connection: None,
            started: false,
            runtime: None,
            address,
            network_side: NetworkSide::Client,
            connection_name : connection_name_clone,
            clients_connected: Arc::new(Mutex::new(HashMap::new())),
            messages_receiver: None,
            messages_sender: None,
            receiver_local_tcp_connected: None,
            client_connected_receiver: None
        });
    }

    fn remove_connection(&mut self, connection_name: String){
        let _ = match self.0.get_mut(&connection_name) {
            Some(connection) => connection,
            None => {return;}
        };

        self.0.remove(&connection_name);
    }

    fn can_connect(&mut self, connection_name: &String) -> bool {
        !self.0.contains_key(connection_name)
    }
}