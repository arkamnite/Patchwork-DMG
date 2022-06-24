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
use patchwork_dmg::components::graphics_components::{GBPalette, Tile};

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