use spidev::spidevioctl::SpidevTransfer;
use spidev::{SpiModeFlags, Spidev, SpidevOptions};
use std::{thread, time};
use std::cell::RefCell;
use std::rc::Rc;

pub mod gpio;
pub mod adc;

/* A large portion of the code here is lifted from example code from
    https://github.com/rust-embedded/rust-spidev/blob/master/examples/spidev-bidir.rs
    While libbeaglebone does exist, that project has been abandoned while in alpha
    status, so not using it here.
*/
fn main() {
    /* Create a spidev wrapper to work with
    you call this wrapper to handle and all transfers */
    let mut spidev = Spidev::open("/dev/spidev0.0").unwrap();
    /* ADS124S06 only supports SPI mode 1, so we're sticking with that here
      For a primer on SPI modes of operation, look here:
      https://stackoverflow.com/questions/43155025/why-different-modes-are-provided-in-spi-communication
    */
    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(100000)
        .lsb_first(false)
        .mode(SpiModeFlags::SPI_MODE_1)
        .build();
    spidev.configure(&options).unwrap();

    let ref_spidev: Rc<RefCell<_>> = Rc::new(RefCell::new(spidev));
    let mut adc_cl = adc::ADC::new(adc::Measurement::CurrentLoopPt, ref_spidev.clone());
    //let mut adc_diff = adc::ADC::new(adc::Measurement::DiffSensors, ref_spidev.clone());
    read_adc_test(&mut adc_cl);
}

fn read_adc_test(adc: &mut adc::ADC) {
    adc.init_gpio();
    println!("Resetting ADC");
    adc.reset_status();

    // delay for at least 4000*clock period
    println!("Delaying for 1 second");
    thread::sleep(time::Duration::from_millis(1000));

    test_measurement_init(adc);
    adc.start_conversion();
    loop {
        test_read_all(adc);
        thread::sleep(time::Duration::from_millis(500));
    }
}

fn test_measurement_init(adc: &mut adc::ADC) {
    // Read initial registers
    println!("Reading initial register states");
    adc.read_regs(0, 17);

    // delay for at least 4000*clock period
    println!("Delaying for 1 second");
    thread::sleep(time::Duration::from_millis(1000));

    // Write to registers
    println!("Writing to registers");
    match adc.measurement {
        adc::Measurement::CurrentLoopPt | 
        adc::Measurement::ValveCurrentFb |
        adc::Measurement::ValveVoltageFb | 
        adc::Measurement::Power => {
            adc.write_reg(0x03, 0x00);
            adc.write_reg(0x04, 0x0E);
            adc.write_reg(0x08, 0x40);
            adc.write_reg(0x05, 0x0A)
        }

        adc::Measurement::Rtd => {
            adc.write_reg(0x03, 0x00);
            adc.write_reg(0x04, 0x0E);
            adc.write_reg(0x06, 0x47);
            adc.write_reg(0x07, 0x50);
        }

        adc::Measurement::Tc1 | 
        adc::Measurement::Tc2 | 
        adc::Measurement::DiffSensors => {
            adc.write_reg(0x03, 0x0D);
            adc.write_reg(0x04, 0x0E);
            adc.write_reg(0x05, 0x0A);
        }
    }

    // delay for at least 4000*clock period
    println!("Delaying for 1 second");
    thread::sleep(time::Duration::from_millis(1000));

    // Read registers
    println!("Reading new register states");
    adc.read_regs(0, 17);
    
}

fn test_read_all(adc: &mut adc::ADC) {
    match adc.measurement {
        adc::Measurement::CurrentLoopPt | 
        adc::Measurement::ValveCurrentFb |
        adc::Measurement::ValveVoltageFb | 
        adc::Measurement::Power => {
            adc.write_reg(0x02, 0x50 | 0x0C);
            adc.test_read_individual();
            adc.write_reg(0x02, 0x40 | 0x0C);
            adc.test_read_individual();
            adc.write_reg(0x02, 0x30 | 0x0C);
            adc.test_read_individual();
            adc.write_reg(0x02, 0x20 | 0x0C);
            adc.test_read_individual();
            adc.write_reg(0x02, 0x10 | 0x0C);
            adc.test_read_individual();
            adc.write_reg(0x02, 0x00 | 0x0C);
            adc.test_read_individual();
            println!();
        }

        adc::Measurement::Rtd => {
            adc.write_reg(0x02, 0x34); // INPMUX
            adc.write_reg(0x05, 0x00); // Ref Control
            adc.test_read_individual();     

            adc.write_reg(0x02, 0x12); // INPMUX
            adc.write_reg(0x05, 0x05); // Ref Control
            adc.test_read_individual();
            println!();

        }

        adc::Measurement::DiffSensors => {
            // set INPMUX
            adc.write_reg(0x02, 0x54);
            adc.test_read_individual();
            adc.write_reg(0x02, 0x32);
            adc.test_read_individual();
            adc.write_reg(0x02, 0x10);
            adc.test_read_individual();
            println!();
        }

        adc::Measurement::Tc1 |
        adc::Measurement::Tc2 => {
            adc.write_reg(0x02, 0x10 | 0x00); // INPMUX
            adc.write_reg(0x08, 0x02); // VBIAS
            adc.test_read_individual();
        
            adc.write_reg(0x02, 0x30 | 0x02); // INPMUX
            adc.write_reg(0x08, 0x08); // VBIAS
            adc.test_read_individual();

            adc.write_reg(0x02, 0x50 | 0x04); // INPMUX
            adc.write_reg(0x08, 0x20); // VBIAS
            adc.test_read_individual();
            println!();
        }
    }
}


// use command::command_loop;
// use data::data_loop;
// use discovery::discovery_loop;
// use std::{thread, time::Duration};

// pub mod command;
// pub mod data;
// pub mod discovery;

// fn main() {
//     let data_loop = thread::spawn(|| data_loop::begin(500));
//     let command_loop = thread::spawn(|| command_loop::begin());
//     let discovery_loop = thread::spawn(|| discovery_loop::begin(Duration::from_secs(5)));

//     command_loop.join().unwrap();
//     data_loop.join().unwrap();
//     discovery_loop.join().unwrap();
//     panic!("Control loop terminated!");
// }




