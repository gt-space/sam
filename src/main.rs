use command::command_loop::begin;
use std::thread;

pub mod gpio;
pub mod adc;
pub mod command;
pub mod data;
pub mod discovery;
pub mod state;

// https://github.com/rust-embedded/rust-spidev/blob/master/examples/spidev-bidir.rs
fn main() {
    // /* Create a spidev wrapper to work with
    // you call this wrapper to handle and all transfers */
    // let mut spidev = Spidev::open("/dev/spidev0.0").unwrap();
    // /* ADS124S06 only supports SPI mode 1, so we're sticking with that here
    //   For a primer on SPI modes of operation, look here:
    //   https://stackoverflow.com/questions/43155025/why-different-modes-are-provided-in-spi-communication
    // */

    // let options = SpidevOptions::new()
    //     .bits_per_word(8)
    //     .max_speed_hz(100000)
    //     .lsb_first(false)
    //     .mode(SpiModeFlags::SPI_MODE_1)
    //     .build();
    // spidev.configure(&options).unwrap();

    // let ref_spidev: Rc<RefCell<_>> = Rc::new(RefCell::new(spidev));
    // let adc_differential = adc::ADC::new(adc::Measurement::DiffSensors, ref_spidev.clone());

    let state_thread = thread::spawn( || {
        init_state();
    });

    let command_thread = thread::spawn(move || loop {
        begin();
    });

    state_thread.join();
    command_thread.join();
}

fn init_state() {
    let mut sam_state = state::State::Init;
    let mut data = state::Data::new();
    loop {
        sam_state = sam_state.next(&mut data);
    }
}