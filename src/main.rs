use command::command_loop;
use data::data_loop;
use discovery::discovery_loop;
use std::{thread, time::Duration};

pub mod command;
pub mod data;
pub mod discovery;

fn main() {
    let data_loop = thread::spawn(|| data_loop::begin(500));
    let command_loop = thread::spawn(|| command_loop::begin());
    let discovery_loop = thread::spawn(|| discovery_loop::begin(Duration::from_secs(5)));

    command_loop.join().unwrap();
    data_loop.join().unwrap();
    discovery_loop.join().unwrap();
    panic!("Control loop terminated!");
}
