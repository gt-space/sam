use std::{collections::HashMap, net::{IpAddr, SocketAddr, UdpSocket}, sync::Arc};
use spidev::{SpiModeFlags, Spidev, SpidevOptions};
use std::rc::Rc;
use crate::{discovery::get_ips, adc::{self, gpio_controller_mappings, pull_gpios_high, data_ready_mappings}, data::data_loop::data_message_formation, gpio::Gpio};

// const FC_ADDR: &str = "flight-computer-01.local";
const FC_ADDR: &str = "patrick-XPS-15-9500.local";
const HOSTNAMES: [&str; 1] = [FC_ADDR];

pub struct Data {
    ip_addresses: HashMap<String, Option<IpAddr>>,
    pub data_socket: UdpSocket,
    flight_computer: Option<SocketAddr>,
    adcs: Option<Vec<adc::ADC>>,
    state_num: u32,
    curr_measurement: Option<adc::Measurement>,
    curr_iteration: u64
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
            curr_iteration: 0
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
    // SendData,
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
                    .max_speed_hz(1000000)
                    .lsb_first(false)
                    .mode(SpiModeFlags::SPI_MODE_1)
                    .build();
                spidev.configure(&options).unwrap();

                let ref_spidev: Rc<_> = Rc::new(spidev);
                let ref_controllers = Rc::new(gpio_controller_mappings(controllers));
                let ref_drdy = Rc::new(data_ready_mappings(controllers));
        
                //let adc_ds = adc::ADC::new(adc::Measurement::DiffSensors, ref_spidev.clone(), ref_controllers.clone(), ref_drdy.clone());
                let adc_cl = adc::ADC::new(adc::Measurement::CurrentLoopPt, ref_spidev.clone(), ref_controllers.clone(), ref_drdy.clone());
                //let board_power = adc::ADC::new(adc::Measurement::VPower, ref_spidev.clone(), ref_controllers.clone(), ref_drdy.clone());
                //let board_current = adc::ADC::new(adc::Measurement::IPower, ref_spidev.clone(), ref_controllers.clone(), ref_drdy.clone());
                //let adc_valve = adc::ADC::new(adc::Measurement::VValve, ref_spidev.clone(), ref_controllers.clone(), ref_drdy.clone());
                //let adc_tc2 = adc::ADC::new(adc::Measurement::Tc2, ref_spidev.clone(), ref_controllers.clone(), ref_drdy.clone());

                let mut adcs: Vec<adc::ADC> = Vec::new();
 
                adcs.push(adc_cl);
                //adcs.push(board_power);
                //adcs.push(board_current);
                //adcs.push(adc_valve);
                //adcs.push(adc_tc2);

                pull_gpios_high(controllers);
                
                data.adcs = Some(adcs);
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
                
                return State::InitAdcs
            }

            State::InitAdcs => {
                for adc in data.adcs.as_mut().unwrap() {
                    adc.init_gpio(data.curr_measurement);
                    data.curr_measurement = Some(adc.measurement);
                    println!("Resetting ADC");
                    adc.reset_status();

                    adc.init_regs();
                    adc.start_conversion();

                    adc.write_iteration(data.curr_iteration);
                }
                data.curr_iteration += 1;
                State::PollAdcs
            }

            State::PollAdcs => {
                for adc in data.adcs.as_mut().unwrap() {
                    adc.init_gpio(data.curr_measurement);
                    data.curr_measurement = Some(adc.measurement);
                    let mut measurement: Vec<f64> = Vec::new();
                    
                    // Read ADC
                    let data_serialized = adc.get_adc_reading(data.curr_iteration);
                    measurement.push(data_serialized);

                    // Write ADC for next iteration
                    adc.write_iteration(data.curr_iteration);
                
                    let message = data_message_formation(adc.measurement.clone(), measurement, data.curr_iteration - 1);

                    if let Some(socket_addr) = data.flight_computer {
                        data.data_socket
                        .send_to(&message, socket_addr)
                        .expect("couldn't send data");
                    }
                }
                data.curr_iteration += 1;
                State::PollAdcs
            }
        }
    }
}