#[macro_use]
extern crate ftdi_mpsse;
mod dac;
mod error;
mod interface;

use ftdi_mpsse::{MpsseCmdBuilder, MpsseSettings};

use std::time::Duration;

enum SPIError {}
/// Transaction enum defines possible SPI transactions

fn main() {
    println!("Hello, world!");

    let cmd = MpsseCmdBuilder::new().enable_loopback();
    let settings = MpsseSettings::default();
}
