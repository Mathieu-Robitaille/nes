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
    emulation_run: bool,
    run_for_frame: bool,
    debug_text: bool,
    show_debug: bool,
    system_clock: usize,
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
                // cpu.reset(None);
                return Self {
                    cpu,
                    decoded_rom,
                    emulation_run: false,
                    debug_text: false,
                    run_for_frame: false,
                    show_debug: true,
                    system_clock: 0,
                };
            }
            Err(x) => {
                println!("{:?}", x);
                panic!()
            }
        }
    }
    fn debug_print(&self, text: String) {
        if self.debug_text {
            println!("{}", text);
        }
    }
    fn clock(&mut self) {
        for _ in 0..3 {
            self.cpu.bus.ppu.clock();
        }

        if let Some(x) = self.cpu.clock() {
            self.debug_print(x);
        }

        if self.cpu.bus.ppu.nmi {
            self.cpu.bus.ppu.nmi = false;
            self.cpu.nmi();
        }

        self.system_clock += 1;
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

        if olc::get_key(olc::Key::Y).pressed {
            println!("palette: {:?}", self.cpu.bus.ppu.palette);
        }

        if olc::get_key(olc::Key::E).pressed {
            self.cpu.bus.ppu.debug = !self.cpu.bus.ppu.debug;
        }

        if self.show_debug {
            draw_debug(self)?;
        } else if self.cpu.instruction_count % 1000 == 0 {
            println!("{:?}", self.cpu.instruction_count);
        }

        // Prints the string starting at the position (40, 40) and using white colour.
        if olc::get_key(olc::Key::Q).pressed {
            self.emulation_run = !self.emulation_run;
        }
        if olc::get_key(olc::Key::W).pressed {
            self.debug_text = !self.debug_text;
        }
        if olc::get_key(olc::Key::D).pressed {
            self.show_debug = !self.show_debug;
        }

        // Run non stop
        if self.emulation_run || (self.run_for_frame && !self.cpu.bus.ppu.frame_complete) {
            self.clock();
            if self.cpu.bus.ppu.frame_complete {
                self.run_for_frame = !self.run_for_frame
            }
        } else {
            // Run for one frame
            if olc::get_key(olc::Key::F).pressed {
                self.run_for_frame = !self.run_for_frame
            }
            // Run one instruction
            else if olc::get_key(olc::Key::SPACE).pressed {
                loop {
                    self.clock();
                    if self.cpu.complete() {
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    fn on_user_destroy(&mut self) -> Result<(), olc::Error> {
        // Mirrors `olcPixelGameEngine::onUserDestroy`. Your code goes here.
        Ok(())
    }
}
