#![feature(thread_id_value)]
#![feature(duration_millis_float)]

use std::net::SocketAddrV4;

use clap::Parser;
use client::exec_client;
use server::exec_server;

mod client;
mod common;
mod server;

#[derive(Parser)]
#[command(version, about, long_about = None)]
enum Args {
    Server(ServerCommand),
    Client(ClientCommand),
}

#[derive(Parser)]
struct ServerCommand {
    #[arg(short, long)]
    port: u16,
}

#[derive(Parser)]
struct ClientCommand {
    #[arg(short, long)]
    address: SocketAddrV4,
}

pub fn main() {
    match Args::parse() {
        Args::Server(command) => {
            exec_server(command.port);
        }
        Args::Client(command) => {
            exec_client(command.address).unwrap();
        }
    }
}
