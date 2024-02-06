use common::comm::SamControlMessage;
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
        let deserialized_result = postcard::from_bytes::<SamControlMessage>(&buf[..num_bytes]);
        println!("{:#?}", deserialized_result);
        match deserialized_result {
            Ok(message) => {
                execute(message, gpio_controllers.clone());
            },
            Err(_error) => println!("Bad"),
        };
    }
}

fn execute(command: SamControlMessage, gpio_controllers: Vec<Arc<Gpio>>) {
    match command {
       SamControlMessage::SetLed { channel, on } => {
            //let led = set_led_command.led.unwrap();
            match on {
                true => match channel {
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
                false => match channel {
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

        SamControlMessage::ActuateValve { channel, open } => {
            match open {
                true => match channel {
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
                false => match channel {
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
    }
}
