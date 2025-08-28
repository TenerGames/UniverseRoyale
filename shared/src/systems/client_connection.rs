use std::net::SocketAddr;
use std::sync::{atomic, Arc, Weak};
use std::sync::atomic::AtomicBool;
use bincode::config::standard;
use bincode::serde::encode_to_vec;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{UnboundedSender};
use tokio::sync::Mutex;
use uuid::Uuid;
use crate::NetworkSide;
use crate::systems::stream_connection::{deserialize_message, MessageTrait};

pub struct ClientComponent {
    pub write_half: Arc<Mutex<OwnedWriteHalf>>,
    pub read_half: Arc<Mutex<OwnedReadHalf>>,
    pub socket_addr: Arc<SocketAddr>,
    pub connection_name: String,
    pub network_side: NetworkSide,
    pub listening: bool,
    pub runtime: Option<Weak<Runtime>>,
    pub connection_dropped: Arc<AtomicBool>,
    pub uuid: Uuid
}

impl Drop for ClientComponent {
    fn drop(&mut self) {
        println!("Client from connection {}, with SocketAddr {} disconnected", &self.connection_name, &self.socket_addr);
    }
}

impl ClientComponent {
    pub fn send_client_message(&self, message: Box<dyn MessageTrait>) {
        assert!(self.network_side == NetworkSide::Server, "You cant call this on client, just on server");

        let write_half_downcast = Arc::downgrade(&self.write_half);
        let config = standard();
        let encoded = encode_to_vec(&message, config).unwrap();
        let length = encoded.len() as u32;
        
        if let Some(weak_runtime) = &self.runtime {
            if let Some(runtime) = weak_runtime.upgrade() {
                runtime.spawn(async move {
                    if let Some(write_half) = write_half_downcast.upgrade() {
                        let mut write_half = write_half.lock().await;

                        match write_half.write_u32(length).await {
                            Ok(_) => {
                                match write_half.write_all(&encoded).await {
                                    Ok(_) => {
                                        println!("Sent Message");
                                    },
                                    Err(_) => {
                                        println!("Failed to send Message");
                                    }
                                }
                            },
                            Err(_) => {

                            }
                        }
                    };
                });
            }
        }
    }

    pub fn send_message_server(&self, message: Box<dyn MessageTrait>) {
        assert!(self.network_side == NetworkSide::Client, "You cant call this on server, just on client");

        let write_half_downcast = Arc::downgrade(&self.write_half);
        let config = standard();
        let encoded = encode_to_vec(&message, config).unwrap();
        let length = encoded.len() as u32;

        if let Some(weak_runtime) = &self.runtime {
            if let Some(runtime) = weak_runtime.upgrade() {
                runtime.spawn(async move {
                    if let Some(write_half) = write_half_downcast.upgrade() {
                        let mut write_half = write_half.lock().await;

                        match write_half.write_u32(length).await {
                            Ok(_) => {
                                match write_half.write_all(&encoded).await {
                                    Ok(_) => {

                                    },
                                    Err(_) => {
                                        println!("Failed to send Message");
                                    }
                                }
                            },
                            Err(_) => {
                                println!("error to send message")
                            }
                        }
                    }else {
                        println!("Error to upgrade");
                    }
                });
            }
        }
        
    }

    pub fn start_listening_client(&mut self, sender_message: Weak<UnboundedSender<(Box<dyn MessageTrait>, Option<Uuid>)>>) {
        assert!(self.network_side == NetworkSide::Server, "You cant call this on client, just on server");
        assert_ne!(self.listening, true, "You are already listening");

        let read_half_downcast = Arc::downgrade(&self.read_half);
        let arc_connection_dropped = Arc::downgrade(&self.connection_dropped);
        let uuid = self.uuid.clone();

        self.listening = true;

        if let Some(weak_runtime) = &self.runtime {
            if let Some(runtime) = weak_runtime.upgrade() {
                runtime.spawn(async move {
                    loop {
                        if let Some(read_half) = read_half_downcast.upgrade() {
                            let mut read_half = read_half.lock().await;

                            match read_half.read_u32().await {
                                Ok(length) => {
                                    let mut buf = vec![0u8; length as usize];

                                    match read_half.read_exact(&mut buf).await {
                                        Ok(0) => {
                                            if let Some(arc_connection_dropped) = arc_connection_dropped.upgrade() {
                                                arc_connection_dropped.store(true, atomic::Ordering::SeqCst);
                                            }
                                            println!("Connection dropped from no value");
                                            break
                                        },
                                        Ok(_) => {
                                            if let Some(sender_message) = sender_message.upgrade() {
                                                if let Some(message) = deserialize_message(&buf) {
                                                    match sender_message.send((message,Some(uuid.clone()))) {
                                                        Ok(_) => {

                                                        },
                                                        Err(e) => println!("Error sending message: {:?}", e)
                                                    };
                                                } else {
                                                    println!("Mensagem não registrada ou falha na desserialização");
                                                }
                                            }
                                        },
                                        Err(_) => {
                                            println!("Fatal error")
                                        }
                                    }
                                },
                                Err(_) => {
                                    if let Some(arc_connection_dropped) = arc_connection_dropped.upgrade() {
                                        arc_connection_dropped.store(true, atomic::Ordering::SeqCst);
                                    }
                                    println!("Failed to read from stream");
                                    break
                                }
                            }
                        }else {
                            if let Some(arc_connection_dropped) = arc_connection_dropped.upgrade() {
                                arc_connection_dropped.store(true, atomic::Ordering::SeqCst);
                            }
                            println!("Connection dropped");
                            break
                        }
                    }
                });
            }
        }
    }
}