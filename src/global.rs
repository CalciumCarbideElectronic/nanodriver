#![allow(dead_code)]
use crate::{
    dac::ad537x::{driver::AD5370, reg::Register},
    interface::ftdi::gpio::FtdiGPIOController,
    interface::ftdi::spi::FtdiSPIController,
    log::log,
    sin::{Action, SinExeciter},
};
use embedded_hal::{digital::v2::OutputPin, spi::Polarity};
use ftdi_embedded_hal as hal;
use hal::{FtHal, Initialized};
use libftd2xx::{Ft4232h, MpsseSettings};
use once_cell::sync::Lazy;
use std::{
    sync::mpsc::{self, SyncSender},
    thread::{self, JoinHandle},
};

use std::{sync::Mutex, time::Duration};

pub static FTDI: Lazy<FtHal<Ft4232h, Initialized>> = Lazy::new(|| {
    let settings = MpsseSettings {
        reset: true,
        in_transfer_size: 4096,
        read_timeout: Duration::from_secs(1),
        write_timeout: Duration::from_secs(1),
        latency_timer: Duration::from_millis(16),
        mask: 0,
        clock_frequency: Some(10_000_000),
    };
    let ftdi: FtHal<Ft4232h, Initialized> = hal::Ft4232hHal::new()
        .expect("Failed to open FT232H device")
        .init(&settings)
        .expect("Failed to initialize MPSSE");
    ftdi
});

pub static GLOBAL_AD5370: Lazy<Mutex<AD5370>> = Lazy::new(|| {
    let mut _spi = FTDI.spi().unwrap();
    _spi.set_clock_polarity(Polarity::IdleLow);

    let mut spi = Box::new(FtdiSPIController {
        _spi,
        _cs: FTDI.ad3(),
    });
    spi._cs.set_high().unwrap();
    let mut _busy = FtdiGPIOController::new_boxed(FTDI.ad4());
    let mut _ldac = FtdiGPIOController::new_boxed(FTDI.ad5());
    let mut _reset = FtdiGPIOController::new_boxed(FTDI.ad6());
    let mut _clr = FtdiGPIOController::new_boxed(FTDI.ad7());
    let mut t = AD5370 {
        vref: 4.0,
        reg: Register::default(),
        spi,
        _busy,
        _ldac,
        _reset,
        _clr,
    };
    t.init().unwrap();
    Mutex::new(t)
});

pub static mut TERMINATE_SENDER: Lazy<Option<SyncSender<Action>>> = Lazy::new(|| None);

pub static mut HANDLE: Lazy<Option<JoinHandle<()>>> = Lazy::new(|| unsafe {
    log(b"start");

    let (tx, rx) = mpsc::sync_channel(2);
    let mut _sin_exec = SinExeciter::new(rx);
    let _h = thread::spawn(move || loop {
        _sin_exec.run();
    });
    TERMINATE_SENDER.replace(tx);
    Some(_h)
});
