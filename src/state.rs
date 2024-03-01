use std::{collections::HashMap, net::{IpAddr, SocketAddr, UdpSocket}, sync::Arc};
use common::comm::DataPoint;
use spidev::{SpiModeFlags, Spidev, SpidevOptions};
use std::rc::Rc;
use crate::{discovery::get_ips, 
            adc::{self, gpio_controller_mappings, pull_gpios_high, data_ready_mappings, ADC}, 
            data::{generate_data_point, serialize_data}, 
            gpio::Gpio};
use jeflog::{task, pass, fail};

const FC_ADDR: &str = "server-01.local";
const HOSTNAMES: [&str; 1] = [FC_ADDR];

pub struct Data {
    ip_addresses: HashMap<String, Option<IpAddr>>,
    pub data_socket: UdpSocket,
    flight_computer: Option<SocketAddr>,
    adcs: Option<Vec<adc::ADC>>,
    state_num: u32,
    curr_measurement: Option<adc::Measurement>,
    curr_iteration: u64,
    data_points: Vec<DataPoint>
}

impl Data {
    pub fn new() -> Data {
        Data {
            ip_addresses: HashMap::new(),
            data_socket: UdpSocket::bind(("0.0.0.0", 4573)).expect("Could not bind client socket"),
            flight_computer: None,
            adcs: None,
            state_num: 0,
            curr_measurement: None,
            curr_iteration: 0,
            data_points: Vec::with_capacity(9)
        }
    }
}



#[derive(PartialEq, Debug)]
pub enum State {
    Init,
    DeviceDiscovery,
    ConnectToFc,
    InitAdcs,
    PollAdcs,
}

impl State {
    pub fn next(self, data: &mut Data, controllers: &Vec<Arc<Gpio>>) -> State {
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
                    .max_speed_hz(10_000_000)
                    .lsb_first(false)
                    .mode(SpiModeFlags::SPI_MODE_1)
                    .build();
                spidev.configure(&options).unwrap();

                let ref_spidev: Rc<_> = Rc::new(spidev);
                let ref_controllers = Rc::new(gpio_controller_mappings(controllers));
                let ref_drdy = Rc::new(data_ready_mappings(controllers));
        
                let ds = ADC::new(adc::Measurement::DiffSensors, ref_spidev.clone(), ref_controllers.clone(), ref_drdy.clone());
                let cl = ADC::new(adc::Measurement::CurrentLoopPt, ref_spidev.clone(), ref_controllers.clone(), ref_drdy.clone());
                let board_power = ADC::new(adc::Measurement::VPower, ref_spidev.clone(), ref_controllers.clone(), ref_drdy.clone());
                let board_current = ADC::new(adc::Measurement::IPower, ref_spidev.clone(), ref_controllers.clone(), ref_drdy.clone());
                let vvalve = ADC::new(adc::Measurement::VValve, ref_spidev.clone(), ref_controllers.clone(), ref_drdy.clone());
                let ivalve = ADC::new(adc::Measurement::IValve, ref_spidev.clone(), ref_controllers.clone(), ref_drdy.clone());
                let rtd = ADC::new(adc::Measurement::Rtd, ref_spidev.clone(), ref_controllers.clone(), ref_drdy.clone());
                let tc1 = ADC::new(adc::Measurement::Tc1, ref_spidev.clone(), ref_controllers.clone(), ref_drdy.clone());
                let tc2 = ADC::new(adc::Measurement::Tc2, ref_spidev.clone(), ref_controllers.clone(), ref_drdy.clone());

                let mut adcs: Vec<adc::ADC> = Vec::with_capacity(9);
 
                adcs.push(ds);
                adcs.push(cl);
                adcs.push(board_power);
                adcs.push(board_current);
                adcs.push(vvalve);
                adcs.push(ivalve);
                adcs.push(rtd);
                adcs.push(tc1);
                adcs.push(tc2);

                pull_gpios_high(controllers);
                
                data.adcs = Some(adcs);
                data.data_socket.set_nonblocking(true).expect("set_nonblocking call failed");

                State::DeviceDiscovery
            }

            State::DeviceDiscovery => {
                task!("Locating flight computer.");
                data.ip_addresses = get_ips(&HOSTNAMES);
                if let Some(ip) = data.ip_addresses.get(FC_ADDR) {
                    match ip {
                        Some(_ipv4_addr) => {
                            pass!("Found the flight computer at: {}", _ipv4_addr.to_string());
                            State::ConnectToFc
                        },
                        None => {
                            State::DeviceDiscovery
                        }
                    }
                } else {
                    fail!("Failed to locate the flight computer. Retrying.");
                    State::DeviceDiscovery
                }
            }

            State::ConnectToFc => {
                let fc_addr = data.ip_addresses.get(FC_ADDR).unwrap().unwrap();
                let socket_addr = SocketAddr::new(fc_addr, 4573);
                
                data.flight_computer = Some(socket_addr);
                
                return State::InitAdcs
            }

            State::InitAdcs => {
                for adc in data.adcs.as_mut().unwrap() {
                    adc.init_gpio(data.curr_measurement);
                    data.curr_measurement = Some(adc.measurement);
                    adc.reset_status();

                    adc.init_regs();
                    adc.start_conversion();

                    adc.write_iteration(data.curr_iteration);
                }
                data.curr_iteration += 1;
                
                State::PollAdcs
            }

            State::PollAdcs => {
                data.data_points.clear();
                for adc in data.adcs.as_mut().unwrap() {
                    adc.init_gpio(data.curr_measurement);
                    data.curr_measurement = Some(adc.measurement);
                    
                    // Read ADC
                    let (raw_value, unix_timestamp) = adc.get_adc_reading(data.curr_iteration);

                    // Write ADC for next iteration
                    adc.write_iteration(data.curr_iteration);
                
                    let data_point = generate_data_point(
                        raw_value, 
                        unix_timestamp, 
                        data.curr_iteration - 1,
                        adc.measurement.clone(), 
                    );

                    data.data_points.push(data_point)
                }

                let serialized = serialize_data(&data.data_points);

                if let Some(socket_addr) = data.flight_computer {
                    data.data_socket
                    .send_to(&serialized.unwrap(), socket_addr)
                    .expect("couldn't send data to flight computer");
                }

                data.curr_iteration += 1;
                State::PollAdcs
            }
        }
    }
}