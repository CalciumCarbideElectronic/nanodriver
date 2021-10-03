pub mod builder;
pub mod driver;
pub mod reg;
mod utils;

type AD5370PerChannelRegister = [u16; 40];
#[derive(Clone, Copy)]
struct ReadResp([u8; 3]);

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
