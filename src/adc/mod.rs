use spidev::spidevioctl::SpidevTransfer;
use spidev::Spidev;
use std::sync::Arc;
use std::time::Instant;
use std::{thread, time};

use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::gpio::{Gpio, Pin, PinMode::Output, PinValue::{High, Low}};
use crate::tc::type_k_tables::typek_convert;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Measurement {
    CurrentLoopPt,
    VValve,
    IValve,
    VPower,
    IPower,
    Tc1,
    Tc2,
    DiffSensors,
    Rtd
}

pub struct ADC {
    pub measurement: Measurement,
    pub spidev: Rc<RefCell<Spidev>>,
    //drdy_mappings: HashMap<Measurement, usize>,
    ambient_temp: f64,
    gpio_mappings: Rc<HashMap<Measurement, Pin>>
}

impl ADC {
    // Constructs a new instance of an Analog-to-Digital Converter 
    pub fn new(measurement: Measurement, spidev: Rc<RefCell<Spidev>>, gpio_mappings: Rc<HashMap<Measurement, Pin>>) -> ADC {
        ADC {
            measurement: measurement,
            spidev: spidev,
            //drdy_mappings: Self::drdy_mappings(),
            ambient_temp: 0.0,
            //gpio_mappings: Self::gpio_controller_mappings()
            gpio_mappings: gpio_mappings
        }
    }

    pub fn cs_mappings() -> HashMap<Measurement, usize> {
        let mut cs_gpios: HashMap<Measurement, usize> = HashMap::new();
        cs_gpios.insert(Measurement::CurrentLoopPt, 30);
        cs_gpios.insert(Measurement::IValve, 4);
        cs_gpios.insert(Measurement::VValve, 26);
        cs_gpios.insert(Measurement::VPower, 13);
        cs_gpios.insert(Measurement::IPower, 15);
        cs_gpios.insert(Measurement::Tc1, 10); 
        cs_gpios.insert(Measurement::Tc2, 20);
        cs_gpios.insert(Measurement::DiffSensors, 16); 
        cs_gpios.insert(Measurement::Rtd, 11);

        cs_gpios
    }

    // pub fn drdy_mappings() -> HashMap<Measurement, (usize, usize)> {
    //     let mut drdy_gpios: HashMap<Measurement, (usize, usize)> = HashMap::new();

    //     drdy_gpios.insert(Measurement::CurrentLoopPt, (1, 28));
    //     drdy_gpios.insert(Measurement::IValve, ());
    //     drdy_gpios.insert(Measurement::VValve, 44);
    //     drdy_gpios.insert(Measurement::VPower, 76);
    //     drdy_gpios.insert(Measurement::IPower, 78);
    //     drdy_gpios.insert(Measurement::Tc1, 0);
    //     drdy_gpios.insert(Measurement::Tc2, 0);
    //     drdy_gpios.insert(Measurement::Rtd, 0);
    //     drdy_gpios.insert(Measurement::DiffSensors, 111); // this should be 112

    //     drdy_gpios
    // }

    pub fn init_gpio(&mut self, prev_adc: Option<Measurement>) { 
        // pull old adc HIGH
        if let Some(old_adc) = prev_adc {
            if let Some(pin) = self.gpio_mappings.get(&old_adc) {
                //pin.mode(Output);
                pin.digital_write(High);
            } 
        }

        // pull new adc LOW
        if let Some(pin) = self.gpio_mappings.get(&self.measurement) {
            //pin.mode(Output);
            pin.digital_write(Low);
        }
    }

    // pub fn poll_data_ready(&mut self) {
    //     // poll the data ready pin till low (active low)
    //     let drdy_pin = self.gpio_controller.get(&self.measurement).unwrap();


    //     loop {
    //         let pin_value = drdy_pin.digital_read();
    //         if pin_value == Low {
    //             break;
    //         }
    //     }

    //     // if let Some(my_drdy) = self.drdy_mappings.get(&self.measurement) {
    //     //     let gpio_str: &str = &format!("{}", my_drdy);
    //     //     loop {
    //     //         let value = gpio::read_gpio_value(gpio_str);
    //     //         if value == 0 {
    //     //             break;
    //     //         }
    //     //     }
    //     // }
    // }

    pub fn init_regs(&mut self) {
        // Read initial registers
        println!("Reading initial register states");
        self.read_regs(0, 17);

        // delay for at least 4000*clock period
        // println!("Delaying for 1 second");
        thread::sleep(time::Duration::from_millis(100));

        // Write to registers
        println!("Writing to registers");

        match self.measurement {
            Measurement::CurrentLoopPt | 
            Measurement::IValve |
            Measurement::VValve | 
            Measurement::VPower |
            Measurement::IPower => {
                self.write_reg(0x03, 0x00);
                self.write_reg(0x04, 0x1E);
                // self.write_reg(0x08, 0x40);
                self.write_reg(0x08, 0x00);
                self.write_reg(0x05, 0x0A);
                println!("here");
            }

            Measurement::Rtd => {
                self.write_reg(0x03, 0x00);
                self.write_reg(0x04, 0x1E);
                self.write_reg(0x06, 0x47);
                //self.write_reg(0x06, 0x07);
                self.write_reg(0x07, 0x05);
            }

            Measurement::Tc1 | 
            Measurement::Tc2 | 
            Measurement::DiffSensors => {
                self.write_reg(0x03, 0x0D);
                self.write_reg(0x04, 0x1E);
                self.write_reg(0x05, 0x0A);
            }
        }

        // delay for at least 4000*clock period
        // println!("Delaying for 1 second");
        thread::sleep(time::Duration::from_millis(100));

        // Read registers
        println!("Reading new register states");
        self.read_regs(0, 17);
    }
    
    pub fn reset_status(&mut self) {
        let tx_buf_reset = [0x06];
        let mut transfer = SpidevTransfer::write(&tx_buf_reset);
        let _status = self.spidev.borrow_mut().transfer(&mut transfer);
    }

    pub fn start_conversion(&mut self) {
        let mut tx_buf_rdata = [ 0x08];
        let mut rx_buf_rdata = [ 0x00];
        let mut transfer = SpidevTransfer::read_write(&mut tx_buf_rdata, &mut rx_buf_rdata);
        let _status = self.spidev.borrow_mut().transfer(&mut transfer);
        //thread::sleep(time::Duration::from_millis(1000));
        thread::sleep(time::Duration::from_millis(1));
    }

    pub fn self_calibrate(&mut self) {
        let mut tx_buf_rdata = [ 0x19];
        let mut rx_buf_rdata = [ 0x00];
        let mut transfer = SpidevTransfer::read_write(&mut tx_buf_rdata, &mut rx_buf_rdata);
        let _status = self.spidev.borrow_mut().transfer(&mut transfer);
        thread::sleep(time::Duration::from_millis(1000));
    }
    
    pub fn read_regs(&mut self, reg: u8, num_regs: u8) {
        let mut tx_buf_readreg = [ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ];
        let mut rx_buf_readreg = [ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ];
        tx_buf_readreg[0] = 0x20 | reg;
        tx_buf_readreg[1] = num_regs;
        let mut transfer = SpidevTransfer::read_write(&mut tx_buf_readreg, &mut rx_buf_readreg);
        let _status = self.spidev.borrow_mut().transfer(&mut transfer);
        println!("data: {:?}", rx_buf_readreg);
    }
    
    pub fn write_reg(&mut self, reg: u8, data: u8) {
        let mut tx_buf_writereg = [ 0x40, 0x00, 0x00 ];
        let mut rx_buf_writereg = [ 0x40, 0x00, 0x00 ];
        tx_buf_writereg[0] = 0x40 | reg;
        tx_buf_writereg[2] = data;
        let mut transfer = SpidevTransfer::read_write(&mut tx_buf_writereg, &mut rx_buf_writereg);
        let _status = self.spidev.borrow_mut().transfer(&mut transfer);
    }

    pub fn get_adc_reading(&mut self, iteration: u64) -> f64 {
        // if  self.measurement == Measurement::Rtd || 
        //     self.measurement == Measurement::Tc1 || 
        //     self.measurement == Measurement::Tc2 {
        //         // can't use data ready for these
        //         thread::sleep(time::Duration::from_micros(800));
        //     }
        // else {
        //     self.poll_data_ready();
        // }
        thread::sleep(time::Duration::from_micros(700));
        let val = self.test_read_individual(iteration - 1).try_into().unwrap();

        // if ((iteration % 4 == 1) && 
        //     (self.measurement == Measurement::Tc1 || self.measurement == Measurement::Tc2)) {
        //         //self.ambient_temp = val;

        //         // disable sys monitor 
        //         self.write_reg(0x09, 0x10);
        //         // set pga gain to 32
        //         self.write_reg(0x03, 0x0D);
        // }

        val
    }

    pub fn write_iteration(&mut self, iteration: u64) {
        match self.measurement {
            Measurement::CurrentLoopPt | 
            Measurement::IValve |
            Measurement::VValve => {
                match iteration % 2 {
                    0 => { self.write_reg(0x02, 0x00 | 0x0C); }
                    1 => { self.write_reg(0x02, 0x10 | 0x0C); }
                    // 2 => { self.write_reg(0x02, 0x20 | 0x0C); }
                    // 3 => { self.write_reg(0x02, 0x30 | 0x0C); }
                    // 4 => { self.write_reg(0x02, 0x40 | 0x0C); }
                    // 5 => { self.write_reg(0x02, 0x50 | 0x0C); }
                    _ => println!("Failed register write — could not mod iteration")
                }
            }
            Measurement::VPower => {
                match iteration % 5 {
                    0 => { self.write_reg(0x02, 0x00 | 0x0C); }
                    1 => { self.write_reg(0x02, 0x10 | 0x0C); }
                    2 => { self.write_reg(0x02, 0x20 | 0x0C); }
                    3 => { self.write_reg(0x02, 0x30 | 0x0C); }
                    4 => { self.write_reg(0x02, 0x40 | 0x0C); }
                    _ => println!("Failed register write — could not mod iteration")
                }
            }
            Measurement::IPower => {
                match iteration % 2 {
                    0 => { self.write_reg(0x02, 0x00 | 0x0C); }
                    1 => { self.write_reg(0x02, 0x10 | 0x0C); }
                    _ => println!("Failed register write — could not mod iteration")
                }
            }
            Measurement::Rtd => {
                match iteration % 2 {
                    0 => { self.write_reg(0x02, 0x45); self.write_reg(0x05, 0x10); } 
                    1 => { self.write_reg(0x02, 0x21); self.write_reg(0x05, 0x14); } 
                    _ => println!("Failed register write — could not mod iteration")
                }
            }

            Measurement::DiffSensors => {
                match iteration % 3 {
                    0 => { self.write_reg(0x02, 0x54); }
                    1 => { self.write_reg(0x02, 0x32); }
                    2 => { self.write_reg(0x02, 0x10); }
                    _ => println!("Failed register write — could not mod iteration")
                }
            }

            Measurement::Tc1 |
            Measurement::Tc2 => {
                match iteration % 3 {
                    // 0 => { self.write_reg(0x09, 0x40); self.write_reg(0x03, 0x0A); } 
                    0 => { self.write_reg(0x02, 0x10 | 0x00); self.write_reg(0x08, 0x02); }
                    1 => { self.write_reg(0x02, 0x30 | 0x02); self.write_reg(0x08, 0x08); }
                    2 => { self.write_reg(0x02, 0x50 | 0x04); self.write_reg(0x08, 0x20); }
                    _ => println!("Failed register write — could not mod iteration")
                }
            }
        }
    }

    pub fn test_read_individual(&mut self, iteration: u64) -> f64 {
        let mut tx_buf_rdata = [ 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ];
        let mut rx_buf_rdata = [ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ];
        let mut transfer = SpidevTransfer::read_write(&mut tx_buf_rdata, &mut rx_buf_rdata);
        let _status = self.spidev.borrow_mut().transfer(&mut transfer);
        let value: i16 = ((rx_buf_rdata[1] as i16) << 8) | (rx_buf_rdata[2] as i16);
        let value2: f64 = value as f64;

        let mut reading = value2;
        //println!("{:?}", Instant::now());

        if self.measurement == Measurement::CurrentLoopPt {
            reading = ((value as i32 + 32768) as f64) * (2.5 / (2u64.pow(15) as f64)) / 200.0;
            //println!("CL {}: {} ", iteration%6, reading);
        }
        if self.measurement == Measurement::VPower { 
            reading = ((value as i32) as f64) * (2.5 / (2u64.pow(15) as f64)) * 11.0; // 0 ref
            //println!("PWR {}: {} ", iteration%5, reading);
        }
        if self.measurement == Measurement::IPower {
            reading = ((value as i32 + 32768) as f64) * (2.5 / (2u64.pow(15) as f64)); // 2.5 ref
            //println!("CURR {}: {} ", iteration%2, reading);
        }
        if self.measurement == Measurement::Rtd {
            reading = ((value as i32) as f64) * (2.5 / (2u64.pow(15) as f64)); // 2.5 ref
        }
        if self.measurement == Measurement::Tc1 ||
           self.measurement == Measurement::Tc2 || 
           self.measurement == Measurement::DiffSensors {
            // if iteration % 4 == 1 {
            //     // ambient temp reading
            //     // gain of 4
            //     reading = ((value as i32) as f64) * (2.5 / (2u64.pow(15) as f64)) / 4000.0; // 2.5 ref
            //     println!("raw reading: {}", value2);

            //     // convert to celcius 
            //     let celcius = 0.403 * (reading) + 129.0;
            //     self.ambient_temp = celcius;
            // } else {
            //     // gain of 32 
            reading = ((value as i32) as f64) * (2.5 / (2u64.pow(15) as f64)) / 0.032; // 2.5 ref
            //println!("tc {}: {} ", iteration%3, reading);

            if self.measurement != Measurement::DiffSensors {
                reading = (typek_convert(30.5, reading as f32) + 273.15) as f64;
                println!("tc {}: {}", iteration%3, reading);
            }
        }
        reading
    }
}

pub fn open_controllers() -> Vec<Arc<Gpio>> {
    (0..=3).map(|i| Gpio::open(i)).collect()
}

pub fn gpio_controller_mappings(controllers: &Vec<Arc<Gpio>>) -> HashMap<Measurement, Pin> {
    // Return (chip select mapping, data ready mapping)
    //let mut gpio_mapping: HashMap<Measurement, Pin> = HashMap::new();

    let cl_pin = controllers[0].get_pin(30);
    cl_pin.mode(Output);

    let i_valve_pin = controllers[2].get_pin(4);
    i_valve_pin.mode(Output);

    let v_valve_pin = controllers[0].get_pin(26);
    v_valve_pin.mode(Output);

    let v_power_pin = controllers[2].get_pin(13);
    v_power_pin.mode(Output);

    let i_power_pin = controllers[2].get_pin(15);
    i_power_pin.mode(Output);

    let tc_1_pin = controllers[0].get_pin(10);
    tc_1_pin.mode(Output);

    let tc_2_pin = controllers[0].get_pin(20);
    tc_2_pin.mode(Output);

    let diff_pin = controllers[3].get_pin(16);
    diff_pin.mode(Output);

    let rtd_pin = controllers[2].get_pin(11);
    rtd_pin.mode(Output);

    let gpio_mapping = HashMap::from([
        (Measurement::CurrentLoopPt, cl_pin),
        (Measurement::IValve, i_valve_pin),
        (Measurement::VValve, v_valve_pin),
        (Measurement::VPower, v_power_pin),
        (Measurement::IPower, i_power_pin),
        (Measurement::Tc1, tc_1_pin),
        (Measurement::Tc2, tc_2_pin),
        (Measurement::DiffSensors, diff_pin),
        (Measurement::Rtd, rtd_pin),
        ]);

    gpio_mapping
}

pub fn pull_gpios_high(controllers: &Vec<Arc<Gpio>>) {
    //let cs_cl = Gpio::open(0);
    let clk_cl = controllers[0].get_pin(30);
    clk_cl.mode(Output);
    clk_cl.digital_write(High);

    //let cs_valve_i = Gpio::open(2);
    let clk_valve_i = controllers[2].get_pin(4);
    clk_valve_i.mode(Output);
    clk_valve_i.digital_write(High);

    //let cs_valve_v = Gpio::open(0);
    let clk_valve_v = controllers[0].get_pin(26);
    clk_valve_v.mode(Output);
    clk_valve_v.digital_write(High);

    //let cs_v_power = Gpio::open(2);
    let clk_v_power = controllers[2].get_pin(13);
    clk_v_power.mode(Output);
    clk_v_power.digital_write(High);

    //let cs_i_power = Gpio::open(2);
    let clk_i_power = controllers[2].get_pin(15);
    clk_i_power.mode(Output);
    clk_i_power.digital_write(High);

    //let cs_tc_1 = Gpio::open(0);
    let clk_tc_1 = controllers[0].get_pin(10);
    clk_tc_1.mode(Output);
    clk_tc_1.digital_write(High);

    //let cs_tc_2 = Gpio::open(0);
    let clk_tc_2 = controllers[0].get_pin(20);
    clk_tc_2.mode(Output);
    clk_tc_2.digital_write(High);

    //let cs_ds = Gpio::open(3);
    let clk_ds = controllers[3].get_pin(16);
    clk_ds.mode(Output);
    clk_ds.digital_write(High);

    //let cs_rtd = Gpio::open(2);
    let clk_rtd = controllers[2].get_pin(11);
    clk_rtd.mode(Output);
    clk_rtd.digital_write(High);

    // others 
    //let cs_spi0 = Gpio::open(0);
    let clk_spi0 = controllers[0].get_pin(5);
    clk_spi0.mode(Output);
    clk_spi0.digital_write(High);

    //let cs_spi1 = Gpio::open(0);
    let clk_spi1 = controllers[0].get_pin(13);
    clk_spi1.mode(Output);
    clk_spi1.digital_write(High);

    //let cs_brd_temp = Gpio::open(0);
    let clk_brd_temp = controllers[0].get_pin(23);
    clk_brd_temp.mode(Output);
    clk_brd_temp.digital_write(High);

    //let cs_tc_cj = Gpio::open(2);
    let clk_tc_cj = controllers[2].get_pin(23);
    clk_tc_cj.mode(Output);
    clk_tc_cj.digital_write(High);
}