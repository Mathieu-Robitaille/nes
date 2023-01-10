#![allow(unused)]
extern crate olc_pixel_game_engine;

use crate::olc_pixel_game_engine as olc;


mod bus;
mod cpu;
mod instructions;
mod disk;
mod cartridge;
mod disassembler;
mod debug;
mod nes;

use bus::Bus;
use nes::Nes;
use cpu::Cpu6502;

fn main() {
    let mut bus: Bus = Bus::new([0x00; 64 * 1024]);
    let mut nes: Nes = Nes { cpu: Cpu6502::new(bus), decoded_rom: None };
    // Launches the program in 200x100 "pixels" screen, where each "pixel" is 4x4 pixel square,
    // and starts the main game loop.
    olc::start("Hello, World!", &mut nes, 800, 400, 1, 1).unwrap();
}



