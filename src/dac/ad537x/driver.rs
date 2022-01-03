#![allow(dead_code)]

use std::convert::Infallible;

use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_hal::spi::FullDuplex;
use libftd2xx::TimeoutError;

use super::{
    builder::*,
    reg::{ReadBackAddr, Register},
    ReadResp,
};

use super::reg::{ChannelAddress, WriteMode};
use crate::error::IError;

pub type AD5370FDTD = AD5370<TimeoutError, Infallible>;
pub type AD5370RP = AD5370<rppal::spi::Error, Infallible>;

pub struct AD5370<E, R>
where
    IError: From<E>,
    IError: From<R>,
{
    pub vref: f64,
    pub reg: Register,
    pub _spi: Box<dyn FullDuplex<u8, Error = E> + Send>,
    ///BUSY Input/Output (Active Low). BUSY is open-drain when an output.
    ///See the BUSY and LDAC Functions section for more information
    pub _busy: Box<dyn InputPin<Error = R> + Send>,
    //Load DAC Logic Input (Active Low).
    pub _ldac: Box<dyn OutputPin<Error = R> + Send>,
    //Digital Reset Input
    pub _reset: Box<dyn OutputPin<Error = R> + Send>,
    ///Asynchronous Clear Input (Level Sensitive, Active Low).
    ///See the Clear Function section for more information
    pub _clr: Box<dyn OutputPin<Error = R> + Send>,
}

pub trait AD5370Instance: Send {
    fn init(&mut self) -> Result<(), IError>;
    fn reset(&mut self) -> Result<(), IError>;
    fn clear(&mut self) -> Result<(), IError>;
    fn set_ldac(&mut self) -> Result<(), IError>;

    fn clear_ldac(&mut self) -> Result<(), IError>;

    fn get_reg(&self) -> Register;
    fn get_vref(&self) -> f64;
    fn send_data(&mut self, data: &[u8; 3]) -> Result<(), IError>;
    fn read_register(&mut self, addr: &[u8; 3], dest: &mut [u8]) -> Result<(), IError>;

    fn set_code(&mut self, code: u16, target: ChannelAddress) -> Result<(), IError> {
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

        self.send_data(&data)?;
        Ok(())
    }
    fn set_voltage(&mut self, vol: f64, target: ChannelAddress) -> Result<(), IError> {
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
        self.send_data(&data)?;
        Ok(())
    }

    fn set_gain(&mut self, value: u16) -> Result<(), IError> {
        let data = MainBuilder::default()
            .write(WriteMode::Gain)
            .address(ChannelAddress::AllCh)
            .data(value)
            .build();
        self.send_data(&data)?;
        Ok(())
    }

    fn set_offset(&mut self, value: u16) -> Result<(), IError> {
        let data = MainBuilder::default()
            .write(WriteMode::Offset)
            .address(ChannelAddress::AllCh)
            .data(value)
            .build();
        self.send_data(&data)?;
        Ok(())
    }

    fn voltage_to_input(&self, vol: f64, group: u8, ch: u8) -> u16 {
        let vs = 0.0;
        let k1 = 1_u32 << 16_u32;
        let k2 = 1_u16 << 15_u16;
        let reg = self.get_reg();
        let ofs: u16 = match group {
            0 => reg.ofs0,
            _ => reg.ofs1,
        };
        let idx = (group * 8 + ch) as usize;
        let c = reg.offset[idx];
        let m = reg.gain[idx];

        let first_item = (vol - vs) * (k1 as f64) / (4.0 * self.get_vref());
        let suffix = (4 * ofs + k2 - c) as f64;
        let coef = (k1 as f64 / (m + 1) as f64) as f64;

        ((first_item + suffix) * coef).round() as u16
    }

    #[allow(dead_code)]
    fn read_all(&mut self) -> Result<(), IError> {
        let mut builder = MainBuilder::default();
        let mut reg = self.get_reg();
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
                    self.read_register(&prefix[item], data[item].as_mut())?;
                    println!(
                        "read reg (group:{},ch:{}){} , value:{:}",
                        group, ch, name[item], &data[item]
                    );
                }
                reg.x1_a[ch as usize] = data[0].to_u16();
                reg.x1_b[ch as usize] = data[1].to_u16();
                reg.offset[ch as usize] = data[2].to_u16();
                reg.gain[ch as usize] = data[3].to_u16();
            }
        }

        let mut data = ReadResp::new();

        let mut prefix: [u8; 3] = builder.read(ReadBackAddr::OFS0).build();

        self.read_register(&prefix, data.as_mut())?;
        reg.ofs0 = data.to_u16();
        println!("read reg ofs0, value:{:}", &data);

        prefix = (*builder.read(ReadBackAddr::OFS1)).build();
        self.read_register(&prefix, data.as_mut())?;
        reg.ofs1 = data.to_u16();
        println!("read reg ofs1, value:{:}", &data);

        self.read_register(&prefix, data.as_mut())?;
        //TODO set_reg here
        Ok(())
    }
}

impl<E, R> AD5370Instance for AD5370<E, R>
where
    IError: From<E>,
    IError: From<R>,
{
    fn init(&mut self) -> Result<(), IError> {
        //TODO: use button on the EVAL board for now. please keep LK3 connected.
        // self._reset.reset();
        // sleep(Duration::from_millis(1));
        // self._reset.set()?;

        self._clr.set_high()?;
        self._ldac.set_high()?;
        Ok(())
    }
    fn reset(&mut self) -> Result<(), IError> {
        self._reset.set_low()?;
        self._reset.set_high()?;
        Ok(())
    }
    // On the rising
    // edge of RESET, the AD5370 state machine initiates a reset
    // sequence to reset the X, M, and C registers to their default
    // values.

    fn clear(&mut self) -> Result<(), IError> {
        self._clr.set_low()?;
        Ok(())
    }
    fn get_reg(&self) -> Register {
        self.reg
    }

    fn get_vref(&self) -> f64 {
        self.vref
    }

    fn send_data(&mut self, data: &[u8; 3]) -> Result<(), IError> {
        for byte in data {
            self._spi.send(*byte).map_err(|_x| IError::General {
                msg: "unknown spi write",
            })?;
        }
        Ok(())
    }

    fn read_register(&mut self, addr: &[u8; 3], dest: &mut [u8]) -> Result<(), IError> {
        self.send_data(addr)?;
        for i in dest {
            self._spi.send(0x00).map_err(|_x| IError::General {
                msg: "unknown spi write",
            })?;
            *i = self._spi.read().map_err(|_x| IError::General {
                msg: "unknown spi read",
            })?;
        }
        Ok(())
    }

    fn set_ldac(&mut self) -> Result<(), IError> {
        self._ldac.set_high()?;
        Ok(())
    }
    fn clear_ldac(&mut self) -> Result<(), IError> {
        self._ldac.set_low()?;
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
