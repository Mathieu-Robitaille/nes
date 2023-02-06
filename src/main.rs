// #![allow(unused)]

mod audio;
mod bus;
mod cartridge;
mod consts;
mod cpu;
mod debug;
mod disassembler;
mod emulator;
mod instructions;
mod mapper;
mod nes;
mod ppu;
mod renderer;

use glium::{backend::Facade};
use nes::Nes;
use renderer::*;

fn main() {
    let main_nes: Nes = Nes::new();
    let mut system = init();
    let mut emulation_state = emulator::EmulationState::new();
    emulation_state
        .register_textures(system.display.get_context(), system.renderer.textures())
        .expect("Failed to register textures.");

    system.main_loop(main_nes, emulation_state, move |_, nes, state, ui| {
        debug::draw_debug(nes, state, ui);
    });
}

struct NesEmulator {
    nes: Nes,
    controls: emulator::EmulationControls,
    state: emulator::EmulationState,
}
