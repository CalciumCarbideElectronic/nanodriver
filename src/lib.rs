extern crate ftdi_mpsse;
mod dac;
mod error;
mod global;
mod interface;
mod svc;

use global::GLOBAL_AD5370;

#[no_mangle]
pub extern "C" fn rust_increment(value: u32) -> u32 {
    value + 1
}

#[no_mangle]
pub extern "C" fn set_voltage(value: f64) -> bool {
    let mut guard = GLOBAL_AD5370.lock().unwrap();
    guard
        .set_voltage(value, dac::ad537x::reg::ChannelAddress::AllCh)
        .is_ok()
}
