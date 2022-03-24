use std::{error, fmt};
use std::fmt::Formatter;
use crate::components::register::{BitResult, RegPair};
use std::num::Wrapping;
use std::ops::Add;
use anyhow::{anyhow, Result}; // Used for anyhow's Result type for all fallible functions in our program. Imports the macro as well.
use thiserror::Error; // Allows us to create custom error types.

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
pub enum AddressingMode<'a> {
    Implied, // Stuff like CPL and LD SP,IY
    ImmediateEight, /// Found in the next instruction.
    ImmediateSixteen, /// Found in the next two instructions.
    UnsignedEight, /// Used particularly for 0xE0 and 0xF0, where an offset of 0xFF00 is used.
    AddressSixteen(u16),
    SignedEight,
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

enum RotateDirection {
    Left,
    Right,
}

#[derive(Debug, Clone)]
struct OpcodeError {
    info: String,
    opcode: u8,
}

impl OpcodeError {
    fn new(info: String, opcode: u8) -> OpcodeError {
        OpcodeError {
            info,
            opcode
        }
    }
}

impl fmt::Display for OpcodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Error when executing opcode: {:#04x}\n Message: {}", self.opcode, self.info)
    }
}

impl error::Error for OpcodeError {}

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
        // println!("Opcode found: {:#2x}", self.ir);
        // Program counter is incremented to enable operand reading.
        self.pc += 1;
        // Decode the opcode and execute.
        self.decode_execute();
    }

    /// Given the stored opcode, this function will decode this using pattern matching and will hence
    fn decode_execute(&mut self) {
        // Match on the current opcode.
        match self.ir {
            0x00 => { self.pc += 0; self.cycles += 4; }  // NOP
            0x01 => {
                // LD BC,d16
                self.mdr = self.read_memory(AddressingMode::ImmediateSixteen);
                self.ld_reg_pair(RegisterPairs::BC);
            }
            0x02 => {
                // LD (BC), A
                self.mdr = self.a as u16;
                self.mar = self.bc.get_wide();
                self.ld_memory();
                self.pc += 0;
                self.cycles += 8;
            }
            0x03 => { self.bc.set_wide(self.bc.get_wide() + 1); self.pc += 0; self.cycles += 8; }  // INC BC
            0x04 => { self.inc_reg_8(Registers::B).unwrap(); self.pc += 0; self.cycles += 8; }  // INC B
            0x05 => { self.dec_reg_8(Registers::B).unwrap(); self.pc += 0; self.cycles += 8;}  // DEC B
            0x06 => {
                // LD B,d8
                self.mdr = self.read_memory(AddressingMode::ImmediateEight);
                self.bc.set_high_bin(self.mdr as u8);
                self.pc += 1;
                self.cycles += 8;
            }
            0x07 => { self.rotate_a(RotateDirection::Left, false); self.pc += 0; self.cycles += 4; }  // RLCA
            0x08 => {
                // LD (a16),SP
                // Load the lower byte of SP at a16.
                self.mdr = (self.sp << 8) >> 8;
                self.mar = self.read_memory(AddressingMode::ImmediateSixteen);
                // println!("CPU MDR: {}, CPU MAR: {}", self.mdr, self.mar);
                self.ld_memory();
                // Load the upper byte of SP at a16 + 1;
                self.mdr = self.sp >> 8;
                self.mar = self.mar + 1;
                // println!("CPU MDR: {}, CPU MAR: {}", self.mdr, self.mar);
                self.ld_memory();
                // Increment PC and cycles accordingly.
                self.pc += 2;
                self.cycles += 20;
            }
            0x09 => {}  // ADD HL,BC
            0x0A => {
                // LD A,(BC)
                // Collect address and data
                self.mar = self.bc.get_wide();
                self.mdr = self.read_memory(AddressingMode::ImmediateSixteen);

                // Load bits into A.
                self.a = self.mdr as u8;

                // Increment PC and cycles as appropriate.
                self.pc += 0;
                self.cycles += 8;
            }
            0x0B => { self.bc.set_wide(self.bc.get_wide() - 1); self.pc += 0; self.cycles += 8; }  // DEC BC
            0x0C => { self.inc_reg_8(Registers::C).unwrap(); self.pc += 0; self.cycles += 8; }  // INC C
            0x0D => { self.dec_reg_8(Registers::C).unwrap(); self.pc += 0; self.cycles += 8; }  // DEC C
            0x0E => {
                // LD C,d8
                self.mdr = self.read_memory(AddressingMode::ImmediateEight);
                self.bc.set_low_bin(self.mdr as u8);
                self.pc += 1;
                self.cycles += 8;
            }
            0x0F => { self.rotate_a(RotateDirection::Right, false); self.pc += 0; self.cycles += 4; }  // RRCA

            0x10 => {}
            0x11 => {
                // LD DE,d16
                self.mdr = self.read_memory(AddressingMode::ImmediateSixteen);
                println!("PC: {}", self.pc);
                self.ld_reg_pair(RegisterPairs::DE);
                println!("PC: {}", self.pc);
            }
            0x12 => {
                self.mar = self.bc.get_wide();
                self.memory[self.mar as usize] = self.a as u16;
                self.pc += 0;
                self.cycles += 8;
            }
            0x13 => {}
            0x14 => { self.inc_reg_8(Registers::D).unwrap(); self.pc += 0; self.cycles += 8; } // INC D
            0x15 => { self.dec_reg_8(Registers::D).unwrap(); self.pc += 0; self.cycles += 8; } //  DEC D
            0x16 => {
                // LD D, d8
                self.mdr = self.read_memory(AddressingMode::ImmediateEight);
                self.de.set_high_bin(self.mdr as u8);
                self.pc += 1;
                self.cycles += 8;
            }
            0x17 => {
                // RLA

            }
            0x18 => {}
            0x19 => {}
            0x1A => {}
            0x1B => { self.bc.set_wide(self.de.get_wide() - 1); self.pc += 0; self.cycles += 8; } // DEC DE
            0x1C => { self.inc_reg_8(Registers::E).unwrap(); self.pc += 0; self.cycles += 8; } // INC E
            0x1D => { self.dec_reg_8(Registers::E).unwrap(); self.pc += 0; self.cycles += 8;} // DEC E
            0x1E => {
                // LD E, d8
                self.mdr = self.read_memory(AddressingMode::ImmediateEight);
                self.de.set_low_bin(self.mdr as u8);
                self.pc += 1;
                self.cycles += 8;
            }
            0x1F => {}

            0x20 => {}
            0x21 => {
                // LD HL,d16
                self.mdr = self.read_memory(AddressingMode::ImmediateSixteen);
                self.ld_reg_pair(RegisterPairs::HL);
            }
            0x22 => {
                self.mar = self.de.get_wide();
                self.memory[self.mar as usize] = self.a as u16;
                self.pc += 0;
                self.cycles += 8;
            }
            0x23 => {}
            0x24 => {}
            0x25 => {}
            0x26 => {}
            0x27 => {}
            0x28 => {}
            0x29 => {}
            0x2A => {}
            0x2B => { self.hl.set_wide(self.hl.get_wide() - 1); self.pc += 0; self.cycles += 8; } // DEC HL
            0x2C => { self.inc_reg_8(Registers::L).unwrap(); self.pc += 0; self.cycles += 8;} // INC L
            0x2D => { self.dec_reg_8(Registers::L).unwrap(); self.pc += 0; self.cycles += 8; } // DEC L
            0x2E => {
                // LD L, d8
                self.mdr = self.read_memory(AddressingMode::ImmediateEight);
                self.hl.set_high_bin(self.mdr as u8);
                self.pc += 1;
                self.cycles += 8;
            }
            0x2F => {}

            0x30 => {}
            0x31 => {
                // LD HL,d16
                self.mdr = self.read_memory(AddressingMode::ImmediateSixteen);
                self.sp = self.mdr;
                self.pc += 2;
                self.cycles += 12;
            }
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
            AddressingMode::SignedEight => {
                // This will take the signed operand in memory, and convert it from TC to an unsigned 16 bit integer.
                from_signed_byte(self.memory[self.pc as usize] as u8) as u16
            }
            AddressingMode::RegisterPairDirect(reg) => {
                self.memory[reg.get_wide() as usize]
            }
            AddressingMode::RegisterDirect(reg, is_high) => {
                // We will read from the memory address in either the high or low byte of the RegPair
                if is_high {
                    // println!("Reg value is {:#04x}", reg.get_high());
                    self.memory[reg.get_high() as usize]
                } else {
                    // println!("Reg value is {:#04x}", reg.get_low());
                    self.memory[reg.get_low() as usize]
                }
            }
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

    /// Increment the value stored in one half-register (i.e. a single register).
    /// It will increment the BCD value inside this register and hence the result will be stored as BCD too.
    fn inc_reg_8(&mut self, reg: Registers) -> Result<u8, OpcodeError> {
        // Match the correct RegisterPair and store the correct reference.
        let mut high = false;
        let regtarg: Option<&mut RegPair> = match reg {
            Registers::B => { high = true; Some(&mut self.bc) }
            Registers::C => { high = false; Some(&mut self.bc) }
            Registers::D => { high = true; Some(&mut self.de) }
            Registers::E => { high = false; Some(&mut self.de) }
            Registers::H => { high = true; Some(&mut self.hl) }
            Registers::L => { high = false; Some(&mut self.hl) }
            _ => { None }
        };

        // Check if this was used appropriately.
        return if regtarg.is_none() {
            Err(OpcodeError::new("Attempted to increment the A or SP register.".to_string(), self.ir as u8))
        } else {
            // Create our local register target from within a register pair.
            let target = regtarg.unwrap();

            // Create a local copy of the old value.
            let oldval = if high {
                RegPair::bcd_to_decimal(target.get_high())
            } else {
                RegPair::bcd_to_decimal(target.get_low())
            };

            // Adjust the correct register from within a register pair.
            if high {
                target.set_high_bcd(oldval + 1).unwrap();
                // Toggle zero flag as appropriate.
                self.flags.zero = RegPair::bcd_to_decimal(target.get_high()) == 0;
            } else {
                target.set_low_bcd(oldval + 1).unwrap();
                // Toggle zero flag as appropriate.
                self.flags.zero = RegPair::bcd_to_decimal(target.get_low()) == 0;
            }
            // This instruction always sets the subtraction flag to false;
            self.flags.subtraction = false;
            // Toggle carry flag as appropriate.
            // We need to toggle a half carry if the 0th bit of the oldval is set yet incrementing resulted in an overall zero.
            // This is the only way we would have caused a carry on the third bit.
            self.flags.half_carry = ((oldval & 0b0001) == 0b0001) && self.flags.zero;
            Ok(oldval + 1)
        }
    }

    /// Decrement a value in an r8 register. This is functionally very similar to inc_reg_8 except
    /// we use a negative number as an operand to increment the value by.
    fn dec_reg_8(&mut self, reg:Registers) -> Result<u8, OpcodeError> {
        // Match the correct RegisterPair and store the correct reference.
        let mut high = false;
        let regtarg: Option<&mut RegPair> = match reg {
            Registers::B => { high = true; Some(&mut self.bc) }
            Registers::C => { high = false; Some(&mut self.bc) }
            Registers::D => { high = true; Some(&mut self.de) }
            Registers::E => { high = false; Some(&mut self.de) }
            Registers::H => { high = true; Some(&mut self.hl) }
            Registers::L => { high = false; Some(&mut self.hl) }
            _ => { None }
        };

        // Check if this was used appropriately.
        return if regtarg.is_none() {
            Err(OpcodeError::new("Attempted to increment the A or SP register.".to_string(), self.ir as u8))
        } else {
            // Create our local register target from within a register pair.
            let target = regtarg.unwrap();

            // Create a local copy of the old value.
            let oldval = if high {
                RegPair::bcd_to_decimal(target.get_high())
            } else {
                RegPair::bcd_to_decimal(target.get_low())
            };

            let old_wrapped = Wrapping(oldval);
            let operand_wrapped = Wrapping(0b1111_1111);
            let decr = if old_wrapped.0 != 0 {
                get_magnitude_tc(old_wrapped.add(operand_wrapped).0 as i8)
            } else {
                0xF // Sneaky shortcut.
            };

            // println!("The decreased value will be {}, aka {:#2b}, where the old value was {}", decr, decr, oldval);


            // Adjust the correct register from within a register pair.
            if high {
                target.set_high_bcd(decr).unwrap();
                // Toggle zero flag as appropriate.
                self.flags.zero = RegPair::bcd_to_decimal(target.get_high()) == 0;
            } else {
                target.set_low_bcd(decr).unwrap();
                // Toggle zero flag as appropriate.
                self.flags.zero = RegPair::bcd_to_decimal(target.get_low()) == 0;
            }
            // Set the subtraction flag appropriately.
            self.flags.subtraction = true;

            // Toggle carry flag as appropriate.
            // This checks if we had a carry from bit 3 to bit 4.
            self.flags.half_carry = (decr & 0x10) == 0x10;
            Ok(decr)
        }
    }

    // fn rotate_r8(&mut self, dir: RotateDirection, reg: Registers, throughCarry: bool) {
    //     // Match on the register
    // }

    fn rotate_a(&mut self, dir: RotateDirection, through_carry: bool) {
        match dir {
            RotateDirection::Left => {
                // Check if we must also rotate through carry.
                if !through_carry {
                    // Toggle the carry flag to match bit 7 prior to a rotate.
                    self.flags.carry = (self.a & 0b1000_0000) == 0b1000_0000;
                    self.a = self.a << 1;
                    if self.flags.carry {
                        self.a = self.a | 0b0000_0001;
                    };
                }
            }
            RotateDirection::Right => {
                if !through_carry {
                    // Toggle the carry flag to match bit 7 prior to a rotate.
                    self.flags.carry = (self.a & 0b0000_0001) == 0b0000_0001;
                    self.a = self.a >> 1;
                    if self.flags.carry {
                        self.a = self.a | 0b1000_0000;
                    };
                }
            }
        }
    }

    fn write_bytes(&mut self, bytes: &[u16], index: usize) -> Result<()>{
        if (index + bytes.len()) > 65536 {
            return Err(anyhow!(MemoryError("BIG NUMBER")));
        }

        for i in 0..bytes.len() {
            self.memory[i + index] = bytes[i];
        }
        Ok(())
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

#[derive(Debug, Error)]
#[error("Attempted to write to invalid memory index {0}")]
pub struct MemoryError(pub &'static str);

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

    #[test]
    fn write_bytes() {
        let mut cpu = CPU::new();
        cpu.write_bytes(&[0xA, 0xB, 0xC, 0xD], 0).unwrap();
        assert_eq!(cpu.memory[0..4], [0xA, 0xB, 0xC, 0xD]);
        cpu.write_bytes(&[0xA, 0xB, 0xC, 0xD, 0xE], 1).unwrap();
        assert_eq!(cpu.memory[1..6], [0xA, 0xB, 0xC, 0xD, 0xE]);
    }
}

#[cfg(test)]
mod opcodes {
    use crate::components::dmg_cpu::CPU;
    use crate::components::register::RegPair;

    #[test]
    fn ld_r16_d16() {

    }

    #[test]
    fn ld_d16_r8() {

    }

    #[test]
    fn inc_r16() {
        let mut cpu = CPU::new();
        cpu.memory[0] = 0x03;
        cpu.memory[1] = 0x03;
        cpu.memory[2] = 0x03;
        cpu.memory[3] = 0x03;
        cpu.cycle();
        assert_eq!(1, cpu.bc.get_wide());
        cpu.cycle();
        assert_eq!(2, cpu.bc.get_wide());
        cpu.cycle();
        assert_eq!(3, cpu.bc.get_wide());
        cpu.cycle();
        assert_eq!(4, cpu.bc.get_wide());
        for i in 4..10 {
            cpu.memory[i] = 0x0B;
        }
        cpu.cycle();
        assert_eq!(3, cpu.bc.get_wide());
        cpu.cycle();
        assert_eq!(2, cpu.bc.get_wide());
        cpu.cycle();
        assert_eq!(1, cpu.bc.get_wide());
        cpu.cycle();
        assert_eq!(0, cpu.bc.get_wide());
        cpu.cycle();
        assert_eq!(0xFF, cpu.bc.get_wide());
    }

    #[test]
    fn load_r8_d8() {
        let mut cpu = CPU::new();
        cpu.memory[0] = 0x06; // LD B, d8
        cpu.memory[1] = 0xAB;
        cpu.memory[2] = 0x06;
        cpu.memory[3] = 0x01;
        cpu.memory[4] = 0x06;
        cpu.memory[5] = 0x00;
        cpu.cycle();
        assert_eq!(0xAB, cpu.bc.get_high());
        cpu.cycle();
        assert_eq!(0x01, cpu.bc.get_high());
        cpu.cycle();
        assert_eq!(0x00, cpu.bc.get_high());
    }

    #[test]
    fn rxca() {
        let mut cpu = CPU::new();
        // Todo: Complete loading instructions so that A can be loaded.
    }

    #[test]
    fn ld_a16_sp() {
        let mut cpu = CPU::new();
        cpu.sp = 0xABCD;
        cpu.memory[0] = 0x08;
        cpu.memory[1] = 0x04;
        cpu.memory[2] = 0x00; // Sets the address to 0x0004.
        cpu.cycle(); // We expect m[0x0004]: AB; m[0x0005]: CD.
        assert_eq!(0xCD, cpu.memory[0x0004]);
        assert_eq!(0xAB, cpu.memory[0x0005]);
    }
}

#[cfg(test)]
/// Instruction tests, grouped by specific categories of opcodes.
mod opcode_category_tests {
    use crate::components::dmg_cpu::CPU;
    use crate::components::register::RegPair;

    #[test]
    /// Opcode 0x00
    fn nop() {
        let mut cpu = CPU::new();
        cpu.memory[0] = 0x00;
        cpu.cycle();
        assert_eq!(cpu.pc, 1);
        assert_eq!(cpu.bc.get_wide(), 0);
        assert_eq!(cpu.de.get_wide(), 0);
        assert_eq!(cpu.hl.get_wide(), 0);
        assert_eq!(cpu.sp, 0);
    }

    #[test]
    /// Opcodes tested:
    /// - 0x01, 0x11, 0x21
    ///
    fn ld_r16_d16() {
        let mut cpu = CPU::new();
        cpu.memory[0] = 0x01; // LD BC, d16. Will spell out 0xABCD
        cpu.memory[1] = 0xCD; // Lower bytes of 0xABCD
        cpu.memory[2] = 0xAB; // Lower bytes of 0xABCD
        cpu.cycle();
        assert_eq!(cpu.bc.get_wide(), 0xABCD);
        cpu.memory[3] = 0x11; // LD DE, d16
        cpu.memory[4] = 0xEF;
        cpu.memory[5] = 0xCD;
        cpu.cycle();
        assert_eq!(cpu.de.get_wide(), 0xCDEF);
        cpu.memory[6] = 0x21; // LD HL, d16
        cpu.memory[7] = 0xBB;
        cpu.memory[8] = 0xAA;
        cpu.cycle();
        assert_eq!(cpu.hl.get_wide(), 0xAABB);
        cpu.write_bytes(&[0x31, 0xBB, 0xAA], 9).unwrap(); // LD SP d16
        cpu.cycle();
        assert_eq!(cpu.sp, 0xAABB);
    }

    #[test]
    fn ld_r16_a() {
        let mut cpu = CPU::new();
        cpu.a = 0xAB;
        // Load addresses into BC and DE, then store the accumulator value
        // into these memory addresses.
        // BC = 0x000A, DE = 0x000C, HL = 0x000F
        let instr = &[0x01, 0x0A, 0x00, 0x11, 0x0C, 0x00, 0x02, 0x12];
        cpu.write_bytes(instr, 1).unwrap();
        cpu.cycle(); // LD BC, d16
        cpu.cycle(); // LD DE, d16
        cpu.cycle(); // LD (BC), A
        println!("IR: {}", cpu.ir);
        assert_eq!(cpu.memory[0x000A], 0xAB);
        cpu.cycle(); // LD (DE), A
        assert_eq!(cpu.memory[0x000A], 0xAB);
        assert_eq!(cpu.memory[0x000C], 0xAB);
    }

    #[test]
    fn inc_r16() {}

    #[test]
    fn inc_r8() {}

    #[test]
    fn dec_r8() {}

    #[test]
    fn ld_r8_d8() {
        let mut cpu = CPU::new();
        cpu.memory[0] = 0x06; // LD B, d8
        cpu.memory[1] = 0xAB;
        cpu.memory[2] = 0x06;
        cpu.memory[3] = 0x01;
        cpu.memory[4] = 0x06;
        cpu.memory[5] = 0x00;
        cpu.cycle();
        assert_eq!(0xAB, cpu.bc.get_high());
        cpu.cycle();
        assert_eq!(0x01, cpu.bc.get_high());
        cpu.cycle();
        assert_eq!(0x00, cpu.bc.get_high());
    }

    #[test]
    fn rlca() {}

    #[test]
    fn ld_a16_sp() {}

    #[test]
    fn add_r16_r16() {}

    #[test]
    fn rrca() {}

    // 1x
    #[test]
    fn stop() {}

    #[test]
    fn rla() {}

    #[test]
    fn jr_s8() {}

    #[test]
    fn rra() {}

    // 2x
    #[test]
    fn jr_b_s8() {}

    #[test]
    fn ld_ri_r8() {}

    #[test]
    fn daa() {}

    #[test]
    fn ld_r8_ri16() {}

    #[test]
    fn cpl() {}

    // 3x
    #[test]
    fn ld_sp_d16() {}

    #[test]
    fn inc_rd16() {}

    #[test]
    fn dec_rd16() {}

    #[test]
    fn ld_rd16_d8() {}

    #[test]
    fn scf() {}

    #[test]
    fn ccf() {}

    // 4x
    #[test]
    fn ld_r8_r8() {}

    #[test]
    fn ld_r8_rd16() {}

    // 5x
    #[test]
    fn halt() {}

    // 6x
    #[test]
    fn add_r8_r8() {}

    #[test]
    fn add_r8_rd16() {}

    #[test]
    fn adc_r8_r8() {}

    #[test]
    fn adc_r8_rd16() {}

    // 7x
    #[test]
    fn sub_r8() {}

    #[test]
    fn sub_rd16() {}

    #[test]
    fn subc_r8_r8() {}

    #[test]
    fn subc_r8_rd16() {}

    // 8x
    #[test]
    fn and_r8() {}

    #[test]
    fn and_rd16() {}

    #[test]
    fn xor_r8() {}

    #[test]
    fn xor_rd16() {}

    // 9x
    #[test]
    fn or_r8() {}

    #[test]
    fn or_rd16() {}

    #[test]
    fn cp_r8() {}

    #[test]
    fn cp_rd16() {}

    // Ax
    #[test]
    fn ret_b() {}

    #[test]
    fn pop_r16() {}

    #[test]
    fn jp_b_a16() {}

    #[test]
    fn jp_a16() {}

    #[test]
    fn call_b_a16() {}

    #[test]
    fn push_r16() {}

    #[test]
    fn add_r8_d8() {}

    #[test]
    fn rst() {}

    #[test]
    fn ret() {}

    #[test]
    fn call_a16() {}

    #[test]
    fn adc_r8_d8() {}

    // Bx
    #[test]
    fn sub_d8() {}

    #[test]
    fn reti() {}

    #[test]
    fn sbc_r8_d8() {}

    // Cx
    #[test]
    fn ld_a8_r8() {}

    #[test]
    fn ld_rd8_r8() {}

    #[test]
    fn and_d8() {}

    #[test]
    fn add_sp_s8() {}

    #[test]
    fn ld_a16_r8() {}

    #[test]
    fn xor_d8() {}

    // Dx
    #[test]
    fn ld_r8_a8() {}

    #[test]
    fn di() {}

    #[test]
    fn or_d8() {}

    #[test]
    fn ld_r16_sp() {}

    #[test]
    fn ld_sp_hl() {}

    #[test]
    fn ld_r8_a16() {}

    #[test]
    fn ei() {}

    #[test]
    fn cp_d8() {}
}
