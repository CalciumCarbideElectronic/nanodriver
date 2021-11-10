extern crate ftdi_mpsse;
mod dac;
mod error;
mod global;
mod interface;
mod svc;

use ftdi_embedded_hal::OutputPin as FtPin;
use global::{FTDI, GLOBAL_AD5370};
use libftd2xx::Ft4232h;
use once_cell::sync::Lazy;

pub static mut PIN: Lazy<FtPin<'static, Ft4232h>> = Lazy::new(|| FTDI.ad3());

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn set_code_to_all(code: u16) -> u32 {
    let mut guard = GLOBAL_AD5370.lock().unwrap();

    guard
        .set_code(code, dac::ad537x::reg::ChannelAddress::AllCh)
        .unwrap();
    0
}
