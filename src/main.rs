#![allow(unused)]
extern crate olc_pixel_game_engine;

use crate::olc_pixel_game_engine as olc;

mod bus;
mod cartridge;
mod cpu;
mod debug;
mod disassembler;
mod disk;
mod instructions;
mod mapper;
mod nes;
mod ppu;


use nes::Nes;

fn main() {
    let mut nes: Nes = Nes::new();
    // Launches the program in 200x100 "pixels" screen, where each "pixel" is 4x4 pixel square,
    // and starts the main game loop.
    olc::start("Hello, World!", &mut nes, 800, 400, 1, 1).unwrap();
}
