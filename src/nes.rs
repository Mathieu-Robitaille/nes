use crate::olc_pixel_game_engine as olc;

use std::collections::HashMap;
use crate::cartridge::{Cartridge, TestCart, get_test_bytes};
use crate::disassembler::{HumanInstruction, decode_rom};
use crate::debug::draw_debug;
use crate::cpu::Cpu6502;
pub struct Nes {
    pub cpu: Cpu6502,
    pub decoded_rom: Option<HashMap<u16, HumanInstruction>>
}

impl olc::Application for Nes {
    fn on_user_create(&mut self) -> Result<(), olc::Error> {
        // Mirrors `olcPixelGameEngine::onUserCreate`. Your code goes here.

        if let Ok(cart) = get_test_bytes(TestCart::BASIC) {
            self.cpu.bus.load_program(&cart);
            self.decoded_rom = Some(decode_rom(&cart));
        } else {
            println!("Heck.");
        }
        self.cpu.reset();
        // self.cpu.program_counter = 0x0400;
        Ok(())
    }

    fn on_user_update(&mut self, _elapsed_time: f32) -> Result<(), olc::Error> {
        // Mirrors `olcPixelGameEngine::onUserUpdate`. Your code goes here.

        // Clears screen and sets black colour.
        olc::clear(olc::BLACK);
        // Prints the string starting at the position (40, 40) and using white colour.
        
        if olc::get_key(olc::Key::SPACE).pressed {
            loop {
                self.cpu.clock();
                if self.cpu.complete() {
                    break; 
                }
            }
        }
        
        draw_debug(self)?;
        Ok(())
    }

    fn on_user_destroy(&mut self) -> Result<(), olc::Error> {
        // Mirrors `olcPixelGameEngine::onUserDestroy`. Your code goes here.
        Ok(())
    }
}