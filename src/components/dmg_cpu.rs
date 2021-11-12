use crate::components::register::RegPair;

pub struct CPU {
    a: u8,
    sp: u16,
    pc: u16,
    adr: u16,
    bc: RegPair,
    de: RegPair,
    hl: RegPair,
    flags: Flags,
    lcd_reg: LCDReg,
    pub memory: [u16; 65536],
    pub cycles: u32,
}

/// Representation of the status flags within the CPU.
/// - (Z) Zero flag
/// - (N) Subtraction flag for BCD
/// - (H) Half-carry flag for BCD
/// - (C) Carry flag
struct Flags {
    /// Z flag
    pub zero: bool,
    /// N flag, for BCD
    pub subtraction: bool,
    /// H flag, for BCD
    pub half_carry: bool,
    /// C flag
    pub carry: bool,
}

impl Flags {
    pub fn new() -> Self {
        Flags {
            zero: false,
            subtraction: false,
            half_carry: false,
            carry: false
        }
    }
}

/// Representation of the LCD control register.
struct LCDReg {
    /// Bit 7 - LCD Display Enable (0=Off, 1=On)
    pub lcd_enable: bool,
    /// Bit 6 - Window Tile Map Display Select (0=9800-9BFF, 1=9C00-9FFF)
    pub window_display_select: bool,
    /// Bit 5 - Window Display Enable (0=Off, 1=On)
    pub window_enable: bool,
    /// Bit 4 - BG & Window Tile Data Select (0=8800-97FF, 1=8000-8FFF)
    pub bg_window_select: bool,
    /// Bit 3 - BG Tile Map Display Select (0=9800-9BFF, 1=9C00-9FFF)
    pub bg_tile_data_select: bool,
    /// Bit 2 - OBJ (Sprite) Size (0=8x8, 1=8x16)
    pub sprite_size: bool,
    /// Bit 1 - OBJ (Sprite) Display Enable (0=Off, 1=On)
    pub sprite_enable: bool,
    /// Bit 0 - BG Display (for CGB see below) (0=Off, 1=On)
    pub bg_display_cgb: bool,
}

impl LCDReg {
    pub fn new() -> Self {
        LCDReg {
            lcd_enable: false,
            window_display_select: false,
            window_enable: false,
            bg_window_select: false,
            bg_tile_data_select: false,
            sprite_size: false,
            sprite_enable: false,
            bg_display_cgb: false
        }
    }
}

enum AddressingMode {
    ImmediateEight,
    ImmediateSixteen,
    BigEndianSixteen,
    UnsignedEight(u8),
    AddressSixteen(u16),
    SignedEight(u8),
}

impl CPU {

    pub fn new() -> Self {
        CPU {
            a: 0,
            sp: 0,
            pc: 0,
            adr: 0,
            bc: RegPair::new(),
            de: RegPair::new(),
            hl: RegPair::new(),
            flags: Flags::new(),
            lcd_reg: LCDReg::new(),
            memory: [0; 65536],
            cycles: 0
        }
    }

    pub fn cycle(&mut self) {
        self.pc = self.pc + 1;
    }

    /// Given an opcode, this function will decode this using pattern matching and will hence
    fn decode_execute(&self, opcode: u16) {

    }

    /// Will load the address buffer appropriately based on the provided addressing mode.
    pub fn read_memory(&self, mode: AddressingMode) -> u16 {
        let mem_val: u16 = match mode {
            AddressingMode::ImmediateEight => { self.memory[self.pc as usize] }
            AddressingMode::ImmediateSixteen => {
                // Upper bytes are in first byte of memory.
                let mut val = self.memory[(self.pc + 1) as usize];
                // We now collect the upper bytes from the second byte in memory.
                println!("{:0x}", val);
                val <<= 8;
                val = val + self.memory[self.pc as usize];
                println!("{:0x}", val);
                // We now combine the two bytes together.
                val
            }
            AddressingMode::BigEndianSixteen => {self.memory[self.pc as usize]}
            AddressingMode::UnsignedEight(_) => {self.memory[self.pc as usize]}
            AddressingMode::AddressSixteen(_) => {self.memory[self.pc as usize]}
            AddressingMode::SignedEight(_) => {self.memory[self.pc as usize]}
        };

        mem_val
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::dmg_cpu::AddressingMode::ImmediateSixteen;

    #[test]
    fn memory_be_read() {
        let mut cpu = CPU::new();
        cpu.memory[0] = 0xCD;
        cpu.memory[1] = 0xAB;
        assert_eq!(0xABCD, cpu.read_memory(ImmediateSixteen));
    }
}