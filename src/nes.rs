use crate::olc_pixel_game_engine as olc;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::cartridge::{Rom, load_cart};
use crate::disassembler::disassemble_rom;
use crate::debug::draw_debug;
use crate::cpu::Cpu6502;
use crate::bus::Bus;

pub struct Nes {
    pub cpu: Cpu6502,
    pub decoded_rom: HashMap<u16, String>
}

impl Nes {
    pub fn new() -> Self {
        if let Ok(cart) = load_cart(Rom::CPUTest) {
            let cart_rc = Rc::new(RefCell::new(cart));
            let bus = Bus::new(cart_rc.clone());
            let decoded_rom = disassemble_rom(0x0000, 0xFFFF, cart_rc.clone());
            return Self {
                cpu: Cpu6502::new(bus),
                decoded_rom
            }
        } else {
            panic!("Heck!");
        }
    }
}

impl olc::Application for Nes {
    fn on_user_create(&mut self) -> Result<(), olc::Error> {
        // Mirrors `olcPixelGameEngine::onUserCreate`. Your code goes here.
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