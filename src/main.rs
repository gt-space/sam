use spidev::spidevioctl::SpidevTransfer;
use spidev::{SpiModeFlags, Spidev, SpidevOptions};
use std::{thread, time};

pub mod gpio;

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

    gpio::set_output("10");
    gpio::set_high("10");

    //gpio::set_gpio("12");
    gpio::set_output("12");
    gpio::set_high("12");

    // gpio::set_gpio("13");
    gpio::set_output("13");
    gpio::set_high("13");

    // gpio::set_gpio("20");
    gpio::set_output("20");
    gpio::set_high("20");

    gpio::set_output("23");
    gpio::set_high("23");

    // RTD
    gpio::set_output("30");
    gpio::set_high("30");

    // gpio::set_gpio("33");
    gpio::set_output("33");
    gpio::set_high("33");

    // gpio::set_gpio("36");
    gpio::set_output("36");
    gpio::set_high("36");

    gpio::set_output("44");
    gpio::set_high("44");

    gpio::set_output("67");
    gpio::set_high("67");

    gpio::set_output("86");
    gpio::set_high("86");

    gpio::set_output("87");
    gpio::set_high("87");

    gpio::set_output("112");
    gpio::set_high("112");

    // gpio::set_gpio("7");
    gpio::set_output("7");
    gpio::set_high("7");

    // gpio::set_gpio("5");
    gpio::set_output("5");
    gpio::set_low("5");

    println!("Resetting ADC");
    reset_status(&mut spidev);

    // delay for at least 4000*clock period
    println!("Delaying for 1 second");
    thread::sleep(time::Duration::from_millis(1000));

    //test_read_regs_in_loop(&mut spidev);
    //test_current_loop_init(&mut spidev);
    //start_conversion(&mut spidev);
    // loop {
    //     test_read_individual(&mut spidev, 0x00);
    // }

    loop {
        // test_read_all(&mut spidev);
        // thread::sleep(time::Duration::from_millis(500));
        test_current_loop_init(&mut spidev);
        thread::sleep(time::Duration::from_millis(500));
    }

}

fn reset_status(spidev: &mut Spidev) {
    let tx_buf_reset = [0x06];
    let mut transfer = SpidevTransfer::write(&tx_buf_reset);
    let _status = spidev.transfer(&mut transfer);
}

fn read_regs(spidev: &mut Spidev, reg: u8, num_regs: u8) {
    let mut tx_buf_readreg = [ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ];
    let mut rx_buf_readreg = [ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ];
    tx_buf_readreg[0] = 0x20 | reg;
    tx_buf_readreg[1] = num_regs;
    let mut transfer = SpidevTransfer::read_write(&mut tx_buf_readreg, &mut rx_buf_readreg);
    let _status = spidev.transfer(&mut transfer);
    println!("data: {:?}", rx_buf_readreg);
}

fn write_reg(spidev: &mut Spidev, reg: u8, data1: u8, data2: u8) {
    let mut tx_buf_writereg = [ 0x40, 0x00, 0x00, 0x00 ];
    let mut rx_buf_writereg = [ 0x40, 0x00, 0x00, 0x00 ];
    tx_buf_writereg[0] = 0x40 | reg;
    tx_buf_writereg[2] = data1;
    tx_buf_writereg[3] = data2;
    let mut transfer = SpidevTransfer::read_write(&mut tx_buf_writereg, &mut rx_buf_writereg);
    let _status = spidev.transfer(&mut transfer);
}

fn test_read_regs_in_loop(spidev: &mut Spidev) {
    //println!("Reading initial register states");
    read_regs(spidev, 1, 17);
}

fn test_current_loop_init(spidev: &mut Spidev) {
    // Read initial registers
    println!("Reading initial register states");
    read_regs(spidev, 1, 17);

    // delay for at least 4000*clock period
    println!("Delaying for 1 second");
    thread::sleep(time::Duration::from_millis(1000));

    // Write to registers
    println!("Writing to registers");
    write_reg(spidev, 0x03, 0x00, 0x00);
    write_reg(spidev, 0x04, 0x0E, 0x00);
    write_reg(spidev, 0x08, 0x40, 0x00);

    // delay for at least 4000*clock period
    println!("Delaying for 1 second");
    thread::sleep(time::Duration::from_millis(1000));

    // Read registers
    println!("Reading new register states");
    read_regs(spidev, 1, 17);
}

fn start_conversion(spidev: &mut Spidev) {
    let mut tx_buf_rdata = [ 0x08];
    let mut rx_buf_rdata = [ 0x00];
    let mut transfer = SpidevTransfer::read_write(&mut tx_buf_rdata, &mut rx_buf_rdata);
    let _status = spidev.transfer(&mut transfer);
    thread::sleep(time::Duration::from_millis(1000));
    //println!("reg: {}, data: {:?}", reg, rx_buf_rdata);
}

fn test_read_individual(spidev: &mut Spidev, reg: u8) {
    write_reg(spidev, 0x02, reg, 0x0C);
    thread::sleep(time::Duration::from_millis(10));
    let mut tx_buf_rdata = [ 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ];
    let mut rx_buf_rdata = [ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ];
    let mut transfer = SpidevTransfer::read_write(&mut tx_buf_rdata, &mut rx_buf_rdata);
    let _status = spidev.transfer(&mut transfer);
    let value: u16 = ((rx_buf_rdata[1] as u16) << 8) | (rx_buf_rdata[2] as u16);
    let value2: i16 = value as i16;
    print!("{} ", value2);
}

fn test_read_all(spidev: &mut Spidev) {
    // current loop PT
    test_read_individual(spidev, 0x05);
    test_read_individual(spidev, 0x04);
    test_read_individual(spidev, 0x03);
    test_read_individual(spidev, 0x02);
    test_read_individual(spidev, 0x01);
    test_read_individual(spidev, 0x00);
    println!();
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
