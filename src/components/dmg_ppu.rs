#[allow(dead_code)]

use ux::{u1, u2};
use std::collections::HashMap;
use crate::components::graphics_components::Tile;

enum Mode {
    DMG,
    CGB,
}

/// This determines which background map the the Window / Background should use for rendering.
enum WindowBGArea {
    /// Refers to RESET bit.
    Base = 0x9800,
    /// Refers to SET bit.
    Offset = 0x9C00,
}

enum AddressingMode {
    // TODO: Understand why we use these blocks, and hence what enum values to assign.
    Unsigned,
    Signed
}

enum ObjSize {
    Square,
    Double,
}

/// # Game Boy PPU
/// The PPU is used to organise the various I/O devices which are
/// responsible for driving video output on the Game Boy. These are
/// typically accessed via memory-mapped I/O, but this struct allows
/// for a high-level API which can be called by the CPU or any other
/// user, such as for debugging purposes.
/// ## PPU Components
/// - Sprite attribute table (OAM)
/// - LCD control
/// - LCD status
/// - LCD position and scrolling
/// - Palettes
/// - Pixel FIFO
pub struct PPU<'a> {
    mode: Mode,
    oam: OAM<'a>,
    //LCD Control - enables
    /// Determines if the LCD and PPU are on/active.
    /// Turning off allows immediate and full access to VRAM, OAM, etc.
    lcd_enable: bool,
    /// Displays the window or not. Can be overridden by `bg_window_priority`
    window_enable: bool,
    /// Toggles rendering of sprites on screen; can be toggled mid-frame.
    obj_enable: bool,
    /// In DMG mode, when reset, both the window and the background become blank.
    /// In CGB mode, when reset, sprites will be displayed in front of the BG/window.
    bg_window_priority: bool,
    // LCD Control - data areas
    bg_window_tile_area: AddressingMode,
    window_tile_area: WindowBGArea,
    bg_tile_area: WindowBGArea,
    obj_size: ObjSize,
}

/// # OAM
/// The OAM is used to organise sprites and their attributes within VRAM.
/// Within the Game Boy, only 40 sprites may be on screen at any one point in time. There are 40
/// entries in the OAM, each with a 4 bytes to represent their attributes.
pub struct OAM<'a> {
    /// Hashmap to store the sprites from ROM.
    rom_sprites: HashMap<u8, TableEntry<'a>>
}

/// Each entry in the OAM contains a set of attributes.
pub struct TableEntry<'a> {
    y_pos: u8,
    x_pos: u8,
    index: u8,
    over_obj: bool,
    y_flip: bool,
    x_flip: bool,
    palette: ux::u1, // Non-CGB Mode only
    vram_bank: ux::u1,
    cgb_palette: ux::u2,
    tile: Tile<'a>
}