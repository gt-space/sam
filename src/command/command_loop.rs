use fs_protobuf_rust::compiled::mcfs::command;
use fs_protobuf_rust::compiled::mcfs::core;
use fs_protobuf_rust::compiled::mcfs::board;
use quick_protobuf::Error;
use quick_protobuf::deserialize_from_slice;
use std::fs::File;

use std::io::Write;
use std::net::UdpSocket;
use std::sync::Arc;
use crate::gpio::{Gpio, PinMode::Output, PinValue::{High, Low}};


pub fn begin(gpio_controllers: Vec<Arc<Gpio>>) {
    let socket = UdpSocket::bind("0.0.0.0:8378").expect("Cannot bind to socket");
    let mut buf = [0; 65536];
    loop {
        let (num_bytes, _src_addr) = socket.recv_from(&mut buf).expect("no data received");
        println!("{:?}", num_bytes);
        let deserialized_result: Result<core::Message, Error> = deserialize_from_slice(&buf);
        println!("{:#?}", deserialized_result);
        match deserialized_result {
            Ok(message) => match message.content {
                core::mod_Message::OneOfcontent::command(command) => execute(command, gpio_controllers.clone()),
                core::mod_Message::OneOfcontent::data(..) => println!("Data"),
                core::mod_Message::OneOfcontent::status(..) => println!("Command"),
                _ => println!("Other"),
            },
            Err(_error) => println!("Bad"),
        };
    }
}

fn execute(command: command::Command, gpio_controllers: Vec<Arc<Gpio>>) {
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
                        let pin = gpio_controllers[0].get_pin(8);
                        pin.mode(Output);
                        pin.digital_write(High);

                    }
                    2 => {
                        let pin = gpio_controllers[2].get_pin(16);
                        pin.mode(Output);
                        pin.digital_write(High);
                    }
                    3 => {
                        let pin = gpio_controllers[2].get_pin(17);
                        pin.mode(Output);
                        pin.digital_write(High);

                    }
                    4 => {
                        let pin = gpio_controllers[2].get_pin(25);
                        pin.mode(Output);
                        pin.digital_write(High);
                    }
                    5 => {
                        let pin = gpio_controllers[2].get_pin(1);
                        pin.mode(Output);
                        pin.digital_write(High);
                    }
                    6 => {
                        let pin = gpio_controllers[1].get_pin(14);
                        pin.mode(Output);
                        pin.digital_write(High);
                    }
                    _ => println!("Error"),
                },
                board::ValveState::VALVE_CLOSED => match valve.channel {
                    1 => {
                        let pin = gpio_controllers[0].get_pin(8);
                        pin.mode(Output);
                        pin.digital_write(Low);
                    }
                    2 => {
                        let pin = gpio_controllers[2].get_pin(16);
                        pin.mode(Output);
                        pin.digital_write(Low);
                    }
                    3 => {
                        let pin = gpio_controllers[2].get_pin(17);
                        pin.mode(Output);
                        pin.digital_write(Low);
                    }
                    4 => {
                        let pin = gpio_controllers[2].get_pin(25);
                        pin.mode(Output);
                        pin.digital_write(Low);
                    }
                    5 => {
                        let pin = gpio_controllers[2].get_pin(1);
                        pin.mode(Output);
                        pin.digital_write(Low);
                    }
                    6 => {
                        let pin = gpio_controllers[1].get_pin(14);
                        pin.mode(Output);
                        pin.digital_write(Low);
                    }
                    _ => println!("Error"),
                },
                
            }

        }
        _ => println!("Unknown command"),

    }
}

// For testing only.
// pub fn open_valve(id: u32) {
//     let command = command::Command {
//         command: command::mod_Command::OneOfcommand::click_valve(
//             command::ClickValve { 
//                 valve: (Some(board::ChannelIdentifier {board_id: 10, channel_type: board::ChannelType::VALVE, channel: id})), 
//                 state: (board::ValveState::VALVE_OPEN)
//     })};
//     execute(command);
// }

// pub fn close_valve(id: u32) {
//     let command = command::Command {
//         command: command::mod_Command::OneOfcommand::click_valve(
//             command::ClickValve { 
//                 valve: (Some(board::ChannelIdentifier {board_id: 10, channel_type: board::ChannelType::VALVE, channel: id})), 
//                 state: (board::ValveState::VALVE_CLOSED)
//     })};
//     execute(command);
// }
