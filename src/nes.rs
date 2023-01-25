use crate::bus::Bus;
use crate::cartridge::{load_cart, Rom};
use crate::consts::{
    emulation_consts::EMU_DEBUG,
    nes_consts::CART,
    ppu_consts,
    screen_consts::{HEIGHT, WIDTH},
};
use crate::cpu::Cpu6502;
use crate::debug::draw_debug;
use crate::disassembler::disassemble_rom;
use crate::ppu::{get_oam_field, set_oam_field, ObjectAttributeEntry};

use glium::{
    backend::Facade,
    texture::{ClientFormat, RawImage2d},
    uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerBehavior},
    Display, Surface, Texture2d,
};
use imgui::{TextureId, Textures};
use imgui_glium_renderer::Texture;
use std::borrow::Cow;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Nes {
    pub cpu: Cpu6502,
    pub decoded_rom: HashMap<u16, String>,
    system_clock: usize,
}

impl Nes {
    pub fn new() -> Self {
        match load_cart(CART) {
            Ok(cart) => {
                let cart_rc = Rc::new(RefCell::new(cart));
                let bus = Bus::new(cart_rc.clone());
                let decoded_rom = disassemble_rom(0x0000, 0xFFFF, cart_rc.clone());
                let mut cpu = Cpu6502::new(bus);
                match CART {
                    Rom::CPUTest => {
                        cpu.reset(Some(0xC000));
                    }
                    _ => {
                        cpu.reset(None);
                    }
                }

                return Self {
                    cpu,
                    decoded_rom,
                    system_clock: 0,
                };
            }
            Err(x) => {
                println!("{:?}", x);
                panic!()
            }
        }
    }

    pub fn clock(&mut self) {
        self.cpu.bus.ppu.clock();

        if self.system_clock % 3 == 0 {
            if self.cpu.bus.dma_transfer {
                if self.cpu.bus.dma_dummy {
                    if self.system_clock % 2 == 1 {
                        self.cpu.bus.dma_dummy = false;
                    }
                } else {
                    if self.system_clock % 2 == 0 {
                        let addr =
                            ((self.cpu.bus.dma_page as u16) << 8) | self.cpu.bus.dma_addr as u16;
                        self.cpu.bus.dma_data = self.cpu.bus.cpu_read(addr, true)
                    } else {
                        set_oam_field(
                            &mut self.cpu.bus.ppu.oam,
                            self.cpu.bus.dma_addr,
                            self.cpu.bus.dma_data,
                        );
                        let (r, _) = self.cpu.bus.dma_addr.overflowing_add(1);
                        self.cpu.bus.dma_addr = r;
                        if self.cpu.bus.dma_addr == 0x00 {
                            self.cpu.bus.dma_transfer = false;
                            self.cpu.bus.dma_dummy = true;
                        }
                    }
                }
            } else {
                if !EMU_DEBUG {
                    if let Some(x) = self.cpu.clock() {
                        println!("{x}")
                    }
                } else {
                    self.cpu.clock();
                }
            }
        }

        if self.cpu.bus.ppu.nmi {
            self.cpu.bus.ppu.nmi = false;
            self.cpu.nmi();
        }
        self.system_clock += 1;
    }

    pub fn get_frame_status(&self) -> bool {
        self.cpu.bus.ppu.frame_complete
    }
    pub fn reset_frame_status(&mut self) {
        self.cpu.bus.ppu.frame_complete = false;
    }

    pub fn reset(&mut self) {
        self.cpu.reset(None);
        self.cpu.bus.ppu.reset();
    }

    pub fn get_pattern_table(
        &mut self,
        idx: usize,
        palette_id: u8,
    ) -> ppu_consts::SprPatternTableUnitT {
        self.cpu.bus.ppu.get_pattern_table(idx, palette_id)
    }

    pub fn clock_one_frame(&mut self) {
        self.reset_frame_status();
        while !self.cpu.bus.ppu.frame_complete {
            self.clock();
        }
    }

    pub fn clock_one_instruction(&mut self) {
        self.reset_instruction_status();
        while !self.get_instruction_status() {
            self.clock();
        }
    }
    pub fn get_instruction_status(&self) -> bool {
        self.cpu.instruction_complete
    }
    pub fn reset_instruction_status(&mut self) {
        self.cpu.instruction_complete = false;
    }

    pub fn get_oam(&self) -> [ObjectAttributeEntry; ppu_consts::OAM_SIZE] {
        self.cpu.bus.ppu.oam.clone()
    }

    pub fn get_ppu_status(&self) -> u8 {
        self.cpu.bus.ppu.debug_get_status()
    }
    pub fn get_ppu_scanline(&self) -> usize {
        self.cpu.bus.ppu.debug_get_scanline()
    }
    pub fn get_ppu_cycle(&self) -> usize {
        self.cpu.bus.ppu.debug_get_cycle()
    }
    pub fn get_ppu_vram(&self) -> u16 {
        self.cpu.bus.ppu.debug_get_vram_addr()
    }
    pub fn get_ppu_tram(&self) -> u16 {
        self.cpu.bus.ppu.debug_get_tram_addr()
    }
    pub fn get_ppu_ctrl(&self) -> u8 {
        self.cpu.bus.ppu.debug_get_ctrl()
    }
    pub fn get_ppu_mask(&self) -> u8 {
        self.cpu.bus.ppu.debug_get_mask()
    }
    pub fn get_ppu_frame_count(&self) -> i32 {
        self.cpu.bus.ppu.frame_complete_count
    }
    pub fn clock_one_scanline(&mut self) {
        let sl = self.cpu.bus.ppu.debug_get_scanline();
        while self.cpu.bus.ppu.debug_get_scanline() == sl {
            self.clock_one_instruction();
        }
    }
}
