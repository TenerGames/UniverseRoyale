mod plugins;

use std::net::{IpAddr, Ipv4Addr};
use bevy::DefaultPlugins;
use bevy::prelude::{App, EventReader, IntoScheduleConfigs,ResMut, Update};
use bevy::utils::default;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use shared::NetworkSide;
use shared::plugins::replication_plugin::{MyMessage, ReplicationPlugin};
use shared::systems::stream_connection::{ClientConnectedEvent, ConnectionsServer, ConnectionsTrait, MessageReceived, TcpSettings};

pub fn test(mut connections_server: ResMut<ConnectionsServer>){
    connections_server.make_tcp_connection(TcpSettings{
        address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        port: 8080,
        ..default()
    },"Lobby".to_string());
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
