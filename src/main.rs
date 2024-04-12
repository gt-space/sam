pub mod gpio;
pub mod adc;
pub mod command;
pub mod data;
pub mod discovery;
pub mod state;
pub mod tc;

use std::{thread, sync::Arc};
use adc::open_controllers;
use command::begin;
use gpio::Gpio;
use clap::{Arg, Command};

fn main() {
    let matches = Command::new("sam")
    .arg(
        Arg::new("printing-frequency")
            .long("freq")
            .short('f')
            .num_args(1)
            .value_parser(clap::value_parser!(u8).range(..=200))
    )
    .get_matches();

    let printing_frequency = *matches.get_one::<u8>("printing-frequency").unwrap_or(&0);

    let controllers = open_controllers();
    let controllers1 = controllers.clone();
    let controllers2 = controllers.clone();
    
    let state_thread = thread::spawn( move || {
        init_state(controllers1, printing_frequency);
    });

    let command_thread = thread::spawn( move || {
        begin(controllers2.clone());
    });

    state_thread.join().expect("Could not join state thread");
    command_thread.join().expect("Could not join command thread");
}

fn init_state(controllers: Vec<Arc<Gpio>>, printing_frequency: u8) {
    let mut sam_state = state::State::Init;
    let mut data = state::Data::new();
    loop {
        sam_state = sam_state.next(&mut data, &controllers, printing_frequency);
    }
}