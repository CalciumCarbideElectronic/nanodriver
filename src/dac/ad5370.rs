#![allow(dead_code)]
use crate::{
    error::IError,
    interface::{gpio::IOController, spi::Transactional},
};

type AD5370PerChannelRegister = [u16; 40];

#[derive(Clone, Copy)]
struct ReadResp([u8; 3]);

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
// impl From<[u8; 3]> for u16 {
//     fn from(a: [u8; 3]) -> Self {
//     }
// }

type DataType = u32;
#[derive(Copy, Clone)]
pub struct MainBuilder(u32);
pub trait DataBuilder {
    fn build(&mut self, d: u32) -> u32;
}
#[derive(Copy, Clone)]
pub enum Address {
    AllCh,
    SingleCh { ch: u8, group: u8 },
    SingleGroup { group: u8 },
    Chx { ch: u8 },
    ChxExceptGroup0 { ch: u8 },
    //Write to the Control register.
    WriteControl,
    // Write data in F13:F0 to OFS0 register.
    WriteOFS0,
    //Write data in F13:F0 to OFS1 register.
    WriteOFS1,
    //Select register for readback.
    ReadBack,
    //Write data in F7:F0 to A/B Select x
    WriteSelect { group: u8 },
    // Block write A/B select registers.
    // F7:F0 = 0, write all 0s (all channels use X2A register).
    // F7:F0 = 1, write all 1s (all channels use X2B register).
    WriteSelectAll,
}
impl From<Address> for u8 {
    fn from(c: Address) -> Self {
        match c {
            Address::AllCh => 0,
            Address::SingleCh { ch, group } => ((group + 1) << 3) | ch,
            Address::SingleGroup { group } => group + 1,
            Address::Chx { ch } => (6 << 3) | ch,
            Address::ChxExceptGroup0 { ch } => (7 << 3) | ch,
            Address::WriteControl => 1,
            Address::WriteOFS0 => 2,
            Address::WriteOFS1 => 3,
            Address::ReadBack => 5,
            Address::WriteSelect { group } => (6 + group),
            Address::WriteSelectAll => 11,
        }
    }
}

#[derive(Copy, Clone)]
pub enum Mode {
    //Writes to the DAC input data (X) register, depending on the control register A/B bit
    WriteData = 3,
    //Writes to the DAC offset (C) register
    WriteOffset = 2,
    //Writes to the DAC gain (M) register
    WriteGain = 1,
    //Special function, used in combination with other bits of the data-word
    SpecialFunction = 0,
}

#[derive(Copy, Clone)]
pub enum ReadBackAddr {
    X1A { group: u8, ch: u8 },
    X1B { group: u8, ch: u8 },
    C { group: u8, ch: u8 },
    M { group: u8, ch: u8 },
    Control,
    OFS0,
    OFS1,
    Select { group: u8 },
}

pub fn to_ch_seq(group: u8, ch: u8) -> u16 {
    (group * 8 + ch + 8) as u16
}

impl From<ReadBackAddr> for u16 {
    fn from(c: ReadBackAddr) -> Self {
        match c {
            ReadBackAddr::X1A { group, ch } => to_ch_seq(group, ch) << 7,
            ReadBackAddr::X1B { group, ch } => (1_u16 << 13) | to_ch_seq(group, ch) << 7,

            ReadBackAddr::C { group, ch } => (2_u16 << 13) | to_ch_seq(group, ch) << 7,
            ReadBackAddr::M { group, ch } => (3_u16 << 13) | to_ch_seq(group, ch) << 7,
            ReadBackAddr::Control => (4_u16 << 13) | (1_u16 << 7),
            ReadBackAddr::OFS0 => (4_u16 << 13) | (2_u16 << 7),
            ReadBackAddr::OFS1 => (4_u16 << 13) | (3_u16 << 7),
            ReadBackAddr::Select { group } => (4_u16 << 13) | ((group + 6) as u16) << 7,
        }
    }
}
impl Default for MainBuilder {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl MainBuilder {
    pub fn target(&mut self, mode: Mode) -> &mut Self {
        self.0 |= (mode as u32) << 22;
        self
    }
    pub fn address(&mut self, ch: Address) -> &mut Self {
        let code: u8 = ch.into();
        self.0 |= ((code as u32) << 16) as u32;
        self
    }
    pub fn data(&mut self, data: u16) -> &mut Self {
        self.0 |= data as u32;
        self
    }
    pub fn read(&mut self, addr: ReadBackAddr) -> &Self {
        self.target(Mode::SpecialFunction)
            .address(Address::ReadBack)
            .data(addr.into());
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
pub struct AD537xRegister {
    //Input Data Register A. One for each DAC channel.
    pub x1_a: AD5370PerChannelRegister,
    //Input Data Register B. One for each DAC channel.
    pub x1_b: AD5370PerChannelRegister,

    //Gain trim register. One for each DAC channel.
    pub gain: AD5370PerChannelRegister,
    //Offset trim register. One for each DAC channel.
    pub offset: AD5370PerChannelRegister,
    //Offset DAC 0 data register. Sets the offset for Group 0.
    pub ofs0: u16,
    //Offset DAC 1 data register. Sets the offset for Group 1 to Group 4.
    pub ofs1: u16,
    // Bit 2 = A/B.
    // 0 = global selection of X1A input data registers.
    // 1 = X1B registers.
    // Bit 1 = enable temperature shutdown.
    // 0 = disable temperature shutdown.
    // 1 = enable.
    // Bit 0 = soft power-down.
    // 0 = soft power-up.
    // 1 = soft power-down.
    pub control: u8,
    // Each bit in this register determines
    // if a DAC channel in Group x takes its data from Register X2A or X2B.
    // 0 = X2A.
    // 1 = X2B
    pub select: [u8; 5],
}
impl Default for AD537xRegister {
    fn default() -> Self {
        Self {
            x1_a: [0x1555; 40],
            x1_b: [0x1555; 40],
            gain: [0x3FFF; 40],
            offset: [0x2000; 40],
            ofs0: 0x1555,
            ofs1: 0x1555,
            control: 0x00,
            select: [0; 5],
        }
    }
}

pub struct AD5370 {
    vref: f64,
    reg: AD537xRegister,
    spi: Box<dyn Transactional>,

    ///BUSY Input/Output (Active Low). BUSY is open-drain when an output.
    ///See the BUSY and LDAC Functions section for more information
    _busy: Box<dyn IOController>,
    //Load DAC Logic Input (Active Low).
    _ldac: Box<dyn IOController>,
    //Digital Reset Input
    _reset: Box<dyn IOController>,
    ///Active Low Input.
    ///This is the frame synchronization signal for the serial interface.
    ///See the Timing Characteristics section for more details.
    _sync: Box<dyn IOController>,
    ///Asynchronous Clear Input (Level Sensitive, Active Low).
    ///See the Clear Function section for more information
    _clr: Box<dyn IOController>,
}

#[allow(dead_code)]
impl AD5370 {
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
    pub fn write(&mut self, data: [u8; 3]) -> Result<(), IError> {
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

    pub fn set_voltage(&mut self, vol: f64, target: Address) -> Result<(), IError> {
        let mut builder = MainBuilder(0);
        let (g, c) = match target {
            Address::AllCh => (0, 0),
            Address::SingleCh { ch, group } => (group, ch),
            Address::SingleGroup { group } => (group, 0),
            Address::Chx { ch } => (0, ch),
            Address::ChxExceptGroup0 { ch } => (1, ch),
            _ => unreachable!(),
        };
        builder
            .target(Mode::WriteData)
            .address(target)
            .data(self.voltage_to_input(vol, g, c));
        let data: [u8; 3] = builder.into();
        self.spi.spi_write(&data)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn read_all(&mut self) -> Result<(), IError> {
        let mut builder = MainBuilder(0);
        for group in 0..5 {
            for ch in 0..8 {
                let prefix: [[u8; 3]; 4] = [
                    builder.read(ReadBackAddr::X1A { group, ch }).into(),
                    builder.read(ReadBackAddr::X1B { group, ch }).into(),
                    builder.read(ReadBackAddr::C { group, ch }).into(),
                    builder.read(ReadBackAddr::M { group, ch }).into(),
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

        let mut prefix: [u8; 3] = builder.read(ReadBackAddr::OFS0).into();
        let mut data = ReadResp::new();
        self.spi.spi_read(&prefix, data.as_mut())?;
        self.reg.ofs0 = data.to_u16();

        prefix = (*builder.read(ReadBackAddr::OFS1)).into();
        self.spi.spi_read(&prefix, data.as_mut())?;

        self.spi.spi_read(&prefix, data.as_mut())?;
        Ok(())
    }
}

#[allow(clippy::unusual_byte_groupings)]
#[allow(dead_code)]
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_builder() {
        let data: [u8; 3] = MainBuilder::default()
            .target(Mode::WriteData)
            .address(Address::Chx { ch: 5 })
            .data(0xA5A5)
            .to_owned()
            .into();

        assert_eq!([0b11_110101, 0xA5, 0xA5], data);

        let data: [u8; 3] = MainBuilder::default()
            .target(Mode::WriteOffset)
            .address(Address::SingleCh { group: 3, ch: 7 })
            .data(0xA5A5)
            .to_owned()
            .into();

        assert_eq!([0b10_100111, 0xA5, 0xA5], data);

        let data: [u8; 3] = MainBuilder::default()
            .read(ReadBackAddr::M { group: 4, ch: 7 })
            .to_owned()
            .into();

        assert_eq!([0b00_000101, 0b011_10111, 0b1000_0000], data);

        let data: [u8; 3] = MainBuilder::default()
            .read(ReadBackAddr::Select { group: 3 })
            .to_owned()
            .into();

        assert_eq!([0b00_000101, 0b100_00100, 0b1000_0000], data);

        let data: [u8; 3] = MainBuilder::default()
            .read(ReadBackAddr::OFS0)
            .to_owned()
            .into();
        assert_eq!([0b00_000101, 0b100_00001, 0b0000_0000], data);

        let data: [u8; 3] = MainBuilder::default()
            .read(ReadBackAddr::X1A { group: 2, ch: 2 })
            .to_owned()
            .into();
        assert_eq!([0b00_000101, 0b000_01101, 0b0000_0000], data);
    }
}
