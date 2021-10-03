use std::sync::Arc;

use dac::ad537x::driver::AD5370;

use interface::gpio::Pin;
use interface::spi::FtdiSPIController;

// #[macro_use]
extern crate ftdi_mpsse;
mod dac;
mod error;
mod interface;
use ftdi_embedded_hal as hal;

use crate::{dac::ad537x::reg::Register, interface::gpio::FtdiGPIOController};

/// Transaction enum defines possible SPI transactions

fn main() {
    let ftdi = hal::Ft4232hHal::new()
        .expect("Failed to open FT232H device")
        .init_default()
        .expect("Failed to initialize MPSSE");

    let h = Arc::new(ftdi);
    let spi = FtdiSPIController { _ft: h.clone() };

    let _ad5370 = AD5370 {
        vref: 4.0,
        reg: Register::default(),
        spi: Box::new(spi),
        _busy: Box::new(FtdiGPIOController::new(h.clone(), Pin::AD0)),
        _ldac: Box::new(FtdiGPIOController::new(h.clone(), Pin::AD1)),
        _reset: Box::new(FtdiGPIOController::new(h.clone(), Pin::AD2)),
        _clr: Box::new(FtdiGPIOController::new(h, Pin::AD3)),
    };
}
