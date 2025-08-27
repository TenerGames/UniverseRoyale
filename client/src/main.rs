use bevy::app::Update;
use bevy::DefaultPlugins;
use bevy::prelude::{App, EventReader, IntoScheduleConfigs, ResMut};
use shared::NetworkSide;
use shared::plugins::replication_plugin::{MyMessage, ReplicationPlugin};
use shared::systems::stream_connection::{ConnectionsClient, ConnectionsTrait, MessageReceived};

pub mod plugins;

pub fn test(mut connections_client: ResMut<ConnectionsClient>){
    connections_client.make_tcp_connection("127.0.0.1:8080".to_string(),"Lobby".to_string());
}

pub fn message_received(
    mut message_received: EventReader<MessageReceived<MyMessage>>
){
    for ev in message_received.read() {
        println!("Message from server {} ", &ev.message.test)
    }
}

fn main() {
    App::new().add_plugins((DefaultPlugins,ReplicationPlugin{
        network_side: NetworkSide::Client,
    })).add_systems(Update,(test,message_received).chain()).run();
}
