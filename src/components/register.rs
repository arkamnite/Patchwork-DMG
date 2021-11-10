use std::fmt::{Display, Formatter};
use std::num::ParseIntError;

/// An object that represents a "register pair" which is found in the DMG unit.
pub struct RegPair {
    high_bits: u8,
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
    pub fn new() -> Self {
        RegPair {
            high_bits: 0,
            low_bits: 0,
        }
    }

    /// Set the bits in the "high" register to a BCD encoding of the given value.
    pub fn set_high_bcd(&mut self, val: u8) -> Result<u8, ParseIntError> {
        // Collect the BCD representation of the input number.
        let bcd = RegPair::decimal_to_bcd(val)?; // If there is an error, exit early.
        self.high_bits = bcd;
        Ok(bcd)
    }

    /// Set the bits in the "low" register to a BCD encoding of the given value.
    pub fn set_low_bcd(&mut self, val: u8) -> Result<u8, ParseIntError> {
        // Collect the BCD representation of the input number.
        let bcd = RegPair::decimal_to_bcd(val)?; // Exit early if there are any errors.
        self.low_bits = bcd;
        Ok(bcd)
    }

    /// Set the bits across both the high and low registers directly.
    /// This will not encode them in BCD! If you need to encode these values into BCD, then use successive calls to
    /// set_high_bcd and set_low_bcd respectively.
    pub fn set_wide(&mut self, val: u16) -> BitResult {
        // Split the value into two bytes
        let low = ((val << 8) >> 8) as u8; // No freaking idea why this works, most likely to because of right-hand padding which we then populate
                                                // with a right-hand shift.
        let high = (val >> 8 ) as u8;
        // println!("{:08b}, {:016b}", high, low);
        self.high_bits = high;
        self.low_bits = low;
        BitResult::Standard
    }

    /// Returns the 16-bit integer value held across this register pair.
    pub fn to_int(&self) -> u16 {
        // Convert the 2-digit BCD value in each register to decimal.
        let hi_dec = RegPair::bcd_to_decimal(self.high_bits);
        let lo_dec = RegPair::bcd_to_decimal(self.low_bits);
        (hi_dec * 100 + lo_dec) as u16
    }

    /// Returns the 8-bit integer value stored in the low register.
    pub fn get_low(&self) -> u8 {
        self.low_bits
    }

    /// Returns the 8-bit integer value stored in the high register.
    pub fn get_high(&self) -> u8 {
        self.high_bits
    }

    /// This will convert a 2-digit 8-bit integer stored in BCD to the standard decimal representation.
    /// # Examples:
    pub fn bcd_to_decimal(bcd: u8) -> u8 {
        // Look at first 4 bits.
        let dig1 = bcd >> 4;
        let dig2 = (bcd << 4) >> 4;
        (dig1 * 10) + dig2
    }

    /// This will convert a decimal representation of an 8-bit integer into BCD.
    pub fn decimal_to_bcd(dec: u8) -> Result<u8, ParseIntError> {
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
    fn bcd() {
        let bcd = RegPair::decimal_to_bcd(32);
        assert_eq!(bcd, Ok(0b0011_0010));
    }

    #[test]
    fn set_high() {
        let mut reg = RegPair::new();
        reg.set_high_bcd(32);
        assert_eq!(reg.get_high(), 0b0011_0010);
    }

    #[test]
    fn set_low() {
        let mut reg = RegPair::new();
        reg.set_low_bcd(32);
        assert_eq!(reg.get_low(), 0b0011_0010);
    }

    #[test]
    fn set_wide() {
        let mut reg = RegPair::new();
        reg.set_wide(0b0011_0101_1100_1011);
        assert_eq!(reg.get_high(), 0b0011_0101);
        assert_eq!(reg.get_low(), 0b1100_1011);
    }

    #[test]
    fn bcd_to_decimal() {
        let dec = RegPair::bcd_to_decimal(0b0011_0010); // BCD for 32
        assert_eq!(dec, 32);
    }
}