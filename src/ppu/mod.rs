mod control;
pub mod debug;
pub mod helpers;
mod read;
mod statics;
pub mod structures;
mod write;

// use bitfield;
use crate::cartridge::Cartridge;
use crate::consts::{
    emulation_consts::COLOR_CHANNELS,
    ppu_consts::*,
    screen_consts::{HEIGHT, WIDTH},
};
use std::{cell::RefCell, rc::Rc};

pub struct PPU {
    cart: Rc<RefCell<Cartridge>>,
    pub name_table: NameTableT,
    pub pattern_table: PatternTableT,
    pub palette: PaletteT,

    screen: ScreenT,

    #[allow(unused)]
    spr_name_table: SprNameTableT,

    spr_pattern_table: SprPatternTableT,

    pub oam: [structures::ObjectAttributeEntry; OAM_SIZE],
    pub oam_addr: u8,

    pub frame_complete: bool,
    pub frame_complete_count: i32,

    pub(crate) status: structures::StatusRegister,
    pub(crate) mask: structures::MaskRegister,
    pub(crate) ctrl: structures::ControlRegister,
    pub(crate) vram_addr: structures::VramRegister,
    pub(crate) tram_addr: structures::VramRegister,

    pub nmi: bool,

    fine_x: u8, // Pixel offset horizontally

    // Internal communications
    ppu_first_write: bool, // true = 0 / false = 1
    ppu_data_buffer: u8,

    // Pixel "dot" position information
    scanline: usize,
    cycle: usize,

    // Background rendering
    bg_next_tile_id: u8,
    bg_next_tile_attrib: u8,
    bg_next_tile_lsb: u8,
    bg_next_tile_msb: u8,
    bg_shifter_pattern_lo: u16,
    bg_shifter_pattern_hi: u16,
    bg_shifter_attrib_lo: u16,
    bg_shifter_attrib_hi: u16,

    pub debug: bool,
    clock_count: usize,
}

impl PPU {
    pub fn new(cart: Rc<RefCell<Cartridge>>) -> Self {
        Self {
            cart: cart.clone(),
            name_table: [[0; 1024], [0; 1024]],
            pattern_table: [[0; 4096], [0; 4096]],
            palette: [0; 32],

            // We need to be sure that the functions that call these
            //  return valid mem
            screen: generate_dummy_screen(),
            spr_name_table: [
                [0; WIDTH * HEIGHT * COLOR_CHANNELS],
                [0; WIDTH * HEIGHT * COLOR_CHANNELS],
            ],
            spr_pattern_table: [
                [0; SPR_PATTERN_TABLE_SIZE * SPR_PATTERN_TABLE_SIZE * COLOR_CHANNELS],
                [0; SPR_PATTERN_TABLE_SIZE * SPR_PATTERN_TABLE_SIZE * COLOR_CHANNELS],
            ],

            oam: [structures::ObjectAttributeEntry::default(); OAM_SIZE],
            oam_addr: 0,

            frame_complete: false,
            frame_complete_count: 0,

            status: 0.into(),
            mask: 0.into(),
            ctrl: 0.into(),
            vram_addr: 0.into(),
            tram_addr: 0.into(),
            nmi: false,
            fine_x: 0,
            ppu_first_write: true,
            ppu_data_buffer: 0,

            scanline: STARTING_SCANLINE,
            cycle: 0,

            bg_next_tile_id: 0,
            bg_next_tile_attrib: 0,
            bg_next_tile_lsb: 0,
            bg_next_tile_msb: 0,
            bg_shifter_pattern_lo: 0,
            bg_shifter_pattern_hi: 0,
            bg_shifter_attrib_lo: 0,
            bg_shifter_attrib_hi: 0,
            debug: false,
            clock_count: 0,
        }
    }

    // Only called by clock
    fn process_visible_cycle(&mut self) {
        /// Fetches the pattern table location
        /// lsb toggles if we add 8 or not
        fn calculate_pattern_table_addr(ppu: &mut PPU, lsb: bool) {
            let a: u16 = (ppu.ctrl.pattern_background.get_as_value() as u16) << 12;
            let b: u16 = (ppu.bg_next_tile_id as u16) << 4;
            let c: u16 = ppu.vram_addr.fine_y.get_as_value();
            if lsb {
                ppu.bg_next_tile_lsb = ppu.ppu_read(a + b + c + 0x0000);
            } else {
                ppu.bg_next_tile_msb = ppu.ppu_read(a + b + c + 0x0008);
            };
        }

        fn get_tile_attrib(ppu: &mut PPU) {
            let a: u16 = ppu.vram_addr.nametable_y.get();
            let b: u16 = ppu.vram_addr.nametable_x.get();
            let c: u16 = (ppu.vram_addr.coarse_y.get_as_value() >> 2) << 3;
            let d: u16 = ppu.vram_addr.coarse_x.get_as_value() >> 2;
            let addr = 0x23C0 | a | b | c | d;
            ppu.bg_next_tile_attrib = ppu.ppu_read(addr);

            if ppu.vram_addr.coarse_y.get_as_value() & 0x02 > 0 {
                ppu.bg_next_tile_attrib >>= 4
            }
            if ppu.vram_addr.coarse_x.get_as_value() & 0x02 > 0 {
                ppu.bg_next_tile_attrib >>= 2
            }
            ppu.bg_next_tile_attrib &= 0x03;
        }

        if (1..=257).contains(&self.cycle) || (321..=337).contains(&self.cycle) {
            helpers::update_shifters(self);
            match (self.cycle - 1) % 8 {
                // Fetch nametable byte
                0 => {
                    helpers::load_background_shifters(self);
                    self.bg_next_tile_id =
                        self.ppu_read(0x2000 | (self.vram_addr.get_register() & 0x0FFF));
                }

                // Fetch attribute byte
                2 => {
                    get_tile_attrib(self);
                }

                // fetch pattern table tile lsb
                4 => {
                    calculate_pattern_table_addr(self, true);
                }

                // fetch pattern table tile msb
                6 => {
                    calculate_pattern_table_addr(self, false);
                }
                7 => helpers::increment_scroll_x(self),
                _ => {}
            }
            let mut bg_pixel: u8 = 0x00;
            let mut bg_palette: u8 = 0x00;
            if self.mask.render_background.get_as_value() > 0 {
                let bit_mux = 0x8000 >> self.fine_x;
                let p0_pixel: u8 = ((self.bg_shifter_pattern_lo & bit_mux) > 0) as u8;
                let p1_pixel: u8 = ((self.bg_shifter_pattern_hi & bit_mux) > 0) as u8;

                bg_pixel = (p1_pixel << 1) | p0_pixel;

                let bg_pal0: u8 = ((self.bg_shifter_attrib_lo & bit_mux) > 0) as u8;
                let bg_pal1: u8 = ((self.bg_shifter_attrib_hi & bit_mux) > 0) as u8;
                bg_palette = (bg_pal1 << 1) | bg_pal0;
            }

            let color = self.get_color_from_palette_ram(bg_palette, bg_pixel);

            if (0..NUM_SCANLINES_RENDERED).contains(&self.scanline) && (0..WIDTH).contains(&self.cycle) {
                helpers::write_pixel_to_output(
                    ((self.scanline * NUM_CYCLES_PER_SCANLINE) + self.cycle) * COLOR_CHANNELS,
                    &mut self.screen,
                    color,
                );
            }
        }

        if self.cycle == 338 || self.cycle == 340 {
            self.bg_next_tile_id = self.ppu_read(0x2000 | (self.vram_addr.get_register() & 0x0FFF))
        }

        if self.cycle == 256 {
            helpers::increment_scroll_y(self);
        }

        if self.cycle == 257 {
            // helpers::load_background_shifters(self);
            helpers::transfer_address_x(self)
        }
    }

    pub fn clock(&mut self) {
        // 0..=239    => Render screen
        // 240        => Post Render
        // 241..=260  => Vblank
        // 261        => pre-render scanline

        if self.scanline == 128 {
            self.nmi = self.nmi;
        }

        match LineState::from(self.scanline) {
            LineState::Visible => {
                if self.scanline == 0 && self.cycle == 0 {
                    self.cycle = 1;
                }
                self.process_visible_cycle();
            }
            LineState::PostRender => {}
            LineState::VBlank => {
                if self.scanline == 241 && self.cycle == 1 {
                    self.status.vertical_blank.one();
                    if self.ctrl.enable_nmi.get_as_value() > 0 {
                        self.nmi = true;
                    }
                }
            }
            LineState::PreRender => {
                if self.cycle == 1 {
                    self.status.vertical_blank.zero();
                    self.status.sprite_overflow.zero();
                    self.status.sprite_zero_hit.zero();
                    // for i in 0..8 {
                    //     // clear shifters
                }

                // self.process_visible_cycle();

                if (280..=304).contains(&self.cycle) {
                    /* vertical scroll bits are reloaded if rendering is enabled. */
                    helpers::transfer_address_y(self);
                }
            }
        }

        self.cycle += 1;
        if self.cycle >= 341 {
            self.cycle = 0;

            self.scanline += 1;
            if self.scanline > 261 {
                self.scanline = 0;
                self.frame_complete = true;
                self.frame_complete_count += 1;
            }
        }
        self.clock_count += 1;
    }

    #[allow(unused)]
    fn clock2(&mut self) {
        ///
        let increment_scroll_x = |ppu: &mut PPU| {
            if ppu.mask.render_background.get() == 0 && ppu.mask.render_sprites.get() == 0 {
                return;
            }
            if ppu.vram_addr.coarse_x.get_as_value() == 31 {
                ppu.vram_addr.coarse_x.zero();
                ppu.vram_addr.nametable_x.flip_bits();
            } else {
                ppu.vram_addr.coarse_x.increment();
            }
        };

        // let
    }

    #[inline(always)]
    fn grayscale(&self) -> u8 {
        if self.mask.grayscale.get_as_value() > 0 {
            0x30
        } else {
            0x3F
        }
    }

    pub fn get_color_from_palette_ram(&self, palette: u8, pixel: u8) -> structures::Pixel {
        let idx = self.ppu_read(0x3F00 + (((palette << 2) + pixel) & 0x3F) as u16);
        statics::COLORS[idx as usize]
    }

    /// This emulates the ppu not accepting writes to registers 0x0000 0x0001 0x0005 0x0006 before give or take 30k cycles/instructions? (unclear)
    #[allow(unused)]
    fn passed_warmup(&self) -> bool {
        self.clock_count > PPU_CTRL_IGNORE_CYCLES && PPU_WARM_UP_ENABLE
    }
}

enum LineState {
    Visible,    // 0..=239
    PostRender, // 240
    VBlank,     // 241..=260
    PreRender,  // 261
}

impl LineState {
    fn from(line: usize) -> LineState {
        match line {
            0..=239 => LineState::Visible,
            240 => LineState::PostRender,
            241..=260 => LineState::VBlank,
            261 => LineState::PreRender,
            _ => panic!("Invalid line state"),
        }
    }
}

pub fn generate_dummy_screen() -> ScreenT {
    [0; NUM_CYCLES_PER_SCANLINE * NUM_SCANLINES_RENDERED * COLOR_CHANNELS]
}
