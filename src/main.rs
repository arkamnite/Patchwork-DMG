mod components;

#[macro_use]
extern crate derive_builder;

use derive_builder::Builder;

use sdl2::pixels::Palette;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::rect::Point;
use sdl2::keyboard::Keycode;
use sdl2::surface::Surface;
use sdl2::video::{Window, WindowContext};
use std::time::Duration;
use ux::{i2, u2};
use sdl2::render::{Texture, Canvas, TextureCreator, WindowCanvas};
use sdl2::Sdl;

fn main() {
    let scale = 6;
    let framerate = 75;
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Patchwork DMG",  160 * scale, 144 * scale)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_scale(scale as f32, scale as f32).unwrap();

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;

    // Creating a test palette.
    // let mut pal = Palette::with_colors(&[C1, C2, C3, C4]).unwrap();
    let mut pal = GBPalette::new(C1, C2, C3, C4);
    let mut pal2 = GBPalette::new(C1, C7, C6, C5);
    let mut tile = Tile::new(&pal, [0x3C, 0x7E, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x7E, 0x5E, 0x7E, 0x0A, 0x7C, 0x56, 0x38, 0x7C]);
    let mut tile2 = Tile::new(&pal2, [0x3C, 0x7E, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x7E, 0x5E, 0x7E, 0x0A, 0x7C, 0x56, 0x38, 0x7C]);
    let mut tile3 = Tile::new(&pal, [0xFF, 0x00, 0x7E, 0xFF, 0x85, 0x81, 0x89, 0x83, 0x93, 0x85, 0xA5, 0x8B, 0xC9, 0x97, 0x7E, 0xFF]);
    tile.paint(Point::new(50, 50), &mut canvas);
    'running: loop {
        i = (i + 1) % (160 as i32);
        tile.paint(Point::new(0 + i, 50), &mut canvas);
        tile2.paint(Point::new(0 + i, i), &mut canvas);
        for i in 0..21 {
            tile3.paint(Point::new(8 * i, 0), &mut canvas);
        }
        canvas.set_draw_color(Color::RGB(255, 255, 255));
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
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / framerate));
        canvas.clear();
    }

    println!("Hello, world!");
}

const C1: Color = Color::RGB(255, 255, 255);
const C2: Color = Color::RGB(190, 190, 190);
const C3: Color = Color::RGB(130, 130, 130);
const C4: Color = Color::RGB(82, 82, 82);
const C5: Color = Color::RGB(181, 170, 140);
const C6: Color = Color::RGB(130, 130, 130);
const C7: Color = Color::RGB(255, 0, 0);

/// A GBPalette represents four colours a single pixel may occupy.
/// For DMG units, this is likely to be the same palette for all tiles. For GBC units however,
/// multiple palettes can be used throughout the program lifecycle. This allows GBC units to emulate
/// DMG games in monochrome, at a software level.
struct GBPalette {
    pub col1: Color,
    pub col2: Color,
    pub col3: Color,
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

struct FrameBuffer<'a> {
    /// Contains a list of all tiles and their origin positions.
    tiles: Vec<(&'a Tile<'a>, sdl2::rect::Point)>,
    /// The canvas this Tile will be drawn onto directly.
    canvas: &'a WindowCanvas,
}

/// A tile is a base graphics unit. It consists of 8x8 pixels that can utilise one of four colours.
/// A Tile requires a reference to a Canvas to be drawn onto, as well as a reference to a GBPalette
/// to determine which colour each of the pixels will occupy.
struct Tile<'a> {
    /// The palette that will be used to colour these pixels.
    palette: &'a GBPalette,
    /// The raw bytes from the ROM.
    bytes: [u8; 16],
    /// The grid of colour indices for the individual pixels
    points: [ux::u2; 64],
}

impl<'a> Tile<'a> {
    fn new(palette: &'a GBPalette, bytes: [u8; 16]) -> Tile<'a> {
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
                canvas.draw_point(pos);
            }
        }
    }
}