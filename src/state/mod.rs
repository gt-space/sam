use ::core::time;
use std::{collections::HashMap, net::{IpAddr, SocketAddr, UdpSocket}, io::{Read, self}, thread::{self, sleep}, time::{Instant, Duration}};
use quick_protobuf::{deserialize_from_slice, Error};

use crate::{discovery::get_ips, adc};

use fs_protobuf_rust::compiled::mcfs::core;

const FC_ADDR: &str = "flight-computer-01.local";
const HOSTNAMES: [&str; 1] = [FC_ADDR];

pub struct Data {
    ip_addresses: HashMap<String, Option<IpAddr>>,
    pub data_socket: UdpSocket,
    flight_computer: Option<SocketAddr>,
    adc: adc::ADC,
    state_num: u32,
}

impl Data {
    pub fn new(mut adc: adc::ADC) -> Data {
        Data {
            ip_addresses: HashMap::new(),
            data_socket: UdpSocket::bind(("0.0.0.0", 4573)).expect("Could not bind client socket"),
            flight_computer: None,
            adc: adc,
            state_num: 0,
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum State {
    Init,
    DeviceDiscovery,
    ConnectToFc,
    InitADC,
    SendData,
}

impl State {
    pub fn next(self, data: &mut Data) -> State {

        println!("{:?} {}", self, data.state_num);
        data.state_num += 1;

        match self {
            State::Init => {
                State::DeviceDiscovery
            }

            State::DeviceDiscovery => {
                data.ip_addresses = get_ips(&HOSTNAMES);
                if let Some(ip) = data.ip_addresses.get(FC_ADDR) {
                    match ip {
                        Some(_ipv4_addr) => {
                            State::ConnectToFc
                        },
                        None => {
                            State::DeviceDiscovery
                        }
                    }
                } else {
                    State::DeviceDiscovery
                }
            }

            State::ConnectToFc => {
                let fc_addr = data.ip_addresses.get(FC_ADDR).unwrap().unwrap();
                let socket_addr = SocketAddr::new(fc_addr, 4573);
                
                data.flight_computer = Some(socket_addr);
                
                return State::InitADC
            }

            State::InitADC => {
                data.adc.init_gpio();
                println!("Resetting ADC");
                data.adc.reset_status();

                // delay for at least 4000*clock period
                println!("Delaying for 1 second");
                thread::sleep(time::Duration::from_millis(1000));

                data.adc.init_regs();
                data.adc.start_conversion();

                return State::SendData
            }

            State::SendData => {
                let data_serialized = data.adc.test_read_all();
                thread::sleep(time::Duration::from_millis(1000));
                let interval = Duration::from_micros(1000000 / 500);
                let mut next_time = Instant::now() + interval;
                // let data_serialized = data_message_formation(data.clone());
                
                if let Some(socket_addr) = data.flight_computer {
                    data.data_socket
                    .send_to(&data_serialized, socket_addr)
                    .expect("couldn't send data");
                }
            
                sleep(next_time - Instant::now());
                next_time += interval; 

                return State::SendData
            }
        }
    }
    
}

fn receive(socket: &UdpSocket) {
    let mut buf = [0; 65536];

    match socket.recv_from(&mut buf) {
        Ok((_n, _src)) => {
            let deserialized: Result<core::Message, Error> = deserialize_from_slice(&buf);
            println!("{:?}", deserialized);
        }
        Err(_err) => {
            return;
        }
    }
}