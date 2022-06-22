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
pub struct PPU {

}