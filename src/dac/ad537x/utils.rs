
pub fn to_ch_seq(group: u8, ch: u8) -> u16 {
    (group * 8 + ch + 8) as u16
}