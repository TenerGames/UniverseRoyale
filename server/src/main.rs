mod plugins;

use bevy::DefaultPlugins;
use bevy::prelude::{App, EventReader, IntoScheduleConfigs,ResMut, Update};
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use shared::NetworkSide;
use shared::plugins::replication_plugin::{MyMessage, ReplicationPlugin};
use shared::systems::stream_connection::{ClientConnectedEvent, ConnectionsServer, ConnectionsTrait, MessageReceived};

pub fn test(mut connections_server: ResMut<ConnectionsServer>){
    connections_server.make_tcp_connection("127.0.0.1:8080".to_string(),"Lobby".to_string());
}

pub fn check_client_connected(
    mut client_connected: EventReader<ClientConnectedEvent>,
){
    for client in client_connected.read(){
        println!("Client connected on {}", &client.0.1);
    }
}

pub fn message_received(
    mut message_received: EventReader<MessageReceived<MyMessage>>
){
    for ev in message_received.read() {
        println!("Message from client {} ", &ev.message.test)
    }
}

fn main() {
    App::new().add_plugins((DefaultPlugins,ReplicationPlugin{
        network_side: NetworkSide::Server,
    },EguiPlugin::default(), WorldInspectorPlugin::new())).add_systems(Update,(test,check_client_connected,message_received).chain()).run();
}
