use spidev::spidevioctl::SpidevTransfer;
use spidev::Spidev;
use std::{thread, time};
use super::gpio;
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Measurement {
    CurrentLoopPt,
    ValveCurrentFb,
    ValveVoltageFb,
    Power,
    Tc1,
    Tc2,
    DiffSensors,
    Rtd
}

pub struct ADC {
    pub measurement: Measurement,
    pub spidev: Rc<RefCell<Spidev>>,
}

impl ADC {
    // Constructs a new instance of an Analog-to-Digital Converter 
    pub fn new(measurement: Measurement, spidev: Rc<RefCell<Spidev>>) -> ADC {
        ADC {
            measurement: measurement,
            spidev: spidev,
        }
    }

    pub fn init_gpio(&mut self) {
        // let mut all_gpios = vec![5, 7, 10, 12, 13, 20, 23, 26, 30, 33, 36, 44, 67, 68, 86, 87, 112];
        let mut all_gpios = vec![5, 7, 10, 12, 13, 20, 23, 26, 30, 33, 36, 44, 67, 68, 86, 87, 112];
        let mut cs_gpios: HashMap<Measurement, usize> = HashMap::new();
        cs_gpios.insert(Measurement::CurrentLoopPt, 30);
        cs_gpios.insert(Measurement::ValveCurrentFb, 68);
        cs_gpios.insert(Measurement::ValveVoltageFb, 26);
        cs_gpios.insert(Measurement::Power, 86);
        cs_gpios.insert(Measurement::Tc1, 10);
        cs_gpios.insert(Measurement::Tc2, 20);
        cs_gpios.insert(Measurement::DiffSensors, 112);
        cs_gpios.insert(Measurement::Rtd, 5);
        
        // pull BeagleBone chip select pin for this ADC low 
        if let Some(my_cs) = cs_gpios.get(&self.measurement) {
            let gpio_str: &str = &format!("{}", my_cs);
            println!("setting low: gpio_str");
            gpio::set_output(gpio_str);
            gpio::set_low(gpio_str);
            all_gpios.retain(|&x| x != *my_cs);
        }
    
        // pull all other BeagleBone gpios high
        for i in &all_gpios {
            let gpio_str: &str = &format!("{}", i);
            println!("setting high: gpio_str");
            gpio::set_output(gpio_str);
            gpio::set_high(gpio_str);
        }
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
        thread::sleep(time::Duration::from_millis(1000));
        //println!("reg: {}, data: {:?}", reg, rx_buf_rdata);
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

    pub fn test_read_individual(&mut self) {
        thread::sleep(time::Duration::from_millis(10));
        let mut tx_buf_rdata = [ 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ];
        let mut rx_buf_rdata = [ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ];
        let mut transfer = SpidevTransfer::read_write(&mut tx_buf_rdata, &mut rx_buf_rdata);
        let _status = self.spidev.borrow_mut().transfer(&mut transfer);
        let value: u16 = ((rx_buf_rdata[1] as u16) << 8) | (rx_buf_rdata[2] as u16);
        let value2: i16 = value as i16;
        let reading = ((value2 as i32 + 32768) as f64) * (2.5 / (2u64.pow(15) as f64));
        print!("{} ", reading);
    }
    
}
