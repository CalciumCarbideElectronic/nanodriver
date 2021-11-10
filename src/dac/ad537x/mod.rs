use std::{
    fmt::Display,
    marker::{Send, Sync},
};

use self::driver::AD5370;

pub mod builder;
pub mod driver;
pub mod labview;
pub mod reg;
mod utils;

pub type Instance<'a> = AD5370<'a>;
type AD5370PerChannelRegister = [u16; 40];
#[derive(Clone, Copy, Debug)]
struct ReadResp([u8; 3]);

impl Display for ReadResp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:02X}{:02X}{:02X}", self.0[0], self.0[1], self.0[2])
    }
}

#[allow(dead_code)]
impl ReadResp {
    fn new() -> Self {
        ReadResp([0; 3])
    }
    fn as_mut(&mut self) -> &mut [u8] {
        self.0.as_mut()
    }
    fn to_u32(self) -> u32 {
        u32::from_be_bytes([0, self.0[0], self.0[1], self.0[2]])
    }
    fn to_u16(self) -> u16 {
        u16::from_be_bytes([self.0[1], self.0[2]])
    }
    fn to_u8(self) -> u8 {
        self.0[2]
    }
}

// impl From<AD5370> for Instance {
//     fn from(d: AD5370) -> Self {
//         return Mutex::new(d);
//     }
// }
pub enum Action {}
unsafe impl Send for Action {}
unsafe impl Sync for Action {}
