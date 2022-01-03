#![allow(dead_code)]
use crate::dac::ad537x::driver::AD5370Instance;
use crate::{
    dac::ad537x::{driver::AD5370, reg::Register},
    log::log,
    sin::{Action, SinExeciter},
};
use ftdi_embedded_hal as hal;
use hal::{FtHal, Initialized};
use libftd2xx::{Ft4232h, MpsseSettings};
use once_cell::sync::Lazy;
use rppal::{
    gpio::Gpio,
    spi::{Bus, Spi},
};
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

// pub static GLOBAL_AD5370: Lazy<Mutex<AD5370FDTD>> = Lazy::new(|| {
//     let mut _spi = FTDI.spi().unwrap();
//     _spi.set_clock_polarity(Polarity::IdleLow);

//     let mut t = AD5370 {
//         vref: 4.0,
//         reg: Register::default(),
//         _spi: Box::new(_spi),
//         _busy: Box::new(FTDI.ad4()),
//         _ldac: Box::new(FTDI.ad5()),
//         _reset: Box::new(FTDI.ad6()),
//         _clr: Box::new(FTDI.ad7()),
//     };
//     t.init().unwrap();
//     Mutex::new(t)
// });

#[cfg(feature = "raspberry")]
pub static GLOBAL_AD5370: Lazy<Mutex<Box<dyn AD5370Instance>>> = Lazy::new(|| {
    let _spi = Spi::new(
        Bus::Spi0,
        rppal::spi::SlaveSelect::Ss0,
        1000,
        rppal::spi::Mode::Mode0,
    )
    .unwrap();
    let _io = Gpio::new().unwrap();

    let mut t = Box::new(AD5370 {
        vref: 4.0,
        reg: Register::default(),
        _spi: Box::new(_spi),
        _busy: Box::new(_io.get(27).unwrap().into_input_pulldown()),
        _ldac: Box::new(_io.get(27).unwrap().into_output()),
        _reset: Box::new(_io.get(27).unwrap().into_output()),
        _clr: Box::new(_io.get(27).unwrap().into_output()),
    });
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
