#[cfg(test)]
use embedded_hal::{digital::v2::OutputPin, prelude::_embedded_hal_spi_FullDuplex, spi::Polarity};
use ftdi_embedded_hal as hal;
use hal::{FtHal, Initialized};
use libftd2xx::{Ft4232h, MpsseSettings};
use once_cell::sync::Lazy;
use std::{sync::Mutex, thread::sleep, time::Duration};

use crate::{
    dac::ad537x::{
        driver::AD5370,
        reg::{ChannelAddress, Register},
    },
    interface::{gpio::FtdiGPIOController, spi::FtdiSPIController},
};

const SLEEP_DURATION: Duration = Duration::from_millis(2000);

use crate::global::{FTDI, GLOBAL_AD5370};

#[test]
fn test_spi_pattern() {
    let mut spi = FTDI.spi().unwrap();
    let mut cs = FTDI.ad3();
    spi.set_clock_polarity(Polarity::IdleLow);
    for _ in 0..100 {
        cs.set_low().unwrap();
        spi.send(0x42).unwrap();
        spi.send(0xA3).unwrap();
        spi.send(0x57).unwrap();
        cs.set_high().unwrap();
    }
}

#[test]
fn test_set_voltage() {
    let mut guard = GLOBAL_AD5370.lock().unwrap();
    guard.init();
    for _i in 0..10 {
        guard._ldac.set();
        guard.set_voltage(0.0051, crate::dac::ad537x::reg::ChannelAddress::AllCh);
        guard._ldac.reset();
    }
}

#[test]
fn test_set_code() {
    let mut guard = GLOBAL_AD5370.lock().unwrap();
    // guard._ldac.set().unwrap();
    guard
        .set_code(0xF4FF, crate::dac::ad537x::reg::ChannelAddress::AllCh)
        .unwrap();
    sleep(Duration::from_millis(200));

    // guard._clr.set().unwrap();
    // guard._ldac.reset().unwrap();
    // guard._ldac.set().unwrap();
}

#[test]
fn test_ldac() {
    let mut guard = GLOBAL_AD5370.lock().unwrap();
    guard._ldac.reset();
    guard._ldac.set();
}

#[test]
fn test_read_all() {
    println!("hi");
    let mut guard = GLOBAL_AD5370.lock().unwrap();
    guard.init().unwrap();
    guard.set_voltage(0.1, ChannelAddress::AllCh).unwrap();
    guard.set_offset(0x1000).unwrap();
    guard.read_all().unwrap();
}
