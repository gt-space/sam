use std::{net::{SocketAddr, UdpSocket}, sync::Arc, thread, time::Instant};
use common::comm::{DataPoint, DataMessage};
use spidev::{SpiModeFlags, Spidev, SpidevOptions};
use std::rc::Rc;
use hostname;
use std::net::ToSocketAddrs;
use crate::{adc::{self, gpio_controller_mappings, pull_gpios_high, data_ready_mappings, ADC}, 
            data::{generate_data_point, serialize_data}, 
            gpio::Gpio};
use jeflog::{task, pass, fail, warn};
use crate::gpio::{PinMode::Output, PinValue::Low};

const FC_ADDR: &str = "server-01";
const HOSTNAMES: [&str; 1] = [FC_ADDR];

const FC_HEARTBEAT_TIMEOUT: u128 = 500;

pub struct Data {
    pub data_socket: UdpSocket,
    flight_computer: Option<SocketAddr>,
    adcs: Option<Vec<adc::ADC>>,
    state_num: u32,
    curr_measurement: Option<adc::Measurement>,
    data_points: Vec<DataPoint>,
    board_id: Option<String>,
    gpio_controllers: Vec<Arc<Gpio>>
}

impl Data {
    pub fn new(gpio_controllers: Vec<Arc<Gpio>>) -> Data {
        Data {
            data_socket: UdpSocket::bind(("0.0.0.0", 4573)).expect("Could not bind client socket"),
            flight_computer: None,
            adcs: None,
            state_num: 0,
            curr_measurement: None,
            data_points: Vec::with_capacity(60),
            board_id: None,
            gpio_controllers: gpio_controllers
        }
    }
}



#[derive(PartialEq, Debug)]
pub enum State {
    Init,
    DeviceDiscovery,
    Identity,
    InitAdcs,
    PollAdcs
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
                    .max_speed_hz(10_000_000)
                    .lsb_first(false)
                    .mode(SpiModeFlags::SPI_MODE_1)
                    .build();
                spidev.configure(&options).unwrap();

                let ref_spidev: Rc<_> = Rc::new(spidev);
                let ref_controllers = Rc::new(gpio_controller_mappings(&data.gpio_controllers));
                let ref_drdy = Rc::new(data_ready_mappings(&data.gpio_controllers));
        
                // Instantiate all measurement types
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

                pull_gpios_high(&data.gpio_controllers);
                
                data.adcs = Some(adcs);
                data.data_socket.set_nonblocking(true).expect("set_nonblocking call failed");

                data.board_id = get_board_id();

                State::DeviceDiscovery
            }

            State::DeviceDiscovery => {
                task!("Locating the flight computer.");
                
                let address = format!("{}.local:4573", FC_ADDR)
                        .to_socket_addrs()
                        .ok()
                        .and_then(|mut addrs| addrs.find(|addr| addr.is_ipv4()));

                let Some(address) = address else {
                    fail!("Target \x1b[1m{}\x1b[0m could not be located.", FC_ADDR);
                    return State::DeviceDiscovery;
                };

                pass!("Target \x1b[1m{}\x1b[0m located at \x1b[1m{}\x1b[0m.", FC_ADDR, address.ip());
                data.flight_computer = Some(address);

                State::InitAdcs
            }

            State::Identity => {
                let mut buf = [0; 65536];

                if let Some(board_id) = data.board_id.clone() {
                    let identity = DataMessage::Identity(board_id);                    
                    let data_serialized = postcard::to_allocvec(&identity);

                    if let Some(socket_addr) = data.flight_computer {
                        data.data_socket
                        .send_to(&data_serialized.unwrap(), socket_addr)
                        .expect("Could not send Identity message.");
                    } else {
                        fail!("Could not send Identity message.");
                    }
                } else {
                    fail!("Could not send Identity message, invalid board information.");
                }

                match data.data_socket.recv_from(&mut buf) {
                    Ok((num_bytes, _src_addr)) => {
                        let deserialized_result = postcard::from_bytes::<DataMessage>(&buf[..num_bytes]);
                        println!("{:#?}", deserialized_result);
                        match deserialized_result {
                            Ok(message) => {
                                match message {
                                    // FC sends identity back 
                                    DataMessage::Identity(_) => {
                                        pass!("Received Identity message from the flight computer, monitoring heartbeat");
    
                                        let socket_copy = data.data_socket.try_clone();
                                        let controllers = data.gpio_controllers.clone();

                                        // Spawn heartbeat thread
                                        thread::spawn(move || {
                                            monitor_heartbeat(socket_copy.ok().unwrap(), &controllers);
                                        });

                                        return State::PollAdcs;
                                    },
                                    _ => { warn!("Received unexpected message from the flight computer"); return State::Identity; } ,
                                }
                            },
                            Err(_error) => {fail!("Bad message from flight computer"); return State::Identity; },
                        };
                    }
                    Err(_) => {
                        ();
                    }
                };
                State::Identity
            }

            State::InitAdcs => {
                for adc in data.adcs.as_mut().unwrap() {
                    adc.init_gpio(data.curr_measurement);
                    data.curr_measurement = Some(adc.measurement);
                    adc.reset_status();

                    adc.init_regs();
                    adc.start_conversion();

                    adc.write_iteration(0);
                }
                
                pass!("Initialized ADCs");
                State::Identity
            }

            State::PollAdcs => {
                data.data_points.clear();
                
                for i in 0..6 {
                    for adc in data.adcs.as_mut().unwrap() {
                        if (i > 2 && adc.measurement == adc::Measurement::DiffSensors) || 
                           (i > 4 && adc.measurement == adc::Measurement::VPower) ||
                           (i > 1 && (adc.measurement == adc::Measurement::IPower || adc.measurement == adc::Measurement::Rtd)) ||
                           (i > 3 && (adc.measurement ==  adc::Measurement::Tc1 || adc.measurement ==  adc::Measurement::Tc2)) {
                            continue;
                        }

                        adc.init_gpio(data.curr_measurement);
                        data.curr_measurement = Some(adc.measurement);
                        
                        // Read ADC
                        let (raw_value, unix_timestamp) = adc.get_adc_reading(i);
                        
                        // Write ADC for next iteration
                        adc.write_iteration(i + 1);
                        
                        // Don't add ambient temp reading to FC message 
                        if  i == 0 && (adc.measurement ==  adc::Measurement::Tc1 || adc.measurement ==  adc::Measurement::Tc2) {
                            continue;
                        }

                        let data_point = generate_data_point(
                            raw_value, 
                            unix_timestamp, 
                            i,
                            adc.measurement.clone(), 
                        );
    
                        data.data_points.push(data_point)
                    }
                }
                
                if let Some(board_id) = data.board_id.clone() {
                    let serialized = serialize_data(board_id, &data.data_points);

                    if let Some(socket_addr) = data.flight_computer {
                        data.data_socket
                        .send_to(&serialized.unwrap(), socket_addr)
                        .expect("couldn't send data to flight computer");
                    }
                }
                State::PollAdcs
            }
        }
    }
}

fn monitor_heartbeat(socket: UdpSocket, gpio_controllers: &Vec<Arc<Gpio>>) {
    let mut buf = [0; 65536];
    let mut last_heartbeat = Instant::now();

    loop {
        let curr_time = Instant::now();
        let time_elapsed = curr_time.duration_since(last_heartbeat).as_millis();

        if time_elapsed > FC_HEARTBEAT_TIMEOUT {
            // Abort system if loss of comms detected 
            break
        }

        // monitor socket for heartbeat messages
        match socket.recv_from(&mut buf) {
            Ok((num_bytes, _src_addr)) => {
                let deserialized_result = postcard::from_bytes::<DataMessage>(&buf[..num_bytes]);
                match deserialized_result {
                    Ok(message) => {
                        match message {
                            // FC sends identity back 
                            DataMessage::FlightHeartbeat => {
                                let new_hb = Instant::now();
                                last_heartbeat = new_hb;
                            },
                            _ => { }
                        }
                    } Err(_) => {
                        fail!("Failed to deserialize DataMessage from flight computer.")
                    }
                }
            }
            Err(_) => {
                ();
            }
        }    
    }
    abort(gpio_controllers);
}

fn abort(controllers: &Vec<Arc<Gpio>>) {
    fail!("Aborting the SAM Board.");
    warn!("You must manually restart SAM software.");

    let pins = vec![controllers[0].get_pin(8),  // valve 1
                    controllers[2].get_pin(16), // valve 2
                    controllers[2].get_pin(17), // valve 3
                    controllers[2].get_pin(25), // valve 4
                    controllers[2].get_pin(1),  // valve 5
                    controllers[1].get_pin(14)];// valve 6

    for pin in pins.iter() {
        pin.mode(Output);
        pin.digital_write(Low);
    }
}

fn get_board_id() -> Option<String> {
    match hostname::get() {
        Ok(hostname) => {
            let name = hostname.to_string_lossy().to_string();
            Some(name)
        }
        Err(e) => {
            fail!("Error getting board ID for Establish message: {}", e);
            None
        }
    }
}
