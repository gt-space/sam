use fs_protobuf_rust::compiled::mcfs::command;
use fs_protobuf_rust::compiled::mcfs::core;
use fs_protobuf_rust::compiled::mcfs::board;
use quick_protobuf::Error;
use quick_protobuf::{deserialize_from_slice};
use std::fs::File;

use std::io::Write;
use std::net::UdpSocket;
use crate::gpio;


pub fn begin() {
    let socket = UdpSocket::bind("0.0.0.0:8378").expect("Cannot bind to socket");
    let mut buf = [0; 65536];
    loop {
        let (_num_bytes, _src_addr) = socket.recv_from(&mut buf).expect("no data received");

        let deserialized_result: Result<core::Message, Error> = deserialize_from_slice(&buf);
        println!("{:#?}", deserialized_result);
        match deserialized_result {
            Ok(message) => match message.content {
                core::mod_Message::OneOfcontent::command(command) => execute(command),
                core::mod_Message::OneOfcontent::data(..) => println!("Data"),
                core::mod_Message::OneOfcontent::status(..) => println!("Command"),
                _ => println!("Other"),
            },
            Err(_error) => println!("Bad"),
        };
    }
}

fn execute(command: command::Command) {
    match command.command {
        command::mod_Command::OneOfcommand::set_led(set_led_command) => {
            let led = set_led_command.led.unwrap();
            match set_led_command.state {
                board::LEDState::LED_ON => match led.channel {
                    0 => {
                        let mut file: File = std::fs::OpenOptions::new()
                            .write(true)
                            .truncate(true)
                            .open("/sys/class/leds/beaglebone:green:usr0/brightness")
                            .unwrap();
                        file.write(b"1").expect("Failed to write");
                    }
                    1 => {
                        let mut file: File = std::fs::OpenOptions::new()
                            .write(true)
                            .truncate(true)
                            .open("/sys/class/leds/beaglebone:green:usr1/brightness")
                            .unwrap();
                        file.write(b"1").expect("Failed to write");
                    }
                    2 => {
                        let mut file: File = std::fs::OpenOptions::new()
                            .write(true)
                            .truncate(true)
                            .open("/sys/class/leds/beaglebone:green:usr2/brightness")
                            .unwrap();
                        file.write(b"1").expect("Failed to write");
                    }
                    3 => {
                        let mut file: File = std::fs::OpenOptions::new()
                            .write(true)
                            .truncate(true)
                            .open("/sys/class/leds/beaglebone:green:usr3/brightness")
                            .unwrap();
                        file.write(b"1").expect("Failed to write");
                    }
                    _ => println!("Error"),
                },
                board::LEDState::LED_OFF => match led.channel {
                    0 => {
                        let mut file: File = std::fs::OpenOptions::new()
                            .write(true)
                            .truncate(true)
                            .open("/sys/class/leds/beaglebone:green:usr0/brightness")
                            .unwrap();
                        file.write(b"0").expect("Failed to write");
                    }
                    1 => {
                        let mut file: File = std::fs::OpenOptions::new()
                            .write(true)
                            .truncate(true)
                            .open("/sys/class/leds/beaglebone:green:usr1/brightness")
                            .unwrap();
                        file.write(b"0").expect("Failed to write");
                    }
                    2 => {
                        let mut file: File = std::fs::OpenOptions::new()
                            .write(true)
                            .truncate(true)
                            .open("/sys/class/leds/beaglebone:green:usr2/brightness")
                            .unwrap();
                        file.write(b"0").expect("Failed to write");
                    }
                    3 => {
                        let mut file: File = std::fs::OpenOptions::new()
                            .write(true)
                            .truncate(true)
                            .open("/sys/class/leds/beaglebone:green:usr3/brightness")
                            .unwrap();
                        file.write(b"0").expect("Failed to write");
                    }
                    _ => println!("Error"),
                },
            }
        }

        command::mod_Command::OneOfcommand::click_valve(click_valve_command) => {
            let valve = click_valve_command.valve.unwrap();
            match click_valve_command.state {
                board::ValveState::VALVE_OPEN => match valve.channel {
                    1 => {
                        gpio::set_output("8");
                        gpio::set_high("8");
                    }
                    2 => {
                        gpio::set_output("80");
                        gpio::set_high("80");
                    }
                    3 => {
                        gpio::set_output("81");
                        gpio::set_high("81");
                    }
                    4 => {
                        gpio::set_output("89");
                        gpio::set_high("89");
                    }
                    5 => {
                        gpio::set_output("65");
                        gpio::set_high("65");
                    }
                    6 => {
                        gpio::set_output("46");
                        gpio::set_high("46");
                    }
                    _ => println!("Error"),
                },
                board::ValveState::VALVE_CLOSED => match valve.channel {
                    1 => {
                        gpio::set_output("8");
                        gpio::set_low("8");
                    }
                    2 => {
                        gpio::set_output("80");
                        gpio::set_low("80");
                    }
                    3 => {
                        gpio::set_output("81");
                        gpio::set_low("81");
                    }
                    4 => {
                        gpio::set_output("89");
                        gpio::set_low("89");
                    }
                    5 => {
                        gpio::set_output("65");
                        gpio::set_low("65");
                    }
                    6 => {
                        gpio::set_output("46");
                        gpio::set_low("46");
                    }
                    _ => println!("Error"),
                },
                
            }

        }
        _ => println!("Unknown command"),

    }
}

// For testing only.
pub fn open_valve(id: u32) {
    let command = command::Command {
        command: command::mod_Command::OneOfcommand::click_valve(
            command::ClickValve { 
                valve: (Some(board::ChannelIdentifier {board_id: 10, channel_type: board::ChannelType::VALVE, channel: id})), 
                state: (board::ValveState::VALVE_OPEN)
    })};
    execute(command);
}

pub fn close_valve(id: u32) {
    let command = command::Command {
        command: command::mod_Command::OneOfcommand::click_valve(
            command::ClickValve { 
                valve: (Some(board::ChannelIdentifier {board_id: 10, channel_type: board::ChannelType::VALVE, channel: id})), 
                state: (board::ValveState::VALVE_CLOSED)
    })};
    execute(command);
}
