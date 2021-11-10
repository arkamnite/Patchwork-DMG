use crate::components::register::RegPair;

pub struct CPU {
    a: u8,
    sp: u16,
    pc: u16,
    bc: RegPair,
    de: RegPair,
    hl: RegPair,
    flags: Flags,
    lcd_reg: LCDReg,
    memory_64kib: [u16; 65536],
    cycles: u32,
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

impl CPU {

}