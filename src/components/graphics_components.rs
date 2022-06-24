use ux::u2;
use sdl2::pixels::Color;
use sdl2::render::WindowCanvas;

/// A GBPalette represents four colours a single pixel may occupy.
/// For DMG units, this is likely to be the same palette for all tiles. For GBC units however,
/// multiple palettes can be used throughout the program lifecycle. This allows GBC units to emulate
/// DMG games in monochrome, at a software level.
pub struct GBPalette {
    pub col1: Color,
    pub col2: Color,
    pub col3: Color,
    pub col4: Color,
}

impl GBPalette {
    pub fn new(col1: Color, col2: Color, col3: Color, col4: Color) -> GBPalette {
        GBPalette {
            col1,
            col2,
            col3,
            col4
        }
    }

    pub fn col_id(&self, id: ux::u2) -> Color {
        let cid = u32::from(id);
        match cid {
            0b00 => {self.col1},
            0b01 => {self.col2},
            0b10 => {self.col3},
            0b11 => {self.col4},
            _ => {self.col1}
        }
    }
}

/// A tile is a base graphics unit. It consists of 8x8 pixels that can utilise one of four colours.
/// A Tile requires a reference to a Canvas to be drawn onto, as well as a reference to a GBPalette
/// to determine which colour each of the pixels will occupy.
pub struct Tile<'a> {
    // The palette that will be used to colour these pixels.
    palette: &'a GBPalette,
    /// The raw bytes from the ROM.
    bytes: [u8; 16],
    /// The grid of colour indices for the individual pixels
    points: [ux::u2; 64],

}

impl<'a> Tile<'a> {
    pub fn new(palette: &'a GBPalette, bytes: [u8; 16]) -> Tile<'a> {
        let mut points = [u2::new(0); 64];
        // Calculate the colour of each pixel.
        // We read two bytes at a time, as these form pairs.
        for pixel in bytes.chunks(2).enumerate() {
            let (i, x): (usize, &[u8]) = pixel;
            // println!("{}", format!("i = {} : {:#10b}, i+1 = {} : {:#10b}", i, x[0], i+1, x[1]));
            // Construct the 2bpp pixels.
            // Reading each row.
            let row_ms = x[1];
            let row_ls = x[0];
            // Constructing the pixels.
            for j in 0..8 {
                // let msb = ((row_ms << j) >> (7 - j));
                // let lsb = ((row_ls << j) >> (7 - j));
                let msb = (((row_ms) << j) >> (7)) << 1;
                let lsb = ((row_ls) << j) >> (7);
                let bbpp = (msb | lsb);
                // println!("{}, {}", (i * 8)+j, format!("{:#10b}, {}", bbpp, palette.col_id(u2::new(bbpp)).r));
                points[(i * 8) + j] = u2::new(bbpp);
            }
            // println!("row {} has bytes {}", i, points[i..(i+8)]);
        }

        Tile {
            palette,
            bytes,
            points,
        }
    }

    pub fn paint(&self, origin: sdl2::rect::Point, canvas: &mut WindowCanvas) {
        for pixel in self.points.chunks(8).enumerate() {
            let (i, x) = pixel;
            for j in 0..8 {
                canvas.set_draw_color(self.palette.col_id(x[j]));
                let pos = sdl2::rect::Point::new(origin.x() + (j as i32), origin.y() + (i as i32));
                canvas.draw_point(pos).unwrap();
            }
        }
    }
}