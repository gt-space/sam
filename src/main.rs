use command::command_loop;
use data::data_loop;
use discovery::discovery_loop;
use std::net::UdpSocket;
use std::thread;

pub mod command;
pub mod data;
pub mod discovery;

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:24013").expect("Cannot bind to socket");
    let command_socket = socket.try_clone().expect("Could not clone socket");
    let data_socket = socket.try_clone().expect("Could not clone socket");
    let data_loop = thread::spawn(|| data_loop::begin(data_socket, 1));
    let command_loop = thread::spawn(|| command_loop::begin(command_socket));
    let discovery_loop = thread::spawn(|| discovery_loop::begin());

    command_loop.join().unwrap();
    data_loop.join().unwrap();
    discovery_loop.join().unwrap();
    panic!("Control loop terminated!");
}
