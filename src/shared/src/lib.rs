pub mod error;
pub mod locations;
pub mod paths;
pub mod position;
pub mod profession;

pub fn read_flags(byte: i8) -> [bool; 8] {
    [
        byte as u8 & 0b0000_0001 != 0,
        byte as u8 & 0b0000_0010 != 0,
        byte as u8 & 0b0000_0100 != 0,
        byte as u8 & 0b0000_1000 != 0,
        byte as u8 & 0b0001_0000 != 0,
        byte as u8 & 0b0010_0000 != 0,
        byte as u8 & 0b0100_0000 != 0,
        byte as u8 & 0b1000_0000 != 0,
    ]
}