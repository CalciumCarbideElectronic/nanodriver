use std::borrow::BorrowMut;

use super::reg::{ChannelAddress, ReadBackAddr, SpecialFunctionAddress, WriteMode};

#[derive(Copy, Clone, Default)]
pub struct MainBuilder(u32);
#[derive(Copy, Clone)]
pub struct WriteBuilder(u32);
#[derive(Copy, Clone)]
pub struct FunctionBuilder(u32);
pub trait Builder {
    fn raw(&mut self) -> &mut u32;
    fn data(&mut self, data: u16) -> &mut Self {
        *self.raw() |= data as u32;
        self
    }
    fn build(&mut self) -> [u8; 3] {
        let d = *(self.raw());
        [(d >> 16) as u8, (d >> 8) as u8, d as u8]
    }
}

impl WriteBuilder {
    pub fn address(&mut self, ch: ChannelAddress) -> &mut Self {
        let code: u8 = ch.into();
        self.0 |= ((code as u32) << 16) as u32;
        self
    }
}
impl FunctionBuilder {
    pub fn address(&mut self, ch: SpecialFunctionAddress) -> &mut Self {
        let code: u8 = ch.into();
        self.0 |= ((code as u32) << 16) as u32;
        self
    }
}

impl Builder for MainBuilder {
    fn raw(&mut self) -> &mut u32 {
        self.0.borrow_mut()
    }
}
impl Builder for WriteBuilder {
    fn raw(&mut self) -> &mut u32 {
        self.0.borrow_mut()
    }
}
impl Builder for FunctionBuilder {
    fn raw(&mut self) -> &mut u32 {
        self.0.borrow_mut()
    }
}
impl From<&mut MainBuilder> for WriteBuilder {
    fn from(c: &mut MainBuilder) -> Self {
        Self(c.0)
    }
}
impl From<&mut MainBuilder> for FunctionBuilder {
    fn from(c: &mut MainBuilder) -> Self {
        Self(c.0)
    }
}

impl MainBuilder {
    pub fn write(&mut self, mode: WriteMode) -> WriteBuilder {
        self.0 = 0;
        self.0 |= (mode as u32) << 22;
        WriteBuilder::from(self)
    }

    pub fn funtion(&mut self) -> FunctionBuilder {
        self.0 = 0;
        FunctionBuilder::from(self)
    }

    pub fn address(&mut self, ch: SpecialFunctionAddress) -> &mut Self {
        let code: u8 = ch.into();
        self.0 |= ((code as u32) << 16) as u32;
        self
    }
    pub fn read(&mut self, addr: ReadBackAddr) -> &mut Self {
        self.0 = self
            .funtion()
            .address(SpecialFunctionAddress::ReadBack)
            .data(addr.into())
            .0;
        self
    }
}

impl From<MainBuilder> for [u8; 3] {
    fn from(d: MainBuilder) -> Self {
        [(d.0 >> 16) as u8, (d.0 >> 8) as u8, d.0 as u8]
    }
}
impl From<&MainBuilder> for [u8; 3] {
    fn from(d: &MainBuilder) -> Self {
        [(d.0 >> 16) as u8, (d.0 >> 8) as u8, d.0 as u8]
    }
}
