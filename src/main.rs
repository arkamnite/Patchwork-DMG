mod components;

#[macro_use]
extern crate derive_builder;

use derive_builder::Builder;

use sdl2::pixels::Palette;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::surface::Surface;
use sdl2::video::{Window, WindowContext};
use std::time::Duration;
use ux::i2;
use sdl2::render::{Texture, Canvas, TextureCreator, WindowCanvas};

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Patchwork DMG",  160, 144)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;

    // Creating a test palette.
    let mut palette = GBPalette::new(C1, C2, C3, C4);

    let mut pal = Palette::with_colors(&[C1, C2, C3, C4]).unwrap();

    'running: loop {
        i = (i + 1) % 255;
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. }  => {
                    break 'running
                },
                _ => {}
            }
        }

        canvas.present();
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    println!("Hello, world!");
}

const C1: Color = Color::RGB(255, 255, 255);
const C2: Color = Color::RGB(190, 190, 190);
const C3: Color = Color::RGB(130, 130, 130);
const C4: Color = Color::RGB(82, 82, 82);

/// Defines four specific colours which will be used by a Tile to represent each of the four possible
/// shades that a pixel can occupy.
// #[derive(Builder)]
#[derive(Clone)]
struct GBPalette {
    // #[builder(default = C1)]
    pub col1: Color,
    // #[builder(default = C2)]
    pub col2: Color,
    // #[builder(default = C3)]
    pub col3: Color,
    // #[builder(default = C4)]
    pub col4: Color,
}

impl GBPalette {
    fn new(col1: Color, col2: Color, col3: Color, col4: Color) -> GBPalette {
        GBPalette {
            col1,
            col2,
            col3,
            col4
        }
    }

    fn col_id(&self, id: ux::i2) -> Color {
        let cid = i32::from(id);
        match cid {
            0b00 => {self.col1},
            0b01 => {self.col2},
            0b10 => {self.col3},
            0b11 => {self.col4},
            _ => {self.col1}
        }
    }
}

/// A single tile occupies 16 bytes, where each line is represented by 2 bytes.
/// Hence there are 8 lines.
/// In each line, the first byte specifies the LSB of the color ID of each pixel, whereas the second
/// byte specifies the MSB.
struct Tile<'a> {
    // pub palette: Palette,

    pub bytes: [u8; 16],
    // pub surface: Surface<'a>,
    c1: Color,
    c2: Color,
    c3: Color,
    c4: Color,
    pub surface: Option<&'a Surface<'a>>, // The texture may not be initialised yet.
    pixels: Option<[u32; 64]>

}

// Tile tests (8x8 pixels).
impl<'a> Tile<'a> {
    fn new(bytes: [u8; 16]) -> Tile<'a> {
        Tile {
            bytes,
            c1: Color::RGB(255, 255, 255),
            c2: Color::RGB(190, 190, 190),
            c3: Color::RGB(130, 130, 130),
            c4: Color::RGB(82, 82, 82),
            surface: None,
            pixels: None
        }
    }

    pub fn create_texture(&mut self, surface: &Surface) {
        // Create a texture
        self.surface = Some(&surface);
        // Create the pixel buffer
        // self.pixels = Some([u32; 64])
        // Modify our pixels accordingly

        // Update the texture with the created pixel buffer
    }

    // fn get_colour_index(id: ux::i2) -> i32 {
    //     cid = i32::from(id);
    //     cid
    // }

}

struct FrameBuffer {

}