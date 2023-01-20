use crate::cartridge::{Cartridge, MIRROR};
use bitfield;
use olc_pixel_game_engine as olc;
use std::{cell::RefCell, rc::Rc};

pub struct PPU {
    cart: Rc<RefCell<Cartridge>>,
    pub name_table: [[u8; 1024]; 2],
    pub pattern_table: [[u8; 4096]; 2],
    pub palette: [u8; 32],

    pal_screen: [olc::Pixel; 0x40],
    spr_screen: Rc<RefCell<olc::Sprite>>,
    spr_name_table: [Rc<RefCell<olc::Sprite>>; 2],
    spr_pattern_table: [Rc<RefCell<olc::Sprite>>; 2],

    pub frame_complete: bool,
    pub frame_complete_count: i32,

    status: u8,
    mask: u8,
    ctrl: u8,

    pub nmi: bool,

    vram_addr: u16, // Active "pointer" address into nametable to extract background tile info
    tram_addr: u16, // Temporary store of information to be "transferred" into "pointer" at various times

    fine_x: u8, // Pixel offset horizontally

    // Internal communications
    address_latch: bool, // true = 0 / false = 1
    ppu_data_buffer: u8,

    // Pixel "dot" position information
    scanline: i16,
    cycle: i16,

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
}

impl PPU {
    pub fn new(cart: Rc<RefCell<Cartridge>>) -> Self {
        Self {
            cart: cart.clone(),
            name_table: [[0; 1024], [0; 1024]],
            pattern_table: [[0; 4096], [0; 4096]],
            palette: [0; 32],
            pal_screen: PPU::init_pal_screen(),

            // We need to be sure that the functions that call these
            //  return valid mem
            spr_screen: Rc::new(RefCell::new(olc::Sprite::with_dims(256, 240))),
            spr_name_table: [
                Rc::new(RefCell::new(olc::Sprite::with_dims(256, 240))),
                Rc::new(RefCell::new(olc::Sprite::with_dims(256, 240))),
            ],
            spr_pattern_table: [
                Rc::new(RefCell::new(olc::Sprite::with_dims(128, 128))),
                Rc::new(RefCell::new(olc::Sprite::with_dims(128, 128))),
            ],

            frame_complete: false,
            frame_complete_count: 0,

            status: 0,
            mask: 0,
            ctrl: 0,
            nmi: false,
            vram_addr: 0,
            tram_addr: 0,
            fine_x: 0,
            address_latch: true,
            ppu_data_buffer: 0,
            scanline: 0,
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
        }
    }

    fn increment_scroll_x(&mut self) {
        if !self.can_render() {
            return;
        }
        if self.vram_addr & REG_COARSE_X == REG_COARSE_X {
            self.vram_addr &= !REG_COARSE_X; // wipe coarse X
            self.vram_addr ^= REG_NAMETABLE_X; // toggle table
        } else {
            let mut x: u16 = self.vram_addr & REG_COARSE_X;
            x += 1;
            self.vram_addr &= !REG_COARSE_X;
            self.vram_addr |= x;
        }
    }

    fn increment_scroll_y(&mut self) {
        if !self.can_render() {
            return;
        }
        let mut fine_y = (self.vram_addr & REG_FINE_Y) >> 12;
        if fine_y < 7 {
            // we can just increment fine y once and leave
            fine_y += 1;
            self.vram_addr &= !REG_FINE_Y;
            self.vram_addr |= (fine_y << 12)
        } else {
            self.vram_addr &= !REG_FINE_Y;

            let mut coarse_y = (self.vram_addr & REG_COARSE_Y) >> 5;
            self.vram_addr &= !REG_COARSE_Y;
            //  do we need to swap vertical nametables?
            if let 29 = coarse_y {
                // yes, reset coarse y
                self.vram_addr ^= REG_NAMETABLE_Y;
            } else {
                coarse_y += 1;
                self.vram_addr |= coarse_y << 5;
            }
        }
    }

    fn transfer_address_x(&mut self) {
        if !self.can_render() {
            return;
        }
        let mask = REG_NAMETABLE_X | REG_COARSE_X;
        self.vram_addr &= !mask;
        self.vram_addr |= self.tram_addr & mask;
    }

    fn transfer_address_y(&mut self) {
        if !self.can_render() {
            return;
        }
        let mask = REG_FINE_Y | REG_NAMETABLE_Y | REG_COARSE_Y;
        self.vram_addr &= !mask;
        self.vram_addr |= self.tram_addr & mask;
    }

    fn load_background_shifters(&mut self) {
        self.bg_shifter_pattern_lo =
            (self.bg_shifter_pattern_lo & 0xFF00) | self.bg_next_tile_lsb as u16;
        self.bg_shifter_pattern_hi =
            (self.bg_shifter_pattern_hi & 0xFF00) | self.bg_next_tile_msb as u16;
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

    fn update_shifters(&mut self) {
        if !self.can_render() {
            return;
        }
        self.bg_shifter_pattern_lo <<= 1;
        self.bg_shifter_pattern_hi <<= 1;
        self.bg_shifter_attrib_lo <<= 1;
        self.bg_shifter_attrib_hi <<= 1;
    }

    fn can_render(&self) -> bool {
        if self.mask & (MASK_RENDER_BACKGROUND | MASK_RENDER_SPRITES) > 0 {
            return true;
        }
        false
    }

    pub fn clock(&mut self) {
        match self.scanline {
            -1..=239 => {
                match (self.scanline, self.cycle) {
                    (-1, 1) => self.status &= !STATUS_VERTICAL_BLANK_MASK,
                    (0, 0) => self.cycle = 1,
                    (_, 2..=257 | 321..=337) => {
                        self.update_shifters();
                        match (self.cycle - 1) % 8 {
                            0 => {
                                self.load_background_shifters();
                                self.bg_next_tile_id =
                                    self.ppu_read(0x2000 | (self.vram_addr & 0x0FFF));
                            }
                            2 => {
                                let a: u16 = self.vram_addr & REG_NAMETABLE_Y << 11;
                                let b: u16 = self.vram_addr & REG_NAMETABLE_X << 10;
                                let c: u16 = ((self.vram_addr & REG_COARSE_Y) >> 7) << 3;
                                let d: u16 = self.vram_addr & REG_COARSE_X >> 2;
                                self.bg_next_tile_attrib = self.ppu_read(0x23C0 | a | b | c | d);

                                if ((self.vram_addr & REG_COARSE_Y) >> 5) & 0x02 > 0 {
                                    self.bg_next_tile_attrib >>= 4
                                }
                                if (self.vram_addr & REG_COARSE_X) & 0x02 > 0 {
                                    self.bg_next_tile_attrib >>= 2
                                }
                                self.bg_next_tile_attrib &= 0x03;
                                // 👀 pls
                            }
                            4 => {
                                let a: u16 = ((self.ctrl & CTRL_PATTERN_BACKGROUND) as u16) << 8;
                                let b: u16 = (self.bg_next_tile_id as u16) << 4;
                                let c: u16 = (self.vram_addr & REG_FINE_Y) >> 12;
                                self.bg_next_tile_lsb = self.ppu_read(a + b + c + 0);
                            }
                            6 => {
                                let a: u16 = ((self.ctrl & CTRL_PATTERN_BACKGROUND) as u16) << 8;
                                let b: u16 = (self.bg_next_tile_id as u16) << 4;
                                let c: u16 = (self.vram_addr & REG_FINE_Y) >> 12;
                                self.bg_next_tile_msb = self.ppu_read(a + b + c + 8);
                            }
                            7 => self.increment_scroll_x(),
                            _ => {}
                        }
                    }
                    (_, 256) => self.increment_scroll_y(),
                    (_, 257) => {
                        self.load_background_shifters();
                        self.transfer_address_x()
                    }
                    (_, 338 | 340) => {
                        self.bg_next_tile_id = self.ppu_read(0x2000 | (self.vram_addr & 0x0FFF))
                    }
                    (-1, 280 | 305) => self.transfer_address_y(),
                    _ => {}
                }
            }
            240 => { /* Verbose "we're not doing anything" */ }
            241..=261 => {
                self.status |= STATUS_VERTICAL_BLANK_MASK;
                if self.ctrl & CTRL_ENABLE_NMI > 0 {
                    self.nmi = true;
                }
            }
            _ => {}
        }

        let mut bg_pixel: u8 = 0x00;
        let mut bg_palette: u8 = 0x00;

        if self.mask & MASK_RENDER_BACKGROUND > 0 {
            let bit_mux = 0x8000 >> self.fine_x;
            let p0_pixel: u8 = (self.bg_shifter_pattern_lo & bit_mux) as u8;
            let p1_pixel: u8 = (self.bg_shifter_pattern_hi & bit_mux) as u8;

            bg_pixel = (p1_pixel << 1) | p0_pixel;

            let bg_pal0: u8 = (self.bg_shifter_attrib_lo & bit_mux) as u8;
            let bg_pal1: u8 = (self.bg_shifter_attrib_hi & bit_mux) as u8;
            bg_palette = (bg_pal1 << 1) | bg_pal0;
        }

        self.spr_screen.borrow_mut().set_pixel(
            (self.cycle as i32) - 1,
            self.scanline as i32,
            self.get_color_from_palette_ram(bg_palette, bg_pixel),
        );

        self.cycle += 1;
        if self.cycle >= 341 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline >= 261 {
                self.scanline = -1;
                self.frame_complete = true;
                self.frame_complete_count += 1;
            }
        }
    }

    // Plaase ignore the cpu read/write sections, the 6502 and 2c02 can eat my shorts.
    pub fn cpu_read(&mut self, addr: u16, read_only: bool) -> u8 {
        match (addr, read_only) {
            (0x0000, true) => {
                return self.ctrl;
            } // Control
            (0x0001, true) => {
                return self.mask;
            } // Mask
            (0x0002, true) => {
                return self.status;
            } // Status
            (0x0002, false) => {
                // Reading from the status register has the effect of resetting
                // different parts of the circuit. Only the top three bits
                // contain status information, however it is possible that
                // some "noise" gets picked up on the bottom 5 bits which
                // represent the last PPU bus transaction. Some games "may"
                // use this noise as valid data (even though they probably
                // shouldn't)
                let ret = (self.status & 0xE0) | (self.ppu_data_buffer & 0x1F);

                // Clear the vertical blanking flag
                self.status &= !STATUS_VERTICAL_BLANK_MASK;

                // Reset Loopy's Address latch flag
                self.address_latch = true;

                return ret;
            } // Status
            // 0x0003 => {
            //     return 0;
            // } // OAM Address
            // 0x0004 => {
            //     return 0;
            // } // OAM Data
            // 0x0005 => {
            //     return 0;
            // } // Scroll
            // 0x0006 => {
            //     return 0;
            // } // PPU Address
            (0x0007, false) => {
                // Reads from the NameTable ram get delayed one cycle,
                // so output buffer which contains the data from the
                // previous read request
                let mut data = self.ppu_data_buffer;

                // then update the buffer for next time
                self.ppu_data_buffer = self.ppu_read(self.vram_addr);

                // However, if the address was in the palette range, the
                // data is not delayed, so it returns immediately
                if (self.vram_addr >= 0x3F00) {
                    data = self.ppu_data_buffer
                };

                // All reads from PPU data automatically increment the nametable
                // address depending upon the mode set in the control register.
                // If set to vertical mode, the increment is 32, so it skips
                // one whole nametable row; in horizontal mode it just increments
                // by 1, moving to the next column
                self.vram_addr += if self.ctrl & CTRL_INCREMENT_MODE > 0 {
                    32
                } else {
                    1
                };
                return data;
            } // PPU Data
            _ => {
                return 0;
            } //
        }
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000 => {
                self.ctrl = data;
                self.tram_addr &= !(REG_NAMETABLE_X | REG_NAMETABLE_Y);
                self.tram_addr |=
                    ((self.ctrl & (CTRL_NAMETABLE_X | CTRL_NAMETABLE_Y)) as u16) << 10;
            } // Control
            0x0001 => self.mask = data, // Mask
            0x0002 => {}                // Status
            0x0003 => {}                // OAM Address
            0x0004 => {}                // OAM Data
            0x0005 => {
                if self.address_latch {
                    self.fine_x = data & 0x07;
                    self.tram_addr &= !REG_COARSE_X;
                    self.tram_addr |= (data >> 3) as u16;
                    self.address_latch = false;
                } else {
                    self.tram_addr &= !(REG_FINE_Y | REG_COARSE_Y);
                    self.tram_addr |= ((data & 0x07) as u16) << 12; // set fine y
                    self.tram_addr |= ((data >> 3) as u16) << 5;
                    self.address_latch = true;
                }
            } // Scroll
            0x0006 => {
                if self.address_latch {
                    self.tram_addr = ((data & 0x3F) as u16) << 8 | self.tram_addr & 0x00FF;
                    self.address_latch = false;
                } else {
                    self.tram_addr = (self.tram_addr & 0xFF00) | data as u16;
                    self.vram_addr = self.tram_addr;
                    self.address_latch = true;
                }
            } // PPU Address
            0x0007 => {
                self.ppu_write(self.vram_addr, data);
                self.vram_addr += if self.ctrl & CTRL_INCREMENT_MODE > 0 {
                    32
                } else {
                    1
                };
            } // PPU Data
            _ => {}
        }
    }

    pub fn ppu_read(&self, addr: u16) -> u8 {
        let mut local_addr: u16 = addr & 0x3FFF;

        if let Ok(x) = self.cart.borrow().ppu_read(local_addr) {
            return x; // If the cart intercepts it we just ship that back
        } else if (0x0000..=0x1FFF).contains(&local_addr) {
            return self.pattern_table[((local_addr & 0x1000) >> 12) as usize]
                [(local_addr & 0x0FFF) as usize];
        } else if (0x2000..=0x3EFF).contains(&local_addr) {
            local_addr &= 0x0FFF;
            let mut bank: usize = 0;

            match (local_addr, self.cart.borrow().mirror) {
                (0x0000..=0x03FF, _) => {}
                (0x0400..=0x07FF, MIRROR::VERTICAL) => bank = 1,
                (0x0800..=0x0BFF, MIRROR::HORIZONTAL) => bank = 1,
                (0x0C00..=0x0FFF, _) => bank = 1,
                _ => {}
            }

            return self.name_table[bank][(local_addr & 0x03FF) as usize];
        } else if (0x3F00..=0x3FFF).contains(&local_addr) {
            local_addr &= 0x001F;
            // Blank cell mirroring for transparancy
            if let 0x0010 | 0x0014 | 0x0018 | 0x001C = local_addr {
                local_addr &= !0x0010
            }
            return self.palette[local_addr as usize] & self.grayscale();
        }
        0
    }

    pub fn ppu_write(&mut self, addr: u16, data: u8) {
        let mut local_addr: u16 = addr & 0x3FFF;
        let mut cart_steal: bool = false;
        if let Ok(_) = self.cart.borrow_mut().ppu_write(addr, data) { 
            cart_steal = true;
        }
        /* else */
        if cart_steal {
        } else if (0x0000..=0x1FFF).contains(&local_addr) {
            // normally a rom but adding because adding random bugs can be fun!
            self.pattern_table[((local_addr & 0x1000) >> 12) as usize]
                [(local_addr & 0x0FFF) as usize] = data;
        } else if (0x2000..=0x3EFF).contains(&local_addr) {
            local_addr &= 0x0FFF;
            let mut bank: usize = 0;

            match (local_addr, self.cart.borrow().mirror) {
                (0x0000..=0x03FF, _) => {}
                (0x0400..=0x07FF, MIRROR::VERTICAL) => bank = 1,
                (0x0800..=0x0BFF, MIRROR::HORIZONTAL) => bank = 1,
                (0x0C00..=0x0FFF, _) => bank = 1,
                _ => {}
            }
            self.name_table[bank][(local_addr & 0x03FF) as usize] = data;
        } else if (0x3F00..=0x3FFF).contains(&local_addr) {
            local_addr &= 0x001F;
            // Blank cell mirroring for transparancy
            if let 0x0010 | 0x0014 | 0x0018 | 0x001C = local_addr {
                local_addr &= !0x0010
            }
            self.palette[local_addr as usize] = data;
        }
    }

    fn grayscale(&self) -> u8 {
        if self.mask & MASK_GRAYSCALE > 0 {
            0x30
        } else {
            0x3F
        }
    }

    pub fn get_screen(&self) -> Rc<RefCell<olc::Sprite>> {
        self.spr_screen.clone()
    }

    pub fn debug_get_status(&self) -> u8 {
        self.status
    }
    pub fn debug_get_scanline(&self) -> i16 {
        self.scanline
    }
    pub fn debug_get_cycle(&self) -> i16 {
        self.cycle
    }

    pub fn get_name_table(&self, index: usize) -> Rc<RefCell<olc::Sprite>> {
        self.spr_name_table[index].clone()
    }

    pub fn reset(&mut self) {
        self.fine_x = 0x00;
        self.address_latch = true;
        self.ppu_data_buffer = 0x00;
        self.scanline = 0;
        self.cycle = 0;
        self.bg_next_tile_id = 0x00;
        self.bg_next_tile_attrib = 0x00;
        self.bg_next_tile_lsb = 0x00;
        self.bg_next_tile_msb = 0x00;
        self.bg_shifter_pattern_lo = 0x0000;
        self.bg_shifter_pattern_hi = 0x0000;
        self.bg_shifter_attrib_lo = 0x0000;
        self.bg_shifter_attrib_hi = 0x0000;
        self.status = 0x00;
        self.mask = 0x00;
        self.ctrl = 0x00;
        self.vram_addr = 0x0000;
        self.tram_addr = 0x0000
    }

    // Gross 1:1 re implementation, maybe we look at this in the future
    // Have to add rc refcell due to no copy/clone
    pub fn get_pattern_table(&self, index: usize, palette_id: u8) -> Rc<RefCell<olc::Sprite>> {
        if !(0..=1).contains(&index) {
            println!("new sprite");
            return Rc::new(RefCell::new(olc::Sprite::with_dims(128, 128)));
        }

        for y in 0..0x000F {
            for x in 0..0x000F {
                let offset = y * 256 + x * 16;
                for r in 0..8 {
                    let mut tile_lsb: u8 = self.ppu_read((index as u16) * 0x1000 + offset + r);
                    let mut tile_msb: u8 = self.ppu_read((index as u16) * 0x1000 + offset + r + 8);
                    for c in 0..8 {
                        let pixel: u8 = (tile_lsb & 0x01) + (tile_msb & 0x01);
                        tile_lsb >>= 1;
                        tile_msb >>= 1;

                        // println!("Setting pattern_table pixel x:{} y:{} -> {}", (x * 8 + (7 - c)), (y * 8 + r), self.get_color_from_palette_ram(palette_id, pixel));

                        self.spr_pattern_table[index].borrow_mut().set_pixel(
                            (x * 8 + (7 - c)) as i32,
                            (y * 8 + r) as i32,
                            self.get_color_from_palette_ram(palette_id, pixel),
                        );
                    }
                }
            }
        }

        self.spr_pattern_table[index].clone()
    }

    pub fn get_color_from_palette_ram(&self, palette: u8, pixel: u8) -> olc::Pixel {
        let idx = self.ppu_read(0x3F00 + ((palette << 2) + pixel) as u16) & 0x3F;
        if self.debug { println!("Reading color at idx: {idx:04X?} | pal : {palette:02X?} | pix: {pixel:02X?}"); }
        // println!("Returning : {:?}", self.pal_screen[idx as usize]);
        self.pal_screen[idx as usize]
    }

    fn init_pal_screen() -> [olc::Pixel; 0x40] {
        let mut pal_screen = [olc::Pixel {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }; 0x40];
        pal_screen[0x00] = olc::Pixel {
            r: 84,
            g: 84,
            b: 84,
            a: 255,
        };
        pal_screen[0x01] = olc::Pixel {
            r: 0,
            g: 30,
            b: 116,
            a: 255,
        };
        pal_screen[0x02] = olc::Pixel {
            r: 8,
            g: 16,
            b: 144,
            a: 255,
        };
        pal_screen[0x03] = olc::Pixel {
            r: 48,
            g: 0,
            b: 136,
            a: 255,
        };
        pal_screen[0x04] = olc::Pixel {
            r: 68,
            g: 0,
            b: 100,
            a: 255,
        };
        pal_screen[0x05] = olc::Pixel {
            r: 92,
            g: 0,
            b: 48,
            a: 255,
        };
        pal_screen[0x06] = olc::Pixel {
            r: 84,
            g: 4,
            b: 0,
            a: 255,
        };
        pal_screen[0x07] = olc::Pixel {
            r: 60,
            g: 24,
            b: 0,
            a: 255,
        };
        pal_screen[0x08] = olc::Pixel {
            r: 32,
            g: 42,
            b: 0,
            a: 255,
        };
        pal_screen[0x09] = olc::Pixel {
            r: 8,
            g: 58,
            b: 0,
            a: 255,
        };
        pal_screen[0x0A] = olc::Pixel {
            r: 0,
            g: 64,
            b: 0,
            a: 255,
        };
        pal_screen[0x0B] = olc::Pixel {
            r: 0,
            g: 60,
            b: 0,
            a: 255,
        };
        pal_screen[0x0C] = olc::Pixel {
            r: 0,
            g: 50,
            b: 60,
            a: 255,
        };
        pal_screen[0x0D] = olc::Pixel {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        };
        pal_screen[0x0E] = olc::Pixel {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        };
        pal_screen[0x0F] = olc::Pixel {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        };

        pal_screen[0x10] = olc::Pixel {
            r: 152,
            g: 150,
            b: 152,
            a: 255,
        };
        pal_screen[0x11] = olc::Pixel {
            r: 8,
            g: 76,
            b: 196,
            a: 255,
        };
        pal_screen[0x12] = olc::Pixel {
            r: 48,
            g: 50,
            b: 236,
            a: 255,
        };
        pal_screen[0x13] = olc::Pixel {
            r: 92,
            g: 30,
            b: 228,
            a: 255,
        };
        pal_screen[0x14] = olc::Pixel {
            r: 136,
            g: 20,
            b: 176,
            a: 255,
        };
        pal_screen[0x15] = olc::Pixel {
            r: 160,
            g: 20,
            b: 100,
            a: 255,
        };
        pal_screen[0x16] = olc::Pixel {
            r: 152,
            g: 34,
            b: 32,
            a: 255,
        };
        pal_screen[0x17] = olc::Pixel {
            r: 120,
            g: 60,
            b: 0,
            a: 255,
        };
        pal_screen[0x18] = olc::Pixel {
            r: 84,
            g: 90,
            b: 0,
            a: 255,
        };
        pal_screen[0x19] = olc::Pixel {
            r: 40,
            g: 114,
            b: 0,
            a: 255,
        };
        pal_screen[0x1A] = olc::Pixel {
            r: 8,
            g: 124,
            b: 0,
            a: 255,
        };
        pal_screen[0x1B] = olc::Pixel {
            r: 0,
            g: 118,
            b: 40,
            a: 255,
        };
        pal_screen[0x1C] = olc::Pixel {
            r: 0,
            g: 102,
            b: 120,
            a: 255,
        };
        pal_screen[0x1D] = olc::Pixel {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        };
        pal_screen[0x1E] = olc::Pixel {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        };
        pal_screen[0x1F] = olc::Pixel {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        };

        pal_screen[0x20] = olc::Pixel {
            r: 236,
            g: 238,
            b: 236,
            a: 255,
        };
        pal_screen[0x21] = olc::Pixel {
            r: 76,
            g: 154,
            b: 236,
            a: 255,
        };
        pal_screen[0x22] = olc::Pixel {
            r: 120,
            g: 124,
            b: 236,
            a: 255,
        };
        pal_screen[0x23] = olc::Pixel {
            r: 176,
            g: 98,
            b: 236,
            a: 255,
        };
        pal_screen[0x24] = olc::Pixel {
            r: 228,
            g: 84,
            b: 236,
            a: 255,
        };
        pal_screen[0x25] = olc::Pixel {
            r: 236,
            g: 88,
            b: 180,
            a: 255,
        };
        pal_screen[0x26] = olc::Pixel {
            r: 236,
            g: 106,
            b: 100,
            a: 255,
        };
        pal_screen[0x27] = olc::Pixel {
            r: 212,
            g: 136,
            b: 32,
            a: 255,
        };
        pal_screen[0x28] = olc::Pixel {
            r: 160,
            g: 170,
            b: 0,
            a: 255,
        };
        pal_screen[0x29] = olc::Pixel {
            r: 116,
            g: 196,
            b: 0,
            a: 255,
        };
        pal_screen[0x2A] = olc::Pixel {
            r: 76,
            g: 208,
            b: 32,
            a: 255,
        };
        pal_screen[0x2B] = olc::Pixel {
            r: 56,
            g: 204,
            b: 108,
            a: 255,
        };
        pal_screen[0x2C] = olc::Pixel {
            r: 56,
            g: 180,
            b: 204,
            a: 255,
        };
        pal_screen[0x2D] = olc::Pixel {
            r: 60,
            g: 60,
            b: 60,
            a: 255,
        };
        pal_screen[0x2E] = olc::Pixel {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        };
        pal_screen[0x2F] = olc::Pixel {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        };

        pal_screen[0x30] = olc::Pixel {
            r: 236,
            g: 238,
            b: 236,
            a: 255,
        };
        pal_screen[0x31] = olc::Pixel {
            r: 168,
            g: 204,
            b: 236,
            a: 255,
        };
        pal_screen[0x32] = olc::Pixel {
            r: 188,
            g: 188,
            b: 236,
            a: 255,
        };
        pal_screen[0x33] = olc::Pixel {
            r: 212,
            g: 178,
            b: 236,
            a: 255,
        };
        pal_screen[0x34] = olc::Pixel {
            r: 236,
            g: 174,
            b: 236,
            a: 255,
        };
        pal_screen[0x35] = olc::Pixel {
            r: 236,
            g: 174,
            b: 212,
            a: 255,
        };
        pal_screen[0x36] = olc::Pixel {
            r: 236,
            g: 180,
            b: 176,
            a: 255,
        };
        pal_screen[0x37] = olc::Pixel {
            r: 228,
            g: 196,
            b: 144,
            a: 255,
        };
        pal_screen[0x38] = olc::Pixel {
            r: 204,
            g: 210,
            b: 120,
            a: 255,
        };
        pal_screen[0x39] = olc::Pixel {
            r: 180,
            g: 222,
            b: 120,
            a: 255,
        };
        pal_screen[0x3A] = olc::Pixel {
            r: 168,
            g: 226,
            b: 144,
            a: 255,
        };
        pal_screen[0x3B] = olc::Pixel {
            r: 152,
            g: 226,
            b: 180,
            a: 255,
        };
        pal_screen[0x3C] = olc::Pixel {
            r: 160,
            g: 214,
            b: 228,
            a: 255,
        };
        pal_screen[0x3D] = olc::Pixel {
            r: 160,
            g: 162,
            b: 160,
            a: 255,
        };
        pal_screen[0x3E] = olc::Pixel {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        };
        pal_screen[0x3F] = olc::Pixel {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        };
        pal_screen
    }
}

fn ppu_ranges() {
    //  Sprites
    let chr_rom = (0x0000..=0x1FFF); // pattern memory

    // Layout
    let name_table = (0x2000..=0x3EFF); // vram

    // Colors
    let palette_memory = (0x3F00..=0x3FFF);
}

// https://www.nesdev.org/wiki/PPU_registers

// Status
pub const STATUS_UNUSED_MASK: u8 = 0b0001_1111;
pub const STATUS_SPRT_OVERFLOW_MASK: u8 = 0b0010_0000;
pub const STATUS_SPRT_HIT_ZERO_MASK: u8 = 0b0100_0000;
pub const STATUS_VERTICAL_BLANK_MASK: u8 = 0b1000_0000;

// Mask
pub const MASK_GRAYSCALE: u8 = 0b0000_0001;
pub const MASK_RENDER_BACKGROUND_LEFT: u8 = 0b0000_0010;
pub const MASK_RENDER_SPRITES_LEFT: u8 = 0b0000_0100;
pub const MASK_RENDER_BACKGROUND: u8 = 0b0000_1000;
pub const MASK_RENDER_SPRITES: u8 = 0b0001_0000;
pub const MASK_ENHANCE_RED: u8 = 0b0010_0000;
pub const MASK_ENHANCE_GREEN: u8 = 0b0100_0000;
pub const MASK_ENHANCE_BLUE: u8 = 0b1000_0000;

// CTRL
pub const CTRL_NAMETABLE_X: u8 = 0b0000_0001;
pub const CTRL_NAMETABLE_Y: u8 = 0b0000_0010;
pub const CTRL_INCREMENT_MODE: u8 = 0b0000_0100;
pub const CTRL_PATTERN_SPRITE: u8 = 0b0000_1000;
pub const CTRL_PATTERN_BACKGROUND: u8 = 0b0001_0000;
pub const CTRL_SPRITE_SIZE: u8 = 0b0010_0000;
pub const CTRL_SLAVE_MODE: u8 = 0b0100_0000;
pub const CTRL_ENABLE_NMI: u8 = 0b1000_0000;

// PPU Register
pub const REG_COARSE_X: u16 = 0b0000_0000_0001_1111;
pub const REG_COARSE_Y: u16 = 0b0000_0011_1110_0000;
pub const REG_NAMETABLE_X: u16 = 0b0000_0100_0000_0000;
pub const REG_NAMETABLE_Y: u16 = 0b0000_1000_0000_0000;
pub const REG_FINE_Y: u16 = 0b0111_0000_0000_0000;
pub const REG_UNUSED: u16 = 0b1000_0000_0000_0000;
