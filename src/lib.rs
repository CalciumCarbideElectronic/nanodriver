extern crate ftdi_mpsse;
mod dac;
mod error;
mod global;
mod interface;
mod sin;
mod svc;

use std::{
    sync::mpsc::{self, Sender},
    thread::{self, JoinHandle},
};

use ftdi_embedded_hal::OutputPin as FtPin;
use global::{FTDI, GLOBAL_AD5370};
use libftd2xx::Ft4232h;
use once_cell::sync::Lazy;
use sin::{Action, SinExeciter};

pub static mut PIN: Lazy<FtPin<'static, Ft4232h>> = Lazy::new(|| FTDI.ad3());

pub static mut HANDLE: Lazy<Option<JoinHandle<()>>> = Lazy::new(|| None);
pub static mut TERMINATE_SENDER: Lazy<Option<Sender<Action>>> = Lazy::new(|| None);

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn set_code_to_all(code: u16) -> u32 {
    let mut guard = GLOBAL_AD5370.lock().unwrap();

    guard
        .set_code(code, dac::ad537x::reg::ChannelAddress::AllCh)
        .unwrap();
    0
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn start(_code: u16) -> u32 {
    let (tx, rx) = mpsc::channel();
    let _sin_exec = SinExeciter::new(rx);

    let _h = thread::spawn(move || loop {
        _sin_exec.run()
    });
    HANDLE.replace(_h);
    TERMINATE_SENDER.replace(tx);
    0
}
#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn stop(_code: u16) -> u32 {
    if let Some(h) = TERMINATE_SENDER.as_mut() {
        h.send(Action::Stop).unwrap();
        TERMINATE_SENDER.take();
    }
    let handle = HANDLE.take();

    if let Some(h) = handle {
        h.join().unwrap_or(());
    }
    0
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn set_voltage(channel: u8, code: u16) -> u32 {
    if let Some(h) = TERMINATE_SENDER.as_mut() {
        h.send(Action::SetAmp { channel, code }).unwrap();
        TERMINATE_SENDER.take();
    }
    0
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn set_freq(channel: u8, freq: u64) -> u32 {
    if let Some(h) = TERMINATE_SENDER.as_mut() {
        h.send(Action::SetFreq { channel, freq }).unwrap();
        TERMINATE_SENDER.take();
    }
    1
}
