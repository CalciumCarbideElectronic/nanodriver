use crate::{
    dac::ad537x::{driver::AD5370, reg::Register},
    interface::gpio::FtdiGPIOController,
    interface::spi::FtdiSPIController,
};
use ftdi_embedded_hal as hal;
use libftd2xx::Ft4232h;
use once_cell::sync::Lazy;
use std::sync::{Mutex};

use hal::{FtHal, Initialized};

pub static FTDI: Lazy<FtHal<Ft4232h, Initialized>> = Lazy::new(|| {
    let ftdi: FtHal<Ft4232h, Initialized> = hal::Ft4232hHal::new()
        .expect("Failed to open FT232H device")
        .init_default()
        .expect("Failed to initialize MPSSE");
    ftdi
});
pub static GLOBAL_AD5370: Lazy<Mutex<AD5370>> = Lazy::new(|| {
    let spi = Box::new(FtdiSPIController { _ft: &FTDI });
    let mut _busy = FtdiGPIOController::new_boxed(FTDI.ad4());
    let mut _ldac = FtdiGPIOController::new_boxed(FTDI.ad5());
    let mut _reset = FtdiGPIOController::new_boxed(FTDI.ad6());
    let mut _clr = FtdiGPIOController::new_boxed(FTDI.ad7());
    Mutex::new(AD5370 {
        vref: 4.0,
        reg: Register::default(),
        spi,
        _busy,
        _ldac,
        _reset,
        _clr,
    })
});
