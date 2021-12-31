#![allow(dead_code)]

use super::{
    builder::*,
    reg::{ReadBackAddr, Register},
    ReadResp,
};

use super::reg::{ChannelAddress, WriteMode};
use crate::{
    error::IError,
    interface::ftdi::{gpio::IOController, spi::Transactional},
};

pub struct AD5370<'a> {
    pub vref: f64,
    pub reg: Register,
    pub _spi: Box<dyn embedded_hal::spi>,
    pub spi: Box<dyn Transactional + 'a>,
    ///BUSY Input/Output (Active Low). BUSY is open-drain when an output.
    ///See the BUSY and LDAC Functions section for more information
    pub _busy: Box<dyn OutputPin>,
    //Load DAC Logic Input (Active Low).
    pub _ldac: Box<dyn OutputPin>,
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
        //TODO: use button on the EVAL board for now. please keep LK3 connected.
        // self._reset.reset();
        // sleep(Duration::from_millis(1));
        // self._reset.set()?;

        self._clr.set()?;
        self._ldac.set()?;
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
        let m = self.reg.gain[idx];

        let first_item = (vol - vs) * (k1 as f64) / (4.0 * self.vref);
        let suffix = (4 * ofs + k2 - c) as f64;
        let coef = (k1 as f64 / (m + 1) as f64) as f64;

        ((first_item + suffix) * coef).round() as u16
    }

    pub fn set_code(&mut self, code: u16, target: ChannelAddress) -> Result<(), IError> {
        let data = MainBuilder::default()
            .write(WriteMode::Data)
            .address(target)
            .data(code)
            .build();

        //11_00 0000_
        // let data =[
        //     0b1100_0000,
        //     (code>>8) as u8,
        //     code as u8
        // ];

        self.spi.spi_write(&data)?;
        Ok(())
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

        println!("set voltage: write data {:?}", data);
        self.spi.spi_write(&data)?;
        Ok(())
    }

    pub fn set_gain(&mut self, value: u16) -> Result<(), IError> {
        let data = MainBuilder::default()
            .write(WriteMode::Gain)
            .address(ChannelAddress::AllCh)
            .data(value)
            .build();
        self.spi.spi_write(&data)
    }

    pub fn set_offset(&mut self, value: u16) -> Result<(), IError> {
        let data = MainBuilder::default()
            .write(WriteMode::Offset)
            .address(ChannelAddress::AllCh)
            .data(value)
            .build();
        self.spi.spi_write(&data)
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
                let name = ["X1A", "X1B", "C", "M"];
                for item in 0..4 {
                    self.spi.spi_read(&prefix[item], data[item].as_mut())?;
                    println!(
                        "read reg (group:{},ch:{}){} , value:{:}",
                        group, ch, name[item], &data[item]
                    );
                }
                self.reg.x1_a[ch as usize] = data[0].to_u16();
                self.reg.x1_b[ch as usize] = data[1].to_u16();
                self.reg.offset[ch as usize] = data[2].to_u16();
                self.reg.gain[ch as usize] = data[3].to_u16();
            }
        }

        let mut data = ReadResp::new();

        let mut prefix: [u8; 3] = builder.read(ReadBackAddr::OFS0).build();
        self.spi.spi_read(&prefix, data.as_mut())?;
        self.reg.ofs0 = data.to_u16();
        println!("read reg ofs0, value:{:}", &data);

        prefix = (*builder.read(ReadBackAddr::OFS1)).build();
        self.spi.spi_read(&prefix, data.as_mut())?;
        self.reg.ofs1 = data.to_u16();
        println!("read reg ofs1, value:{:}", &data);

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
