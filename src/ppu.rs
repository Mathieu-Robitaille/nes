use crate::cartridge::{Cartridge, MIRROR};
use crate::consts::{
    emulation_consts::COLOR_CHANNELS,
    ppu_consts::*,
    screen_consts::{HEIGHT, WIDTH},
};
use bitfield;
use std::{cell::RefCell, rc::Rc};

pub struct PPU {
    cart: Rc<RefCell<Cartridge>>,
    pub name_table: NameTableT,
    pub pattern_table: PatternTableT,
    pub palette: PaletteT,

    possible_colors: [Pixel; 0x40],
    spr_screen: SprScreenT,
    spr_name_table: SprNameTableT,
    spr_pattern_table: SprPatternTableT,

    pub oam: [ObjectAttributeEntry; OAM_SIZE],
    pub oam_addr: u8,

    pub frame_complete: bool,
    pub frame_complete_count: i32,

    status: u8,
    mask: u8,
    ctrl: u8,

    pub nmi: bool,

    vram_addr: u16, // Unions are for people who hate themselves.
    tram_addr: u16, // Unions are for people who hate themselves.

    fine_x: u8, // Pixel offset horizontally

    // Internal communications
    address_latch: bool, // true = 0 / false = 1
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
    clock_count: usize, /* The counter which enables writes to the control register */
}

impl PPU {
    pub fn new(cart: Rc<RefCell<Cartridge>>) -> Self {
        Self {
            cart: cart.clone(),
            name_table: [[0; 1024], [0; 1024]],
            pattern_table: [[0; 4096], [0; 4096]],
            palette: [0; 32],
            possible_colors: init_pal_screen(),

            // We need to be sure that the functions that call these
            //  return valid mem
            spr_screen: [0; WIDTH * HEIGHT * COLOR_CHANNELS],
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

            frame_complete: false,
            frame_complete_count: 0,

            status: 0,
            mask: 0,
            ctrl: 0,
            nmi: false,
            vram_addr: 0, // Change these out for a bunch of registers. dealing with unions can eat my ass.
            tram_addr: 0, // Change these out for a bunch of registers. dealing with unions can eat my ass.
            fine_x: 0,
            address_latch: true,
            ppu_data_buffer: 0,
            scanline: usize::MAX,
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

    fn increment_scroll_x(&mut self) {
        if self.can_render() {
            if (self.vram_addr & REG_COARSE_X) == REG_COARSE_X {
                self.vram_addr &= !REG_COARSE_X; // wipe coarse X
                self.vram_addr ^= REG_NAMETABLE_X; // toggle table
            } else {
                let mut x: u16 = self.vram_addr & REG_COARSE_X;
                x += 1;
                self.vram_addr &= !REG_COARSE_X;
                self.vram_addr |= x;
            }
        }
    }

    fn increment_scroll_y(&mut self) {
        if self.can_render() {
            let mut fine_y = (self.vram_addr & REG_FINE_Y) >> 12;
            let mut coarse_y = (self.vram_addr & REG_COARSE_Y) >> 5;
            self.vram_addr &= !REG_FINE_Y;
            
            if fine_y < 7 {
                fine_y += 1;
                self.vram_addr |= (fine_y << 12)
            } else {
                if coarse_y == 29 {
                    self.vram_addr &= !REG_COARSE_Y;
                    self.vram_addr ^= REG_NAMETABLE_Y;
                } /*  else if coarse_y == 31 {
                    self.vram_addr &= !REG_COARSE_Y;
                } */ else {
                    coarse_y += 1;
                    self.vram_addr |= coarse_y << 5;
                }
            }
        }
    }

    fn transfer_address_x(&mut self) {
        if self.can_render() {
            let mask = REG_NAMETABLE_X | REG_COARSE_X;
            self.vram_addr &= !mask;
            self.vram_addr |= self.tram_addr & mask;
        }
    }

    fn transfer_address_y(&mut self) {
        if self.can_render() {
            let mask = REG_FINE_Y | REG_NAMETABLE_Y | REG_COARSE_Y;
            self.vram_addr &= !mask;
            self.vram_addr |= self.tram_addr & mask;
        }
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
        if self.mask & MASK_RENDER_BACKGROUND > 0 {
            self.bg_shifter_pattern_lo <<= 1;
            self.bg_shifter_pattern_hi <<= 1;
            self.bg_shifter_attrib_lo <<= 1;
            self.bg_shifter_attrib_hi <<= 1;
        }
    }

    fn can_render(&self) -> bool {
        if self.mask & (MASK_RENDER_BACKGROUND | MASK_RENDER_SPRITES) > 0 {
            return true;
        }
        false
    }

    pub fn clock(&mut self) {
        if self.scanline != usize::MAX && self.scanline < HEIGHT {
            if self.scanline == 0 && self.cycle == 0 {
                self.cycle = 1;
            }

            if self.scanline == usize::MAX && self.cycle == 1 {
                self.status &= !STATUS_VERTICAL_BLANK_MASK;
                self.status &= !STATUS_SPRT_OVERFLOW_MASK;
                self.status &= !STATUS_SPRT_HIT_ZERO_MASK;
                for i in 0..8 {
                    // clear shifters
                }
            }

            if (2..258).contains(&self.cycle) || (321..338).contains(&self.cycle) {
                self.update_shifters();
                match (self.cycle - 1) % 8 {
                    0 => {
                        self.load_background_shifters();
                        self.bg_next_tile_id =
                            self.ppu_read(0x2000 | (self.vram_addr & 0x0FFF));
                    }
                    2 => {
                        let a: u16 = self.vram_addr & REG_NAMETABLE_Y;
                        let b: u16 = self.vram_addr & REG_NAMETABLE_X;
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

            if self.cycle == 256 { self.increment_scroll_y(); }

            if self.cycle == 257 {
                self.load_background_shifters();
                self.transfer_address_x()
            }
            
            if self.cycle == 338 || self.cycle == 338 {
                self.bg_next_tile_id = self.ppu_read(0x2000 | (self.vram_addr & 0x0FFF))
            }

            if self.scanline == usize::MAX && (280..305).contains(&self.cycle) {
                self.transfer_address_y();
            }

            if self.cycle == 257 && self.scanline != usize::MAX {
                // Sprite shits
            }

            if self.cycle == 340 {
                // Sprite shit
            }

        }
        if self.scanline == 240 { }

        if self.scanline == 241 && self.cycle == 1 {
            if self.clock_count > PPU_CTRL_IGNORE_CYCLES {
                self.status |= STATUS_VERTICAL_BLANK_MASK;
                if self.ctrl & CTRL_ENABLE_NMI > 0 {
                    self.nmi = true;
                }
            }
        }


        let mut bg_pixel: u8 = 0x00;
        let mut bg_palette: u8 = 0x00;

        if self.mask & MASK_RENDER_BACKGROUND > 0 {
            let bit_mux = 0x8000 >> self.fine_x;
            let p0_pixel: u8 = ((self.bg_shifter_pattern_lo & bit_mux) > 0) as u8;
            let p1_pixel: u8 = ((self.bg_shifter_pattern_hi & bit_mux) > 0) as u8;

            bg_pixel = (p1_pixel << 1) | p0_pixel;

            let bg_pal0: u8 = ((self.bg_shifter_attrib_lo & bit_mux) > 0) as u8;
            let bg_pal1: u8 = ((self.bg_shifter_attrib_hi & bit_mux) > 0) as u8;
            bg_palette = (bg_pal1 << 1) | bg_pal0;

            // println!("bg pal: {:02X} bg pix: {:02X} bit_mux: {:04X}", bg_palette, bg_pixel, bit_mux);
            // println!("lo: {:04X} | hi: {:04X}",self.bg_shifter_pattern_lo, self.bg_shifter_pattern_hi);
        }

        let color = self.get_color_from_palette_ram(bg_palette, bg_pixel);

        if (0..HEIGHT).contains(&self.scanline) && (0..WIDTH).contains(&self.cycle) {
            write_pixel_to_output(
                (self.scanline * WIDTH + self.cycle) * COLOR_CHANNELS,
                &mut self.spr_screen,
                color,
            );
        }

        self.cycle += 1;
        if self.cycle >= 341 {
            self.cycle = 0;

            let (r, _) = self.scanline.overflowing_add(1);
            self.scanline = r;
            if self.scanline >= 261 {
                self.scanline = usize::MAX;
                self.frame_complete = true;
                self.frame_complete_count += 1;
            }
        }

        self.clock_count += 1;
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
            (0x0004, _) => {
                return get_oam_field(&self.oam, self.oam_addr);
            } // OAM Data

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
        if self.clock_count < PPU_CTRL_IGNORE_CYCLES {
            return;
        }
        match addr {
            0x0000 => {
                self.ctrl = data;
                self.tram_addr &= !(REG_NAMETABLE_X | REG_NAMETABLE_Y);
                self.tram_addr |=
                    ((self.ctrl & (CTRL_NAMETABLE_X | CTRL_NAMETABLE_Y)) as u16) << 10;
            } // Control
            0x0001 => self.mask = data,     // Mask
            0x0002 => {}                    // Status
            0x0003 => self.oam_addr = data, // OAM Address
            0x0004 => {
                set_oam_field(&mut self.oam, self.oam_addr, data);
            } // OAM Data
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
                // println!("vram: {:04X} -> {:02X}", self.vram_addr, data);
                self.ppu_write(self.vram_addr, data);
                self.vram_addr += if self.ctrl & CTRL_INCREMENT_MODE > 0 {
                    0x20
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
            return self.palette[local_addr as usize];
            // return self.palette[local_addr as usize] & self.grayscale();
        }
        0
    }

    pub fn ppu_write(&mut self, addr: u16, data: u8) {
        let mut local_addr: u16 = addr & 0x3FFF;
        // let mut cart_steal: bool = false;
        // if let Ok(_) = self.cart.borrow_mut().ppu_write(addr, data) {
        //     cart_steal = true;
        // }
        /* else */
        // if cart_steal {
        // } else
        if (0x0000..=0x1FFF).contains(&local_addr) {
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

    pub fn get_screen(&self) -> SprScreenT {
        self.spr_screen
    }

    pub fn debug_get_status(&self) -> u8 {
        self.status
    }
    pub fn debug_get_scanline(&self) -> usize {
        self.scanline
    }
    pub fn debug_get_cycle(&self) -> usize {
        self.cycle
    }

    pub fn get_name_table(&self, index: usize) -> SprScreenT {
        self.spr_name_table[index]
    }
    pub fn debug_get_tram_addr(&self) -> u16 {
        self.tram_addr
    }
    pub fn debug_get_vram_addr(&self) -> u16 {
        self.vram_addr
    }
    pub fn debug_get_ctrl(&self) -> u8 {
        self.ctrl
    }
    pub fn debug_get_mask(&self) -> u8 {
        self.mask
    }

    pub fn reset(&mut self) {
        self.fine_x = 0x00;
        self.address_latch = true;
        self.ppu_data_buffer = 0x00;
        self.scanline = usize::MAX;
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
        self.tram_addr = 0x0000;
        self.clock_count = 0;
        self.frame_complete_count = 0;
        self.spr_screen = [0; WIDTH * HEIGHT * COLOR_CHANNELS];
    }

    pub fn get_pattern_table(&mut self, index: usize, palette_id: u8) -> SprPatternTableUnitT {
        for y_tile in 0..16 {
            for x_tile in 0..16 {
                let offset = y_tile * 256 + x_tile * 16;
                for r in 0..8 {
                    let mut tile_lsb: u8 = self.ppu_read((index as u16) * 0x1000 + offset + r);
                    let mut tile_msb: u8 = self.ppu_read((index as u16) * 0x1000 + offset + r + 8);
                    for c in 0..8 {
                        let pixel: u8 = ((tile_lsb & 0x01) << 1) | (tile_msb & 0x01);
                        tile_lsb >>= 1;
                        tile_msb >>= 1;

                        // println!("Setting pattern_table pixel x:{} y:{} -> {}", (x * 8 + (7 - c)), (y * 8 + r), self.get_color_from_palette_ram(palette_id, pixel));

                        let color = self.get_color_from_palette_ram(palette_id, pixel);
                        let x_pos: usize = ((x_tile as usize) * 8 + (7 - (c as usize)));
                        let y_pos: usize =
                            ((y_tile as usize) * 8 + (r as usize)) * SPR_PATTERN_TABLE_SIZE;
                        let pos = (y_pos + x_pos) * COLOR_CHANNELS;
                        write_pixel_to_output(pos, &mut self.spr_pattern_table[index], color);
                    }
                }
            }
        }
        self.spr_pattern_table[index]
    }

    pub fn get_color_from_palette_ram(&self, palette: u8, pixel: u8) -> Pixel {
        let idx = self.ppu_read(0x3F00 + ((palette << 2) + pixel) as u16) & 0x3F;
        self.possible_colors[idx as usize]
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ObjectAttributeEntry {
    pub y: u8,
    pub id: u8,
    pub attribute: u8,
    pub x: u8,
}

impl ObjectAttributeEntry {}

pub fn set_oam_field(oam: &mut [ObjectAttributeEntry; 64], addr: u8, data: u8) {
    let (idx, remainder) = (addr / 64, addr % 4);
    match remainder {
        0 => oam[idx as usize].y = data,
        1 => oam[idx as usize].id = data,
        2 => oam[idx as usize].attribute = data,
        3 => oam[idx as usize].x = data,
        _ => { /* No */ }
    }
}
pub fn get_oam_field(oam: &[ObjectAttributeEntry; 64], addr: u8) -> u8 {
    let (idx, remainder) = (addr / 64, addr % 4);
    match remainder {
        0 => oam[idx as usize].y,
        1 => oam[idx as usize].id,
        2 => oam[idx as usize].attribute,
        3 => oam[idx as usize].x,
        _ => {
            /* No */
            0
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
/// R G B
pub struct Pixel(u8, u8, u8);

#[derive(Debug, Default, Clone, Copy)]
/// R G B A
pub struct AlphaPixel(u8, u8, u8, u8);

fn write_pixel_to_output(pos: usize, spr: &mut [u8], pix: Pixel) {
    spr[pos + 0] = pix.0;
    spr[pos + 1] = pix.1;
    spr[pos + 2] = pix.2;
}

fn write_alpha_pixel_to_output(pos: usize, spr: &mut [u8], pix: AlphaPixel) {
    spr[pos + 0] = pix.0;
    spr[pos + 1] = pix.1;
    spr[pos + 2] = pix.2;
    spr[pos + 3] = pix.3;
}

fn init_pal_screen() -> [Pixel; 0x40] {
    let mut pal_screen = [Pixel::default(); 0x40];
    pal_screen[0x00] = Pixel(84, 84, 84);
    pal_screen[0x01] = Pixel(0, 30, 116);
    pal_screen[0x02] = Pixel(8, 16, 144);
    pal_screen[0x03] = Pixel(48, 0, 136);
    pal_screen[0x04] = Pixel(68, 0, 100);
    pal_screen[0x05] = Pixel(92, 0, 48);
    pal_screen[0x06] = Pixel(84, 4, 0);
    pal_screen[0x07] = Pixel(60, 24, 0);
    pal_screen[0x08] = Pixel(32, 42, 0);
    pal_screen[0x09] = Pixel(8, 58, 0);
    pal_screen[0x0A] = Pixel(0, 64, 0);
    pal_screen[0x0B] = Pixel(0, 60, 0);
    pal_screen[0x0C] = Pixel(0, 50, 60);
    pal_screen[0x0D] = Pixel(0, 0, 0);
    pal_screen[0x0E] = Pixel(0, 0, 0);
    pal_screen[0x0F] = Pixel(0, 0, 0);

    pal_screen[0x10] = Pixel(152, 150, 152);
    pal_screen[0x11] = Pixel(8, 76, 196);
    pal_screen[0x12] = Pixel(48, 50, 236);
    pal_screen[0x13] = Pixel(92, 30, 228);
    pal_screen[0x14] = Pixel(136, 20, 176);
    pal_screen[0x15] = Pixel(160, 20, 100);
    pal_screen[0x16] = Pixel(152, 34, 32);
    pal_screen[0x17] = Pixel(120, 60, 0);
    pal_screen[0x18] = Pixel(84, 90, 0);
    pal_screen[0x19] = Pixel(40, 114, 0);
    pal_screen[0x1A] = Pixel(8, 124, 0);
    pal_screen[0x1B] = Pixel(0, 118, 40);
    pal_screen[0x1C] = Pixel(0, 102, 120);
    pal_screen[0x1D] = Pixel(0, 0, 0);
    pal_screen[0x1E] = Pixel(0, 0, 0);
    pal_screen[0x1F] = Pixel(0, 0, 0);

    pal_screen[0x20] = Pixel(236, 238, 236);
    pal_screen[0x21] = Pixel(76, 154, 236);
    pal_screen[0x22] = Pixel(120, 124, 236);
    pal_screen[0x23] = Pixel(176, 98, 236);
    pal_screen[0x24] = Pixel(228, 84, 236);
    pal_screen[0x25] = Pixel(236, 88, 180);
    pal_screen[0x26] = Pixel(236, 106, 100);
    pal_screen[0x27] = Pixel(212, 136, 32);
    pal_screen[0x28] = Pixel(160, 170, 0);
    pal_screen[0x29] = Pixel(116, 196, 0);
    pal_screen[0x2A] = Pixel(76, 208, 32);
    pal_screen[0x2B] = Pixel(56, 204, 108);
    pal_screen[0x2C] = Pixel(56, 180, 204);
    pal_screen[0x2D] = Pixel(60, 60, 60);
    pal_screen[0x2E] = Pixel(0, 0, 0);
    pal_screen[0x2F] = Pixel(0, 0, 0);

    pal_screen[0x30] = Pixel(236, 238, 236);
    pal_screen[0x31] = Pixel(168, 204, 236);
    pal_screen[0x32] = Pixel(188, 188, 236);
    pal_screen[0x33] = Pixel(212, 178, 236);
    pal_screen[0x34] = Pixel(236, 174, 236);
    pal_screen[0x35] = Pixel(236, 174, 212);
    pal_screen[0x36] = Pixel(236, 180, 176);
    pal_screen[0x37] = Pixel(228, 196, 144);
    pal_screen[0x38] = Pixel(204, 210, 120);
    pal_screen[0x39] = Pixel(180, 222, 120);
    pal_screen[0x3A] = Pixel(168, 226, 144);
    pal_screen[0x3B] = Pixel(152, 226, 180);
    pal_screen[0x3C] = Pixel(160, 214, 228);
    pal_screen[0x3D] = Pixel(160, 162, 160);
    pal_screen[0x3E] = Pixel(0, 0, 0);
    pal_screen[0x3F] = Pixel(0, 0, 0);
    pal_screen
}
