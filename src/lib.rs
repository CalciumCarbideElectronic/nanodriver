extern crate ftdi_mpsse;
mod dac;
mod error;
mod global;
mod interface;
mod svc;


use embedded_hal::digital::v2::OutputPin;
use ftdi_embedded_hal::OutputPin as FtPin;
use global::{FTDI, GLOBAL_AD5370};
use libftd2xx::Ft4232h;
use once_cell::sync::Lazy;


pub static mut U3: Lazy<Vec<u32>> = Lazy::new(|| {
   vec![] 
});

#[no_mangle]
pub unsafe extern "C" fn rust_increment(value: u32) -> u32 {
    U3.push(value);
    U3.len() as u32
}

pub static mut PIN: Lazy<FtPin<'static,Ft4232h>> = Lazy::new(|| {
    FTDI.ad3()
});

#[no_mangle]
pub unsafe extern "C" fn set_code_to_all(code: u16) -> u32 {
    let mut guard = GLOBAL_AD5370.lock().unwrap();


    guard
        .set_code(code, dac::ad537x::reg::ChannelAddress::AllCh).unwrap();
    0
}
