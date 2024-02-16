use std::{thread, sync::Arc};

use adc::open_controllers;
use command::command_loop::begin;
use gpio::Gpio;

pub mod gpio;
pub mod adc;
pub mod command;
pub mod data;
pub mod discovery;
pub mod state;
pub mod tc;

// https://github.com/rust-embedded/rust-spidev/blob/master/examples/spidev-bidir.rs
fn main() {
    let controllers = open_controllers();
    let controllers1 = controllers.clone();
    let controllers2 = controllers.clone();
    
    let state_thread = thread::spawn( move || {
        init_state(controllers1);
    });

    let command_thread = thread::spawn( move || {
        begin(controllers2.clone());
    });

    state_thread.join().expect("Could not join state thread");
    command_thread.join().expect("Could not join command thread");
}

fn init_state(controllers: Vec<Arc<Gpio>>) {
    let mut sam_state = state::State::Init;
    let mut data = state::Data::new();
    loop {
        sam_state = sam_state.next(&mut data, &controllers);
    }
}