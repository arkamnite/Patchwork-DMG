use std::fmt::{Display, Formatter};
use std::num::ParseIntError;

/// An object that represents a "register pair" which is found in the DMG unit.
///
pub struct RegPair {
    pub high_bits: u8,
    low_bits: u8,
}

/// Whenever an operation is performed on a register, there may be an overflow etc.
/// All operations will return a BitResult that will indicate whether there are any important
/// considerations once an operation has completed.
enum BitResult {
    Standard,
    Overflow,
    Underflow,
}

impl RegPair {
    pub fn new() -> RegPair {
        RegPair {
            high_bits: 0,
            low_bits: 0,
        }
    }

    /// Set the bits in the "high" register to a given value, using BCD representation.
    pub fn set_high(self, val: u8) -> BitResult {
        todo!()
    }

    /// Set the bits in the "low" register to a given value, using BCD representation.
    pub fn set_low(self, val: u8) -> BitResult {
        todo!()
    }

    pub fn set_wide(self, val: u16) -> BitResult {
        todo!()
    }

    /// Returns the 16-bit integer value held across this register pair.
    pub fn to_int(self) -> u16 {
        todo!()
    }

    /// Returns the 8-bit integer value stored in the low register.
    pub fn get_low(self) -> u8 {
        self.low_bits
    }

    /// Returns the 8-bit integer value stored in the high register.
    pub fn get_high(self) -> u8 {
        self.high_bits
    }

    /// This will convert an 8-bit integer stored in BCD to the standard decimal representation.
    /// # Examples:
    fn bcd_to_decimal(bcd: u8) -> u8 {
        todo!()
    }

    /// This will convert a decimal representation of an 8-bit integer into BCD.
    fn decimal_to_bcd(dec: u8) -> Result<u8, ParseIntError> {
        // Store digits in array. There are 4 bits per digit- and hence only 2 real digits stored.
        let mut dig_array: [u8; 2] = [0; 2];
        dig_array[1] = dec % 10;
        dig_array[0] = (dec / 10) % 10;
        // Debug line.
        // println!("{} {} {:04b} {:04b}", dig_array[0], dig_array[1], dig_array[0], dig_array[1]);

        // Use the str_radix method to get the correct BCD number. However, this is quite unideal.
        let mut bcd_1 = format!("{:04b}", dig_array[0]);
        let bcd_2 = format!("{:04b}", dig_array[1]);
        bcd_1.push_str(bcd_2.as_str());
        u8::from_str_radix(bcd_1.as_str(), 2)
    }
}

impl Display for RegPair {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bcd() {
        let test_in = 42;
        let bcd = RegPair::decimal_to_bcd(42);
        assert_eq!(bcd, Ok(0b0100_0010));
    }
}