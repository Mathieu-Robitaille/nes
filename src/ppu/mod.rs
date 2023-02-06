mod control;
pub mod debug;
pub mod helpers;
mod read;
mod statics;
pub mod structures;
mod write;

// 26:00
use structures::*;
use helpers::*;
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

    pub oam: [ObjectAttributeEntry; OAM_SIZE],
    pub oam_addr: u8,

    pub frame_complete: bool,
    pub frame_complete_count: i32,

    pub(crate) status: StatusRegister,
    pub(crate) mask: MaskRegister,
    pub(crate) ctrl: ControlRegister,
    pub(crate) vram_addr: VramRegister,
    pub(crate) tram_addr: VramRegister,

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

    // Sprites
    sprites_to_render: Vec<ObjectAttributeEntry>,
    sprite_shifter_pattern_lo: [u8; 8],
    sprite_shifter_pattern_hi: [u8; 8],

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

            oam: [ObjectAttributeEntry::default(); OAM_SIZE],
            oam_addr: 0,

            sprites_to_render: vec![],
            sprite_shifter_pattern_lo: [0; 8],
            sprite_shifter_pattern_hi: [0; 8],

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

        // Background logic
        if (1..=257).contains(&self.cycle) || (321..=337).contains(&self.cycle) {
            self.update_bg_shifters();
            match (self.cycle - 1) % 8 {
                // Fetch nametable byte
                0 => {
                    self.load_background_shifters();
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
                7 => self.increment_scroll_x(),
                _ => {}
            }

            let color = self.get_color_to_draw();

            if (0..NUM_SCANLINES_RENDERED).contains(&self.scanline) && (0..WIDTH).contains(&self.cycle) {
                write_pixel_to_output(
                    ((self.scanline * NUM_CYCLES_PER_SCANLINE) + self.cycle) * COLOR_CHANNELS,
                    &mut self.screen,
                    color,
                );
            }
        }

        if self.cycle == 256 {
            self.increment_scroll_y();
        }

        if self.cycle == 257 {
            // load_background_shifters(self);
            self.transfer_address_x()
        }

        if self.cycle == 338 || self.cycle == 340 {
            self.bg_next_tile_id = self.ppu_read(0x2000 | (self.vram_addr.get_register() & 0x0FFF))
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
                    self.sprite_shifter_pattern_lo = [0; 8];
                    self.sprite_shifter_pattern_hi = [0; 8]; 
                }

                self.process_visible_cycle();

                if (280..=304).contains(&self.cycle) {
                    /* vertical scroll bits are reloaded if rendering is enabled. */
                    self.transfer_address_y();
                }
            }
        }

        // Sprite logic
        if self.cycle == 257 && self.scanline != 261 {
            self.load_sprites_to_render();
        }

        if self.cycle == 340 {
            for (i, e) in self.sprites_to_render.iter().enumerate() {
                let pattern_addr_lo: u16;
                let (_x, y, id, attr): (u16, u16, u16, u16) = e.to_u16_arr();
                if !self.ctrl.sprite_size.get_as_bool() {
                    // 8x8 mode
                    let table: u16 = self.ctrl.pattern_sprite.get_as_value() as u16;

                    if e.attribute & 0x80 == 0 {
                        // Normal vertical orientation
                        pattern_addr_lo = 
                            (table << 12)
                            | id << 4
                            | ((self.scanline as u16) - y);

                    } else {
                        // flipped vertical
                        pattern_addr_lo = 
                            (table << 12)
                            | id << 4
                            | (7 - ((self.scanline as u16) - y));
                    }
                } else {
                    // 8x16 mode
                    if e.attribute & 0x80 == 0 {
                        // Normal vertical orientation
                        if (0..8).contains(&(self.scanline - (e.y as usize))) {
                            pattern_addr_lo =
                                (id & 0x0001) << 12
                                | (id & 0x00FE) << 4
                                | ((self.scanline as u16) - y);
                        } else {
                            pattern_addr_lo =
                                (id & 0x0001) << 12
                                | ((id & 0x00FE) + 1) << 4
                                | ((self.scanline as u16) - y);
                        }
                    } else {
                        // flipped vertical
                        if (0..8).contains(&(self.scanline - (e.y as usize))) {
                            pattern_addr_lo =
                                (id & 0x0001) << 12
                                | (id & 0x00FE) << 4
                                | (7 - ((self.scanline as u16) - y));
                        } else {
                            pattern_addr_lo =
                                (id & 0x0001) << 12
                                | ((id & 0x00FE) + 1) << 4
                                | ((self.scanline as u16) - y);
                        }
                    }
                }

                if attr & 0x40 > 0 {
                    self.sprite_shifter_pattern_lo[i] = self.ppu_read(pattern_addr_lo).reverse_bits();
                    self.sprite_shifter_pattern_hi[i] = self.ppu_read(pattern_addr_lo + 8).reverse_bits();
                } else {
                    self.sprite_shifter_pattern_lo[i] = self.ppu_read(pattern_addr_lo); 
                    self.sprite_shifter_pattern_hi[i] = self.ppu_read(pattern_addr_lo + 8);
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

    #[inline(always)]
    fn grayscale(&self) -> u8 {
        if self.mask.grayscale.get_as_value() > 0 {
            0x30
        } else {
            0x3F
        }
    }

    pub fn get_color_from_palette_ram(&self, palette: u8, pixel: u8) -> Pixel {
        let idx = self.ppu_read(0x3F00 + (((palette << 2) + pixel) & 0x3F) as u16);
        statics::COLORS[idx as usize]
    }

    /// This emulates the ppu not accepting writes to registers 0x0000 0x0001 0x0005 0x0006 before give or take 30k cycles/instructions? (unclear)
    #[allow(unused)]
    fn passed_warmup(&self) -> bool {
        self.clock_count > PPU_CTRL_IGNORE_CYCLES && PPU_WARM_UP_ENABLE
    }

    fn increment_scroll_x(&mut self) {
        if self.can_render() {
            if self.vram_addr.coarse_x == REG_COARSE_X {
                // If coarse x is 31 reset it
                self.vram_addr.coarse_x.zero(); // wipe coarse X
                self.vram_addr.nametable_x.flip_bits(); // toggle table
            } else {
                // Otherwise increment it
                self.vram_addr.coarse_x.increment();
            }
        }
    }

    fn increment_scroll_y(&mut self) {
        if self.can_render() {
            if self.vram_addr.fine_y.get_as_value() < 7 {
                self.vram_addr.fine_y.increment();
            } else {
                self.vram_addr.fine_y.zero();

                if self.vram_addr.coarse_y == 29 {
                    // we need to swap vertical nametables
                    self.vram_addr.coarse_y.zero();
                    // flip the nametable bit
                    self.vram_addr.nametable_y.flip_bits();
                } else if self.vram_addr.coarse_y == 31 {
                    // if we're in attr mem, reset it
                    self.vram_addr.coarse_y.zero();
                } else {
                    self.vram_addr.coarse_y.increment();
                }
            }
        }
    }

    fn transfer_address_x(&mut self) {
        if self.can_render() {
            self.vram_addr
                .nametable_x
                .set(self.tram_addr.nametable_x.get());
            self.vram_addr.coarse_x.set(self.tram_addr.coarse_x.get());
        }
    }

    fn transfer_address_y(&mut self) {
        if self.can_render() {
            self.vram_addr.fine_y.set(self.tram_addr.fine_y.get());
            self.vram_addr
                .nametable_y
                .set(self.tram_addr.nametable_y.get());
            self.vram_addr.coarse_y.set(self.tram_addr.coarse_y.get());
        }
    }

    
    fn load_background_shifters(&mut self) {
        self.bg_shifter_pattern_lo = (self.bg_shifter_pattern_lo & 0xFF00) | self.bg_next_tile_lsb as u16;
        self.bg_shifter_pattern_hi = (self.bg_shifter_pattern_hi & 0xFF00) | self.bg_next_tile_msb as u16;
        self.bg_shifter_attrib_lo = (self.bg_shifter_attrib_lo & 0xFF00)
            | if self.bg_next_tile_attrib & 0b01 > 0 {
                0x00FF
            } else {
                0x0000
            };
        self.bg_shifter_attrib_hi = (self.bg_shifter_attrib_hi & 0xFF00)
            | if self.bg_next_tile_attrib & 0b10 > 0 {
                0x00FF
            } else {
                0x0000
            };
    }

    fn update_bg_shifters(&mut self) {
        if self.mask.render_background.get_as_bool() {
            self.bg_shifter_pattern_lo <<= 1;
            self.bg_shifter_pattern_hi <<= 1;
            self.bg_shifter_attrib_lo <<= 1;
            self.bg_shifter_attrib_hi <<= 1;
        }
        if self.mask.render_sprites.get_as_bool() && (0..258).contains(&self.cycle) {
            for (i, e) in self.sprites_to_render.iter().enumerate() {
                let (x, _, _, _) = e.to_u16_arr();
                if (x..(x + 8)).contains(&(self.cycle as u16)) {
                    self.sprite_shifter_pattern_lo[i] <<= 1;
                    self.sprite_shifter_pattern_hi[i] <<= 1;
                }
            }
        }
    }

    fn can_render(&self) -> bool {
        if self.mask.render_background.get_as_value() > 0 || self.mask.render_sprites.get_as_value() > 0 {
            return true;
        }
        false
    }

    fn get_sprite_size(&self) -> usize {
        if self.ctrl.sprite_size.get_as_bool() {
            return 16;
        }
        8
    }

    fn load_sprites_to_render(&mut self) {
        self.sprites_to_render = self.oam
            .iter()
            .filter(|x| {
                (0..self.get_sprite_size()).contains(&(self.scanline - x.y as usize))
            })
            .map(|x| x.clone())
            .collect::<Vec<ObjectAttributeEntry>>();
        if self.sprites_to_render.len() > 8 {
            let (truncated, _) = self.sprites_to_render.split_at(7);
            self.sprites_to_render = truncated.to_vec();
            self.status.sprite_overflow.one();
        }
    }

    fn get_color_to_draw(&self) -> Pixel {
        let mut bg_pixel: u8 = 0x00;
        let mut bg_palette: u8 = 0x00;

        let mut fg_pixel: u8 = 0x00;
        let mut fg_palette: u8 = 0x00;
        let mut fg_priority: bool = false;

        if self.mask.render_background.get_as_bool() {
            let bit_mux = 0x8000 >> self.fine_x;
            let p0_pixel: u8 = ((self.bg_shifter_pattern_lo & bit_mux) > 0) as u8;
            let p1_pixel: u8 = ((self.bg_shifter_pattern_hi & bit_mux) > 0) as u8;

            bg_pixel = (p1_pixel << 1) | p0_pixel;

            let bg_pal0: u8 = ((self.bg_shifter_attrib_lo & bit_mux) > 0) as u8;
            let bg_pal1: u8 = ((self.bg_shifter_attrib_hi & bit_mux) > 0) as u8;
            bg_palette = (bg_pal1 << 1) | bg_pal0;
        }

        if self.mask.render_sprites.get_as_bool() {
            'SpriteEvaluation :for (i, e) in self.sprites_to_render.iter().enumerate() {
                let (x, _, _, _) = e.to_u16_arr();
                if (x..(x + 8)).contains(&(self.cycle as u16)) {
                    let p0_pixel: u8 = ((self.sprite_shifter_pattern_lo[i] & 0x80) > 0) as u8;
                    let p1_pixel: u8 = ((self.sprite_shifter_pattern_hi[i] & 0x80) > 0) as u8;
                    fg_pixel = (p1_pixel << 1) | p0_pixel;
                    if fg_pixel != 0 {
                        fg_palette = (e.attribute & 0x03) + 0x04;
                        fg_priority = (e.attribute & 0x20) == 0;
                        break 'SpriteEvaluation;
                    }   
                }
            }
        }
        
        match (bg_pixel, fg_pixel, fg_priority) {
            (0, 1..=u8::MAX, _) => { return self.get_color_from_palette_ram(fg_palette, fg_pixel); },
            (1..=u8::MAX, 1..=u8::MAX, true) => { return self.get_color_from_palette_ram(fg_palette, fg_pixel); },
            _ => { return self.get_color_from_palette_ram(bg_palette, bg_pixel); },
        }

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
