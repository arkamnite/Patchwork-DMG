use crate::components::dmg_cpu::AddressingMode::ImmediateSixteen;
use crate::components::dmg_cpu::RegisterPairs::BC;
use crate::components::register::{BitResult, RegPair};

pub struct CPU {
    /// The accumulator register.
    a: u8,
    /// The stack pointer.
    sp: u16,
    /// The program counter.
    pc: u16,
    /// The instruction register. Stores the current instruction.
    ir: u16,
    /// The memory address register. Stores the address that memory must either be read/written from/to.
    mar: u16,
    /// The memory data register. Stores the data retrieved from memory.
    mdr: u16,
    /// The BC register pair.
    bc: RegPair,
    /// The DE register pair.
    de: RegPair,
    /// The HL register pair.
    hl: RegPair,
    /// The status flag(s) register. This is defined as a Flags struct.
    flags: Flags,
    /// The LCD control register.
    lcd_reg: LCDReg,
    /// The total memory access space of the DMG unit.
    pub memory: [u16; 65536],
    /// The number of cycles clocked so far.
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

// Lifetime parameter used here as we need to know the lifetime of the register-pair we are borrowing from.
enum AddressingMode<'a> {
    Implied, // Stuff like CPL and LD SP,IY
    ImmediateEight, /// Found in the next instruction.
    ImmediateSixteen, /// Found in the next two instructions.
    UnsignedEight, /// Used particularly for 0xE0 and 0xF0, where an offset of 0xFF00 is used.
    AddressSixteen(u16),
    SignedEight(u8),
    RegisterPairDirect(&'a RegPair),
    RegisterDirect(&'a RegPair, bool),
}

/// This is used to specify which register pair we choose to operate on.
/// It alleviates the need for a mutable reference to a register pair whilst also having a mutable reference to self (the CPU).
enum RegisterPairs {
    BC,
    DE,
    HL,
}

enum Registers {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    SP,
}

impl CPU {

    pub fn new() -> Self {
        CPU {
            a: 0,
            sp: 0,
            pc: 0,
            ir: 0,
            mar: 0,
            mdr: 0,
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
        // Fetch opcode
        self.ir = self.memory[self.pc as usize];
        // Program counter is incremented to enable operand reading.
        self.pc += 1;
        // Decode the opcode and execute.
        self.decode_execute();
    }

    /// Given the stored opcode, this function will decode this using pattern matching and will hence
    fn decode_execute(&mut self) {
        // Match on the current opcode.
        match self.ir {
            0x00 => { self.cycles += 4; }  // NOP
            0x01 => {
                // LD BC,d16
                self.mdr = self.read_memory(ImmediateSixteen);
                self.ld_reg_pair(BC);
            }
            0x02 => {
                // LD (BC), A
                self.mdr = self.a as u16;
                self.mar = self.bc.get_wide();
                self.ld_memory();
            }
            0x03 => { self.bc.set_wide(self.bc.get_wide() + 1); self.pc += 1; self.cycles += 8; }  // INC BC
            0x04 => { self.bc.set_high_bin(self.bc.get_high() + 1); }  // INC B
            0x05 => {}  // DEC B
            0x06 => {}  // LD B,d8
            0x07 => {}  // RLCA
            0x08 => {}  // LD (a16),SP
            0x09 => {}  // ADD HL,BC
            0x0A => {}  // LD A,(BC)
            0x0B => {}  // DEC BC
            0x0C => {}  // INC C
            0x0D => {}  // DEC C
            0x0E => {}  // LD C,d8
            0x0F => {}  // RRCA

            0x10 => {}
            0x11 => {}
            0x12 => {}
            0x13 => {}
            0x14 => {}
            0x15 => {}
            0x16 => {}
            0x17 => {}
            0x18 => {}
            0x19 => {}
            0x1A => {}
            0x1B => {}
            0x1C => {}
            0x1D => {}
            0x1E => {}
            0x1F => {}

            0x20 => {}
            0x21 => {}
            0x22 => {}
            0x23 => {}
            0x24 => {}
            0x25 => {}
            0x26 => {}
            0x27 => {}
            0x28 => {}
            0x29 => {}
            0x2A => {}
            0x2B => {}
            0x2C => {}
            0x2D => {}
            0x2E => {}
            0x2F => {}

            0x30 => {}
            0x31 => {}
            0x32 => {}
            0x33 => {}
            0x34 => {}
            0x35 => {}
            0x36 => {}
            0x37 => {}
            0x38 => {}
            0x39 => {}
            0x3A => {}
            0x3B => {}
            0x3C => {}
            0x3D => {}
            0x3E => {}
            0x3F => {}

            0x40 => {}
            0x41 => {}
            0x42 => {}
            0x43 => {}
            0x44 => {}
            0x45 => {}
            0x46 => {}
            0x47 => {}
            0x48 => {}
            0x49 => {}
            0x4A => {}
            0x4B => {}
            0x4C => {}
            0x4D => {}
            0x4E => {}
            0x4F => {}

            0x50 => {}
            0x51 => {}
            0x52 => {}
            0x53 => {}
            0x54 => {}
            0x55 => {}
            0x56 => {}
            0x57 => {}
            0x58 => {}
            0x59 => {}
            0x5A => {}
            0x5B => {}
            0x5C => {}
            0x5D => {}
            0x5E => {}
            0x5F => {}

            0x60 => {}
            0x61 => {}
            0x62 => {}
            0x63 => {}
            0x64 => {}
            0x65 => {}
            0x66 => {}
            0x67 => {}
            0x68 => {}
            0x69 => {}
            0x6A => {}
            0x6B => {}
            0x6C => {}
            0x6D => {}
            0x6E => {}
            0x6F => {}

            0x70 => {}
            0x71 => {}
            0x72 => {}
            0x73 => {}
            0x74 => {}
            0x75 => {}
            0x76 => {}
            0x77 => {}
            0x78 => {}
            0x79 => {}
            0x7A => {}
            0x7B => {}
            0x7C => {}
            0x7D => {}
            0x7E => {}
            0x7F => {}

            0x80 => {}
            0x81 => {}
            0x82 => {}
            0x83 => {}
            0x84 => {}
            0x85 => {}
            0x86 => {}
            0x87 => {}
            0x88 => {}
            0x89 => {}
            0x8A => {}
            0x8B => {}
            0x8C => {}
            0x8D => {}
            0x8E => {}
            0x8F => {}

            0x90 => {}
            0x91 => {}
            0x92 => {}
            0x93 => {}
            0x94 => {}
            0x95 => {}
            0x96 => {}
            0x97 => {}
            0x98 => {}
            0x99 => {}
            0x9A => {}
            0x9B => {}
            0x9C => {}
            0x9D => {}
            0x9E => {}
            0x9F => {}

            0xA0 => {}
            0xA1 => {}
            0xA2 => {}
            0xA3 => {}
            0xA4 => {}
            0xA5 => {}
            0xA6 => {}
            0xA7 => {}
            0xA8 => {}
            0xA9 => {}
            0xAA => {}
            0xAB => {}
            0xAC => {}
            0xAD => {}
            0xAE => {}
            0xAF => {}

            0xB0 => {}
            0xB1 => {}
            0xB2 => {}
            0xB3 => {}
            0xB4 => {}
            0xB5 => {}
            0xB6 => {}
            0xB7 => {}
            0xB8 => {}
            0xB9 => {}
            0xBA => {}
            0xBB => {}
            0xBC => {}
            0xBD => {}
            0xBE => {}
            0xBF => {}

            0xC0 => {}
            0xC1 => {}
            0xC2 => {}
            0xC3 => {}
            0xC4 => {}
            0xC5 => {}
            0xC6 => {}
            0xC7 => {}
            0xC8 => {}
            0xC9 => {}
            0xCA => {}
            0xCB => {}
            0xCC => {}
            0xCD => {}
            0xCE => {}
            0xCF => {}

            0xD0 => {}
            0xD1 => {}
            0xD2 => {}
            0xD3 => {}
            0xD4 => {}
            0xD5 => {}
            0xD6 => {}
            0xD7 => {}
            0xD8 => {}
            0xD9 => {}
            0xDA => {}
            0xDB => {}
            0xDC => {}
            0xDD => {}
            0xDE => {}
            0xDF => {}

            0xE0 => {}
            0xE1 => {}
            0xE2 => {}
            0xE3 => {}
            0xE4 => {}
            0xE5 => {}
            0xE6 => {}
            0xE7 => {}
            0xE8 => {}
            0xE9 => {}
            0xEA => {}
            0xEB => {}
            0xEC => {}
            0xED => {}
            0xEE => {}
            0xEF => {}

            0xF0 => {}
            0xF1 => {}
            0xF2 => {}
            0xF3 => {}
            0xF4 => {}
            0xF5 => {}
            0xF6 => {}
            0xF7 => {}
            0xF8 => {}
            0xF9 => {}
            0xFA => {}
            0xFB => {}
            0xFC => {}
            0xFD => {}
            0xFE => {}
            0xFF => {}
            _ => {
                // NOP
            }
        }

    }

    /// Will return the correct value from memory that shall be stored in the memory data register.
    /// This function will not modify the PC (indicated by the non-mutable reference to self).
    /// This decision has been made so that read_memory() can be used to read a value directly as well
    /// as for debugging purposes.
    pub fn read_memory(&self, mode: AddressingMode) -> u16 {
        let mem_val: u16 = match mode {
            AddressingMode::Implied => {
                // We leave the CPU exactly as it was before.
                // We may later on "automate" the procedure of setting the MDR based on the result
                // of reading memory, and hence it is ideal to return the original value of the MDR.
                self.mdr
            }
            AddressingMode::ImmediateEight => {
                self.memory[self.pc as usize] as u16
            }
            AddressingMode::ImmediateSixteen => {
                // Upper bytes are in first byte of memory.
                let mut val = self.memory[(self.pc + 1) as usize];
                // We now collect the upper bytes from the second byte in memory.
                // println!("{:0x}", val);
                val <<= 8;
                val = val + self.memory[self.pc as usize];
                // println!("{:0x}", val);
                // We now combine the two bytes together.
                val
            }
            AddressingMode::UnsignedEight => {
                // This mode only uses the operand as an offset for 0xFF00, and hence we only need to add the value to
                self.memory[(0xFF00 + self.pc) as usize]
            }
            AddressingMode::AddressSixteen(val) => {
                self.memory[val as usize]
            }
            AddressingMode::SignedEight(val) => {
                // This will take the signed operand in memory, and convert it from TC to an unsigned 16 bit integer.
                from_signed_byte(self.memory[self.pc as usize] as u8) as u16
            }
            AddressingMode::RegisterPairDirect(reg) => {
                self.memory[reg.get_wide() as usize]
            }
            AddressingMode::RegisterDirect(reg, isHigh) => {
                // We will read from the memory address in either the high or low byte of the RegPair
                if isHigh {
                    // println!("Reg value is {:#04x}", reg.get_high());
                    self.memory[reg.get_high() as usize]
                } else {
                    // println!("Reg value is {:#04x}", reg.get_low());
                    self.memory[reg.get_low() as usize]
                }
            }
            _ => { self.memory[self.pc as usize] as u16}
        };

        mem_val
    }

    /*  OPCODES BEGIN HERE. */

    /// Load a value stored in the MDR into a register pair.
    /// Example: LD BC, d16.
    /// This function will read memory whilst advancing the PC accordingly.
    fn ld_reg_pair(&mut self, reg: RegisterPairs) -> BitResult {
        // Load the MDR value appropriately.
        // self.mdr = self.read_memory(ImmediateSixteen);

        // The MDR must already have been initialised!
        // Set the CPU RP appropriately.
        let res = match reg {
            RegisterPairs::BC => { self.bc.set_wide(self.mdr) }
            RegisterPairs::DE => { self.de.set_wide(self.mdr) }
            RegisterPairs::HL => { self.hl.set_wide(self.mdr) }
        };
        // Advance the PC and the cycles appropriately.
        self.pc += 2;
        self.cycles += 12;
        res
    }

    /// Load a value stored in the MDR into the memory address stored in MAR.
    /// Example: LD (BC), A. Where A has been stored in the MDR.
    /// PC += 1, Cycles += 8.
    fn ld_memory(&mut self) {
        // Load the value in the MDR into the memory address stored in MAR.
        self.memory[self.mar as usize] = self.mdr;
        // Increment cycles and PC appropriately.
        self.pc += 1;
        self.cycles += 8;
    }

    fn inc_reg(&mut self, reg: Registers) {
        // Modify the correct register.
        match reg {
            Registers::A => { self.a += 1; }
            Registers::B => { self.bc.set_high_bin(self.bc.get_high() + 1); }
            Registers::C => { self.bc.set_low_bin(self.bc.get_low() + 1); }
            Registers::D => { self.de.set_high_bin(self.de.get_high() + 1); }
            Registers::E => { self.de.set_low_bin(self.de.get_low() + 1); }
            Registers::H => { self.hl.set_high_bin(self.hl.get_high() + 1); }
            Registers::L => { self.hl.set_low_bin(self.hl.get_low() + 1); }
            Registers::SP => { self.sp += 1; }
        }
    }
}

pub fn msb(v: u16) -> u8 {
    (v >> 8) as u8
}

pub fn lsb(v: u16) -> u8 {
    ((v << 8) >> 8) as u8
}

/// Will convert an 8-bit number represented in TC to an 8-bit signed number.
pub fn from_signed_byte(tc: u8) -> i8 {
    // Check if we have a negative number.
    if (tc & 0b1000_000) == 0b1000_000 {
        // Find out what this number is the negative of.
        let flipped = !tc + 1;
        flipped as i8 * -1
    } else { // Non-negative number, so return the number as it is.
        tc as i8
    }
}

/// Will convert an 8-bit signed number to an 8-bit unsigned number represented in TC.
pub fn get_magnitude_tc(from: i8) -> u8 {
    // Check if we have a negative number.
    if from < 0 {
        (from * -1) as u8
    } else {
        from as u8
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::format;
    use super::*;
    use crate::components::dmg_cpu::AddressingMode::*;

    #[test]
    fn msb_lsb() {
        assert_eq!(0xAB, msb(0xABCD));
        assert_eq!(0xCD, lsb(0xABCD));
    }

    #[test]
    fn from_tc() {
        assert_eq!(-59, from_signed_byte(0b1100_0101));
        assert_eq!(-90, from_signed_byte(0b1010_0110));
        assert_eq!(32, from_signed_byte(0b0010_0000));
    }

    #[test]
    fn to_tc() {
        assert_eq!(0b0010_0000, get_magnitude_tc(32));
        assert_eq!(90, get_magnitude_tc(-90));
        // This fails as a function because the computer already stores these as
    }

    #[test]
    fn immediate_memory_read() {
        let mut cpu = CPU::new();
        cpu.memory[0] = 0xCD;
        cpu.memory[1] = 0xAB;
        cpu.memory[2] = 0b1110_0010; // -30;
        cpu.memory[0xFF00 + 2] = 0xFEE2;
        assert_eq!(0xCD, cpu.read_memory(ImmediateEight)); // 8-bit immediate reading, such as with opcode 0x06: LD B, d8
        assert_eq!(0xABCD, cpu.read_memory(ImmediateSixteen)); // 16-bit immediate reading, such as with opcode LD HL, d16
        cpu.cycle();
        cpu.cycle();
        assert_eq!(0xFEE2, cpu.read_memory(UnsignedEight)); // Come back and look at def for this addressing type.

        // Register(pair) Direct mode
        let mut reg = RegPair::new();
        reg.set_wide(0x1346);
        cpu.memory[0x1346] = 1334;
        cpu.memory[0x13] = 15;
        cpu.memory[0x46] = 32;
        assert_eq!(1334, cpu.read_memory(RegisterPairDirect(&reg)));
        assert_eq!(15, cpu.read_memory(RegisterDirect(&reg, true)));
        assert_eq!(32, cpu.read_memory(RegisterDirect(&reg, false)));
    }
}

#[cfg(test)]
mod opcode_tests {
    // Opcode tests follow this naming convention.
    // name_to_from()
    // For example, 0x02 (LD (BC), A) => ld_mem_16_reg
    // This may be followed by an addressing mode specifier, or an annotation to distinguish this test from similar to-from scenarios.

    #[test]
    fn ld_mem_8_reg() {

    }

    #[test]
    fn ld_mem_16_reg() {

    }

    #[test]
    fn ld_reg_reg() {

    }

    #[test]
    fn ld_reg_mem_8() {

    }

    #[test]
    fn ld_reg_mem_16() {

    }
}

#[cfg(test)]
mod opcodes {
    #[test]
    fn o1() {

    }
}
