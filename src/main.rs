use data::data_loop;
use command::command_loop;
use std::thread;

pub mod data;
pub mod command;

fn main() {
    let command_loop = thread::spawn(|| data_loop::begin(1000));
    let data_loop = thread::spawn(|| command_loop::begin());


    command_loop.join().unwrap();
    data_loop.join().unwrap();
    panic!("Control loop terminated!");
}
