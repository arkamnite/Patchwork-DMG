use crate::components::register::RegPair;

pub struct CPU {
    /// The accumulator register.
    a: u8,
    /// The stack pointer.
    sp: u16,
    /// The program counter.
    pc: u16,
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

impl CPU {

    pub fn new() -> Self {
        CPU {
            a: 0,
            sp: 0,
            pc: 0,
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
        self.pc = self.pc + 1;
    }

    /// Given an opcode, this function will decode this using pattern matching and will hence
    fn decode_execute(&self, opcode: u16) {

    }

    /// Will return the correct value from memory that shall be stored in the memory data register.
    pub fn read_memory(&self, mode: AddressingMode) -> u16 {
        let mem_val: u16 = match mode {
            AddressingMode::Implied => {
                // We leave the CPU exactly as it was before.
                // We may later on "automate" the procedure of setting the MDR based on the result
                // of reading memory, and hence it is ideal to return the original value of the MDR.
                self.mdr
            }
            AddressingMode::ImmediateEight => { self.memory[self.pc as usize] as u16}
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