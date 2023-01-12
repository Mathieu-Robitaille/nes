use crate::olc_pixel_game_engine as olc;

use crate::bus::Bus;
use crate::cartridge::{load_cart, Rom};
use crate::cpu::Cpu6502;
use crate::debug::draw_debug;
use crate::disassembler::disassemble_rom;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Nes {
    pub cpu: Cpu6502,
    pub decoded_rom: HashMap<u16, String>,
}

impl Nes {
    pub fn new() -> Self {
        match load_cart(Rom::CPUTest) {
            Ok(cart) => {
                let cart_rc = Rc::new(RefCell::new(cart));
                let bus = Bus::new(cart_rc.clone());
                let decoded_rom = disassemble_rom(0x0000, 0xFFFF, cart_rc.clone());
                let mut cpu = Cpu6502::new(bus);
                cpu.reset(Some(0xC000));
                return Self {
                    cpu,
                    decoded_rom,
                };
            }
            Err(x) => {
                println!("{:?}", x);
                panic!()
            }
        }
    }
}

impl olc::Application for Nes {
    fn on_user_create(&mut self) -> Result<(), olc::Error> {
        // Mirrors `olcPixelGameEngine::onUserCreate`. Your code goes here.

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
                if let Some(x) = self.cpu.clock() {
                    println!("{}", x);
                }
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
