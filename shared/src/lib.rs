pub mod systems;
pub mod plugins;

#[derive(PartialEq)]
pub enum NetworkSide{
    Client,
    Server
}