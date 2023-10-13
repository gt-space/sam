use ::core::time;
use std::{collections::HashMap, net::{IpAddr, SocketAddr, UdpSocket}, thread::{self, sleep}, time::{Instant, Duration}};
use quick_protobuf::{deserialize_from_slice, Error};
use spidev::{SpiModeFlags, Spidev, SpidevOptions};
use std::cell::RefCell;
use std::rc::Rc;
use crate::{discovery::get_ips, adc};

use fs_protobuf_rust::compiled::mcfs::core;

const FC_ADDR: &str = "flight-computer-01.local";
const HOSTNAMES: [&str; 1] = [FC_ADDR];

pub struct Data {
    ip_addresses: HashMap<String, Option<IpAddr>>,
    pub data_socket: UdpSocket,
    flight_computer: Option<SocketAddr>,
    adc: Option<adc::ADC>,
    state_num: u32,
}

impl Data {
    pub fn new() -> Data {
        Data {
            ip_addresses: HashMap::new(),
            data_socket: UdpSocket::bind(("0.0.0.0", 4573)).expect("Could not bind client socket"),
            flight_computer: None,
            adc: None,
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
        if data.state_num % 100000 == 0 {
            println!("{:?} {}", self, data.state_num);
        }
        data.state_num += 1;

        match self {
            State::Init => {
                /* Create a spidev wrapper to work with
                you call this wrapper to handle and all transfers */
                let mut spidev = Spidev::open("/dev/spidev0.0").unwrap();

                let options = SpidevOptions::new()
                    .bits_per_word(8)
                    .max_speed_hz(100000)
                    .lsb_first(false)
                    .mode(SpiModeFlags::SPI_MODE_1)
                    .build();
                spidev.configure(&options).unwrap();

                let ref_spidev: Rc<RefCell<_>> = Rc::new(RefCell::new(spidev));
                let adc_differential = adc::ADC::new(adc::Measurement::CurrentLoopPt, ref_spidev.clone());
                data.adc = Some(adc_differential);
                data.data_socket.set_nonblocking(true).expect("set_nonblocking call failed");

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
                data.adc.as_mut().unwrap().init_gpio();
                println!("Resetting ADC");
                data.adc.as_mut().unwrap().reset_status();

                data.adc.as_mut().unwrap().init_regs();
                data.adc.as_mut().unwrap().start_conversion();

                State::SendData
            }

            State::SendData => { 
                let data_serialized = data.adc.as_mut().unwrap().test_read_all();
                let interval = Duration::from_micros(1000000 / 500);
                let mut next_time = Instant::now() + interval;
                //let data_serialized = data_message_formation(data.clone());
                //println!("{:?}", data_serialized);
                if let Some(socket_addr) = data.flight_computer {
                    data.data_socket
                    .send_to(&data_serialized, socket_addr)
                    .expect("couldn't send data");
                }
            
                sleep(next_time - Instant::now());
                next_time += interval; 
        
                State::SendData
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