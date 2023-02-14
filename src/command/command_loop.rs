use fs_protobuf_rust::compiled::mcfs::command;
use fs_protobuf_rust::compiled::mcfs::core;
use fs_protobuf_rust::compiled::mcfs::data;
use fs_protobuf_rust::compiled::mcfs::device;
use fs_protobuf_rust::compiled::google::protobuf::Timestamp;
use quick_protobuf::Error;
use quick_protobuf::{serialize_into_vec, deserialize_from_slice};
use std::borrow::Cow;
use std::fs::File;

use std::io::Write;
use std::net::UdpSocket;

pub fn begin() {
    let mut buf = [0; 65536];
    let socket = UdpSocket::bind("0.0.0.0:34254").expect("Could not bind socket");
    loop {
        let (num_bytes, _src_addr) = socket.recv_from(&mut buf).expect("no data received");
        
        let deserialized_result: Result<core::Message, Error> = deserialize_from_slice(&buf);
        println!("{:#?}", deserialized_result);
        match deserialized_result {
            Ok(message) => {
                match message.content {
                    core::mod_Message::OneOfcontent::command(command) => execute(command),
                    core::mod_Message::OneOfcontent::data(..) => println!("Data"),
                    core::mod_Message::OneOfcontent::status(..) => println!("Command"),
                    _ => println!("Other")  
                }
            },
            Err(error) => println!("Bad")
        };
    }
}

fn execute(command: command::Command) {
    match command.command {
        command::mod_Command::OneOfcommand::set_led(set_led_command) => {
            let led = set_led_command.led.unwrap();
            match set_led_command.state {
                device::LEDState::LED_ON => {
                    match led.node_id {
                        0 => {
                            let mut file: File = std::fs::OpenOptions::new().write(true).truncate(true).open("/sys/class/leds/beaglebone:green:usr0/brightness").unwrap();
                            file.write(b"1").expect("Failed to write");
                        }
                        1 => {
                            let mut file: File = std::fs::OpenOptions::new().write(true).truncate(true).open("/sys/class/leds/beaglebone:green:usr1/brightness").unwrap();
                            file.write(b"1").expect("Failed to write");
                        }
                        2 => {
                            let mut file: File = std::fs::OpenOptions::new().write(true).truncate(true).open("/sys/class/leds/beaglebone:green:usr2/brightness").unwrap();
                            file.write(b"1").expect("Failed to write");
                        }
                        3 => {
                            let mut file: File = std::fs::OpenOptions::new().write(true).truncate(true).open("/sys/class/leds/beaglebone:green:usr3/brightness").unwrap();
                            file.write(b"1").expect("Failed to write");
                        }
                        _ => println!("Error")
                    }
                }
                device::LEDState::LED_OFF => {
                    match led.node_id {
                        0 => {
                            let mut file: File = std::fs::OpenOptions::new().write(true).truncate(true).open("/sys/class/leds/beaglebone:green:usr0/brightness").unwrap();
                            file.write(b"0").expect("Failed to write");
                        }
                        1 => {
                            let mut file: File = std::fs::OpenOptions::new().write(true).truncate(true).open("/sys/class/leds/beaglebone:green:usr1/brightness").unwrap();
                            file.write(b"0").expect("Failed to write");
                        }
                        2 => {
                            let mut file: File = std::fs::OpenOptions::new().write(true).truncate(true).open("/sys/class/leds/beaglebone:green:usr2/brightness").unwrap();
                            file.write(b"0").expect("Failed to write");
                        }
                        3 => {
                            let mut file: File = std::fs::OpenOptions::new().write(true).truncate(true).open("/sys/class/leds/beaglebone:green:usr3/brightness").unwrap();
                            file.write(b"0").expect("Failed to write");
                        }
                        _ => println!("Error")
                    }
                }
            }
        },
        _ => println!("Unknown command")
    }
}