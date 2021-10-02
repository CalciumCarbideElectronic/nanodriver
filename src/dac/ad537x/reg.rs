use super::{utils::to_ch_seq, AD5370PerChannelRegister};

#[derive(Copy, Clone)]
pub enum SpecialFunctionAddress {
    WriteControl,
    WriteOFS0,
    WriteOFS1,
    ReadBack,
    //Write data in F7:F0 to A/B Select x
    WriteSelect { group: u8 },
    WriteSelectAll,
}
impl From<SpecialFunctionAddress> for u8 {
    fn from(c: SpecialFunctionAddress) -> Self {
        match c {
            SpecialFunctionAddress::WriteControl => 1,
            SpecialFunctionAddress::WriteOFS0 => 2,
            SpecialFunctionAddress::WriteOFS1 => 3,
            SpecialFunctionAddress::ReadBack => 5,
            SpecialFunctionAddress::WriteSelect { group } => (6 + group),
            SpecialFunctionAddress::WriteSelectAll => 11,
        }
    }
}

#[derive(Copy, Clone)]
pub enum ChannelAddress {
    AllCh,
    SingleCh { ch: u8, group: u8 },
    SingleGroup { group: u8 },
    Chx { ch: u8 },
    ChxExceptGroup0 { ch: u8 },
}
impl From<ChannelAddress> for u8 {
    fn from(c: ChannelAddress) -> Self {
        match c {
            ChannelAddress::AllCh => 0,
            ChannelAddress::SingleCh { ch, group } => ((group + 1) << 3) | ch,
            ChannelAddress::SingleGroup { group } => group + 1,
            ChannelAddress::Chx { ch } => (6 << 3) | ch,
            ChannelAddress::ChxExceptGroup0 { ch } => (7 << 3) | ch,
        }
    }
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
#[derive(Copy, Clone)]
pub enum WriteMode {
    //Writes to the DAC input data (X) register, depending on the control register A/B bit
    Data = 3,
    //Writes to the DAC offset (C) register
    Offset = 2,
    //Writes to the DAC gain (M) register
    Gain = 1,
}

#[derive(Clone, Copy)]
pub struct Register {
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
    // Bit 2 = A/B Select
    // Bit 1 = enable(1)/disable(0) temperature shutdown.
    // Bit 0 = soft power-down(1)/power-up(0).
    pub control: u8,
    // Each bit in this register determines if a DAC channel in Group x takes its data from Register X2A(0) or X2B(1).
    pub select: [u8; 5],
}
impl Default for Register {
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
