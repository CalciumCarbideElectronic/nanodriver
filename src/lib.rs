extern crate chrono;
extern crate ftdi_mpsse;
mod dac;
mod error;
mod global;
mod interface;
mod log;
mod sin;
mod svc;

use ftdi_embedded_hal::OutputPin as FtPin;
use global::{FTDI, GLOBAL_AD5370, HANDLE, TERMINATE_SENDER};
use libftd2xx::Ft4232h;
use once_cell::sync::Lazy;
use sin::Action;

pub static mut PIN: Lazy<FtPin<'static, Ft4232h>> = Lazy::new(|| FTDI.ad3());

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn add(code: u16) -> u16 {
    code * 2
}

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
pub unsafe extern "C" fn start() -> u32 {
    0
}
#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn stop() -> u32 {
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
pub unsafe extern "C" fn set_data(channel: u8, freq: f64, code: u16) -> u32 {
    HANDLE.as_mut();
    if let Some(h) = TERMINATE_SENDER.as_mut() {
        h.try_send(Action::SetData {
            channel,
            freq,
            code,
        })
        .unwrap_or_default();
    }
    1
}
