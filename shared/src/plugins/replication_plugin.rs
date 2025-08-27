use std::any::Any;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use bevy::app::{App, Update};
use bevy::prelude::{Commands, EventWriter, First, IntoScheduleConfigs, Last, Plugin, ResMut, World};
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;
use uuid::Uuid;
use message_derive::Message;
use crate::{NetworkSide};
use crate::systems::client_connection::ClientComponent;
use crate::systems::stream_connection::{register_message_type, ClientConnectedEvent, ConnectionTrait, ConnectionType, ConnectionsClient, ConnectionsServer, MessageTrait, DISPATCHERS};

#[derive(PartialEq)]
pub struct ReplicationPlugin{
    pub network_side: NetworkSide
}

#[derive(Serialize, Deserialize, Message)]
pub struct MyMessage{
    pub test: String,
}

impl Plugin for ReplicationPlugin {
    fn build(&self, app: &mut App) {

        if self.network_side == NetworkSide::Server {
            app.add_event::<ClientConnectedEvent>();
            app.insert_resource(ConnectionsServer(HashMap::default()));
            app.add_systems(First,(setup_server_connections,listener_connection_stabilized,new_client_connected,start_listening_clients).chain());
            app.add_systems(Update,check_new_messages_server);
            app.add_systems(Last,check_clients_disconnected);
        }else {
            app.insert_resource(ConnectionsClient(HashMap::default()));
            app.add_systems(First,(setup_client_connections,client_tcp_connected).chain());
            app.add_systems(Update,check_new_messages_client);
        }

        register_message_type::<MyMessage>(app);
    }
}

//Server area
pub fn setup_server_connections(
    mut connections_server: ResMut<ConnectionsServer>,
){
    for connection in connections_server.0.values_mut(){
        if !connection.can_start() {continue}

        connection.start();
    }
}

pub fn listener_connection_stabilized(
    mut connections_server: ResMut<ConnectionsServer>
){
    for connection in connections_server.0.values_mut(){
        if let ConnectionType::Tcp {
            listener,
            local_client_connection: _local_client_connection,
            started: _started,
            runtime: _runtime,
            address: _address,
            network_side: _network_side,
            receiver_connection_stabilized,
            connection_name: _connection_name,
            clients_connected: _clients_connected,
            messages_receiver: _message_receiver,
            messages_sender: _message_sender,
            receiver_local_tcp_connected: _receiver_local_tcp_connected,
            client_connected_receiver: _client_connected_receiver
        } = connection {
            if let Some(receiver) = receiver_connection_stabilized{
                match receiver.try_recv() {
                    Ok(tcp_listener) => {
                        *listener = Some(tcp_listener);
                    }
                    Err(_) => {
                        continue
                    }
                };
            }
        }
    }
}

pub fn new_client_connected(
    mut connections_server: ResMut<ConnectionsServer>,
    mut clients_connected_event: EventWriter<ClientConnectedEvent>
){
    for connection in connections_server.0.values_mut(){
        if let ConnectionType::Tcp {
            listener : _listener,
            local_client_connection: _local_client_connection,
            started: _started,
            runtime: _runtime,
            address: _address,
            network_side: _network_side,
            receiver_connection_stabilized: _receiver_connection_stabilized,
            connection_name: _connection_name,
            clients_connected: _clients_connected,
            messages_receiver: _message_receiver,
            messages_sender: _message_sender,
            receiver_local_tcp_connected: _receiver_local_tcp_connected,
            client_connected_receiver
        } = connection {
            if let Some(receiver) = client_connected_receiver{
                match receiver.try_recv() {
                    Ok((socket_address,connection_name)) => {
                        clients_connected_event.write(ClientConnectedEvent((socket_address,connection_name)));
                    },
                    Err(_) => {continue}
                }
            }
        }
    }
}

pub fn start_listening_clients(
    mut connections_server: ResMut<ConnectionsServer>,
){
    for connection in connections_server.0.values_mut(){
        if let ConnectionType::Tcp {
            listener : _listener,
            local_client_connection: _local_client_connection,
            started: _started,
            runtime: _runtime,
            address: _address,
            network_side: _network_side,
            receiver_connection_stabilized: _receiver_connection_stabilized,
            connection_name: _connection_name,
            clients_connected,
            messages_receiver: _messages_receiver,
            messages_sender,
            ..
        } = connection {
            let mut clients_connected = match clients_connected.lock() {
                Ok(clients_connected) => clients_connected,
                Err(_) => continue
            };

            for client in clients_connected.values_mut(){
                if client.listening {continue};

                match messages_sender {
                    Some(messages_sender) => {
                        client.start_listening_client(Arc::downgrade(messages_sender));
                    },
                    None => {
                        continue;
                    }
                }
            }
        }
    }
}

pub fn check_clients_disconnected(
    mut connections_server: ResMut<ConnectionsServer>,
){
    for connection in connections_server.0.values_mut(){
        if let ConnectionType::Tcp {
            listener : _listener,
            local_client_connection: _local_client_connection,
            started: _started,
            runtime: _runtime,
            address: _address,
            network_side: _network_side,
            receiver_connection_stabilized: _receiver_connection_stabilized,
            connection_name: _connection_name,
            clients_connected,
            ..
        } = connection {

            let mut clients_connected = match clients_connected.lock() {
                Ok(clients_connected) => clients_connected,
                Err(_) => continue
            };

            let mut keys_remove: Vec<Uuid> = Vec::new();

            for (key,value) in clients_connected.iter(){
                if value.connection_dropped.load(Ordering::SeqCst) {
                    keys_remove.push(*key);
                }
            }

            for key in keys_remove{
                clients_connected.remove(&key);
            }
        }
    }
}

pub fn check_new_messages_server(
    mut connections_server: ResMut<ConnectionsServer>,
    mut commands: Commands,
){
    for connection in connections_server.0.values_mut(){
        if let ConnectionType::Tcp {
            listener : _listener,
            local_client_connection: _local_client_connection,
            started: _started,
            runtime: _runtime,
            address: _address,
            network_side: _network_side,
            receiver_connection_stabilized: _receiver_connection_stabilized,
            connection_name: _connection_name,
            clients_connected: _clients_connected,
            messages_receiver,
            ..
        } = connection
        {
            if let Some(message) = messages_receiver {
                match message.try_recv() {
                    Ok((message_trait, uuid)) => {
                        commands.queue(move |w: &mut World| {
                            let type_id = message_trait.as_any().type_id();
                            let map = DISPATCHERS.lock().unwrap();
                            if let Some(dispatcher) = map.get(&type_id) {
                                let boxed_any = message_trait as Box<dyn Any>;

                                dispatcher(boxed_any, w, uuid);
                            } else {
                                println!("This message does not exist");
                            }
                        });
                    },
                    Err(_) => {

                    }
                }
            }
        }
    }
}

//Client area
pub fn setup_client_connections(
    mut connections_client: ResMut<ConnectionsClient>,
){
    for connection in connections_client.0.values_mut(){
        if !connection.can_start() {continue}

        connection.start();
    }
}

pub fn client_tcp_connected(
    mut connections_client: ResMut<ConnectionsClient>,
){
    for connection in connections_client.0.values_mut(){
        if let ConnectionType::Tcp {
            listener : _listener,
            local_client_connection,
            started,
            runtime: _runtime,
            address: _address,
            network_side: _network_side,
            receiver_connection_stabilized: _receiver_connection_stabilized,
            connection_name,
            clients_connected: _clients_connected,
            messages_receiver: _message_receiver,
            receiver_local_tcp_connected,
            ..
        } = connection
        {
            if local_client_connection.is_some() {continue}

            if let Some(receiver) = receiver_local_tcp_connected {
               match receiver.try_recv() {
                   Ok(message) => {
                       if let Some((read_half,write_half,socket_address)) = message {
                           let client_component = ClientComponent{
                               read_half,
                               write_half,
                               socket_addr: Arc::new(socket_address),
                               connection_name: connection_name.clone(),
                               network_side: NetworkSide::Client,
                               listening: true,
                               runtime: Some(Runtime::new().unwrap()),
                               connection_dropped: Arc::new(AtomicBool::new(false)),
                               uuid: Uuid::new_v4(),
                           };

                           *local_client_connection = Some(Arc::new(Mutex::new(client_component)));

                           if let Some(client_connection) = local_client_connection{
                              client_connection.lock().unwrap().send_message_server(Box::new(MyMessage{
                                  test: "Hi server, itz me client".to_string(),
                              }));
                           }
                       }else {
                           *started = false
                       }
                   }
                   Err(_) => {}
               }
            }
        }
    }
}
pub fn check_new_messages_client(
    mut connections_client: ResMut<ConnectionsClient>,
    mut commands: Commands,
){
    for connection in connections_client.0.values_mut(){
        if let ConnectionType::Tcp {
            listener : _listener,
            local_client_connection: _local_client_connection,
            started: _started,
            runtime: _runtime,
            address: _address,
            network_side: _network_side,
            receiver_connection_stabilized: _receiver_connection_stabilized,
            connection_name: _connection_name,
            clients_connected: _clients_connected,
            messages_receiver,
            ..
        } = connection
        {
            if let Some(message) = messages_receiver {
                match message.try_recv() {
                    Ok((message_trait, uuid)) => {
                        commands.queue(move |w: &mut World| {
                            let type_id = message_trait.as_any().type_id();
                            let map = DISPATCHERS.lock().unwrap();
                            if let Some(dispatcher) = map.get(&type_id) {
                                let boxed_any = message_trait as Box<dyn Any>;

                                dispatcher(boxed_any, w, uuid);
                            } else {
                               println!("This message does not exist");
                            }
                        });
                    },
                    Err(_) => {

                    }
                }
            }
        }
    }
}
