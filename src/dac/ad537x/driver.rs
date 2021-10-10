#![allow(dead_code)]
use super::{
    builder::*,
    reg::{ReadBackAddr, Register},
    ReadResp,
};

use crate::{
    error::IError,
    interface::{gpio::IOController, spi::Transactional},
};

use super::reg::{ChannelAddress, WriteMode};

pub struct AD5370<'a> {
    pub vref: f64,
    pub reg: Register,
    pub spi: Box<dyn Transactional + 'a>,
    ///BUSY Input/Output (Active Low). BUSY is open-drain when an output.
    ///See the BUSY and LDAC Functions section for more information
    pub _busy: Box<dyn IOController + 'a>,
    //Load DAC Logic Input (Active Low).
    pub _ldac: Box<dyn IOController + 'a>,
    //Digital Reset Input
    pub _reset: Box<dyn IOController + 'a>,
    ///Asynchronous Clear Input (Level Sensitive, Active Low).
    ///See the Clear Function section for more information
    pub _clr: Box<dyn IOController + 'a>,
}

impl<'a> AD5370<'a> {
    pub fn get_reg(&self) -> Register {
        self.reg
    }
    // On the rising
    // edge of RESET, the AD5370 state machine initiates a reset
    // sequence to reset the X, M, and C registers to their default
    // values.
    pub fn reset(&mut self) -> Result<(), IError> {
        self._reset.reset()?;
        self._reset.set()?;
        Ok(())
    }
    pub fn clear(&mut self) -> Result<(), IError> {
        self._clr.reset()?;
        Ok(())
    }

    pub fn restore_clear(&mut self) -> Result<(), IError> {
        self._clr.set()?;
        Ok(())
    }

    pub fn init(&mut self) -> Result<(), IError> {
        self._reset.set()?;
        self._clr.set()?;
        self._ldac.reset()?;
        Ok(())
    }
    pub fn write_raw(&mut self, data: [u8; 3]) -> Result<(), IError> {
        self.spi.spi_write(&data)?;
        Ok(())
    }

    fn voltage_to_input(&self, vol: f64, group: u8, ch: u8) -> u16 {
        let vs = 0.0;
        let k1 = 1_u32 << 16_u32;
        let k2 = 1_u16 << 15_u16;
        let ofs: u16 = match group {
            0 => self.reg.ofs0,
            _ => self.reg.ofs1,
        };
        let idx = (group * 8 + ch) as usize;
        let c = self.reg.offset[idx];
        let m = self.reg.offset[idx];

        let first_item = (vol - vs) * (k1 as f64) / (4.0 * self.vref);
        let suffix = (4 * ofs + k2 - c) as f64;
        let coef = (k1 / (m + 1) as u32) as f64;

        let x = (first_item + suffix) * coef;

        x as u16
    }

    pub fn set_voltage(&mut self, vol: f64, target: ChannelAddress) -> Result<(), IError> {
        let (g, c) = match target {
            ChannelAddress::AllCh => (0, 0),
            ChannelAddress::SingleCh { ch, group } => (group, ch),
            ChannelAddress::SingleGroup { group } => (group, 0),
            ChannelAddress::Chx { ch } => (0, ch),
            ChannelAddress::ChxExceptGroup0 { ch } => (1, ch),
        };
        let data = MainBuilder::default()
            .write(WriteMode::Data)
            .address(target)
            .data(self.voltage_to_input(vol, g, c))
            .build();
        self.spi.spi_write(&data)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn read_all(&mut self) -> Result<(), IError> {
        let mut builder = MainBuilder::default();
        for group in 0..5 {
            for ch in 0..8 {
                let prefix: [[u8; 3]; 4] = [
                    builder.read(ReadBackAddr::X1A { group, ch }).build(),
                    builder.read(ReadBackAddr::X1B { group, ch }).build(),
                    builder.read(ReadBackAddr::C { group, ch }).build(),
                    builder.read(ReadBackAddr::M { group, ch }).build(),
                ];
                let mut data: [ReadResp; 4] = [ReadResp::new(); 4];
                for item in 0..4 {
                    self.spi.spi_read(&prefix[item], data[item].as_mut())?;
                }
                self.reg.x1_a[ch as usize] = data[0].to_u16();
                self.reg.x1_b[ch as usize] = data[1].to_u16();
                self.reg.offset[ch as usize] = data[2].to_u16();
                self.reg.gain[ch as usize] = data[3].to_u16();
            }
        }

        let mut prefix: [u8; 3] = builder.read(ReadBackAddr::OFS0).build();
        let mut data = ReadResp::new();
        self.spi.spi_read(&prefix, data.as_mut())?;
        self.reg.ofs0 = data.to_u16();

        prefix = (*builder.read(ReadBackAddr::OFS1)).build();
        self.spi.spi_read(&prefix, data.as_mut())?;

        self.spi.spi_read(&prefix, data.as_mut())?;
        Ok(())
    }
}

#[allow(clippy::unusual_byte_groupings)]
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_builder() {
        let data: [u8; 3] = MainBuilder::default()
            .write(WriteMode::Data)
            .address(ChannelAddress::Chx { ch: 5 })
            .data(0xA5A5)
            .build();

        assert_eq!([0b11_110101, 0xA5, 0xA5], data);

        let data: [u8; 3] = MainBuilder::default()
            .write(WriteMode::Offset)
            .address(ChannelAddress::SingleCh { group: 3, ch: 7 })
            .data(0xA5A5)
            .build();

        assert_eq!([0b10_100111, 0xA5, 0xA5], data);

        let data: [u8; 3] = MainBuilder::default()
            .read(ReadBackAddr::M { group: 4, ch: 7 })
            .build();

        assert_eq!([0b00_000101, 0b011_10111, 0b1000_0000], data);

        let data: [u8; 3] = MainBuilder::default()
            .read(ReadBackAddr::Select { group: 3 })
            .build();

        assert_eq!([0b00_000101, 0b100_00100, 0b1000_0000], data);

        let data: [u8; 3] = MainBuilder::default().read(ReadBackAddr::OFS0).build();
        assert_eq!([0b00_000101, 0b100_00001, 0b0000_0000], data);

        let data: [u8; 3] = MainBuilder::default()
            .read(ReadBackAddr::X1A { group: 2, ch: 2 })
            .build();
        assert_eq!([0b00_000101, 0b000_01101, 0b0000_0000], data);
    }
}
