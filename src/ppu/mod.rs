pub mod helpers;
pub mod structures;


use crate::cartridge::{Cartridge, MIRROR};
use crate::consts::{
    emulation_consts::COLOR_CHANNELS,
    ppu_consts::*,
    screen_consts::{HEIGHT, WIDTH},
};
use bitfield;
use std::{cell::RefCell, rc::Rc};

mod ppu {
    pub struct PPU {
        cart: Rc<RefCell<Cartridge>>,
        pub name_table: NameTableT,
        pub pattern_table: PatternTableT,
        pub palette: PaletteT,
    
        screen: ScreenT,
        spr_name_table: SprNameTableT,
        spr_pattern_table: SprPatternTableT,
    
        pub oam: [ObjectAttributeEntry; OAM_SIZE],
        pub oam_addr: u8,
    
        pub frame_complete: bool,
        pub frame_complete_count: i32,
    
        pub(crate) status: structures::StatusRegister,
        pub(crate) mask: structures::MaskRegister,
        pub(crate) ctrl: structures::ControlRegister,
        pub(crate) vram_addr: structures::VramRegister, // Unions are for people who hate themselves.
        pub(crate) tram_addr: structures::VramRegister, // Unions are for people who hate themselves.
    
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
        clock_count: usize, /* The counter which enables writes to the control register */
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
                    ppu.bg_next_tile_attrib = ppu.ppu_read(0x23C0 | a | b | c | d);
    
                    if ppu.vram_addr.coarse_y.get_as_value() & 0x02 > 0 {
                        ppu.bg_next_tile_attrib >>= 4
                    }
                    if ppu.vram_addr.coarse_x.get_as_value() & 0x02 > 0 {
                        ppu.bg_next_tile_attrib >>= 2
                    }
                    ppu.bg_next_tile_attrib &= 0x03;
                    // 👀 pls
                }
    
            if (2..=257).contains(&self.cycle) || (321..=337).contains(&self.cycle) {
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
            }
    
            if self.cycle == 338 || self.cycle == 340 {
                self.bg_next_tile_id = self.ppu_read(0x2000 | (self.vram_addr.get_register() & 0x0FFF))
            }
    
            if self.cycle == 256 { 
                helpers::increment_scroll_y(self); 
            }
    
            if self.cycle == 257 {
                helpers::load_background_shifters(self);
                helpers::transfer_address_x(self)
            }
        }
    
        pub fn clock(&mut self) {
            // 0..=239    => Render screen
            // 240        => Post Render
            // 241..=260  => Vblank
            // 261        => pre-render scanline
    
            match LineState::from(self.scanline) {
                LineState::Visible => {
                    if self.scanline == 0 && self.cycle == 0 { self.cycle = 1;}
                    else {
                        self.process_visible_cycle();
                    }
        
                }
                LineState::PostRender => {}
                LineState::VBlank => {
                    if self.scanline == 241 && self.cycle == 1 {
                        if self.clock_count > PPU_CTRL_IGNORE_CYCLES {
                            self.status.vertical_blank.one();
                            if self.ctrl.enable_nmi.get_as_value() > 0 {
                                self.nmi = true;
                            }
                        }
                    }
                }
                LineState::PreRender => {
                    if self.cycle == 1 {
                        self.status.vertical_blank.zero();
                        self.status.sprite_overflow.zero();
                        self.status.sprite_zero_hit.zero();
                        for i in 0..8 {
                            // clear shifters
                        }
                    }
    
                    self.process_visible_cycle();
    
                    // if self.cycle == 339 { self.cycle = 0; return; }
    
                    if (280..=304).contains(&self.cycle) { 
                        /* vertical scroll bits are reloaded if rendering is enabled. */ 
                        helpers::transfer_address_y(self);
                    }
                }
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
    
            // if (0..HEIGHT).contains(&self.scanline) && (0..WIDTH).contains(&self.cycle) {
            write_pixel_to_output(
                (self.scanline * WIDTH + self.cycle) * COLOR_CHANNELS,
                &mut self.screen,
                color,
            );
    
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
    
        // Plaase ignore the cpu read/write sections, the 6502 and 2c02 can eat my shorts.
        pub fn cpu_read(&mut self, addr: u16, read_only: bool) -> u8 {
            match (addr, read_only) {
                (0x0000, true) => {
                    return self.ctrl.get_register();
                } // Control
                (0x0001, true) => {
                    return self.mask.get_register();
                } // Mask
                (0x0002, true) => {
                    return self.status.get_register();
                } // Status
                (0x0002, false) => {
                    // Reading from the status register has the effect of resetting
                    // different parts of the circuit. Only the top three bits
                    // contain status information, however it is possible that
                    // some "noise" gets picked up on the bottom 5 bits which
                    // represent the last PPU bus transaction. Some games "may"
                    // use this noise as valid data (even though they probably
                    // shouldn't)
                    self.status.unused.set_with_unshifted(self.ppu_data_buffer & 0x1F);
                    let ret = self.status.get_register();
    
                    // Clear the vertical blanking flag
                    self.status.vertical_blank.zero();
    
                    // Reset Loopy's Address latch flag
                    self.ppu_first_write = true;
    
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
                    self.ppu_data_buffer = self.ppu_read(self.vram_addr.get_register());
    
                    // However, if the address was in the palette range, the
                    // data is not delayed, so it returns immediately
                    if (self.vram_addr.get_register() >= 0x3F00) {
                        data = self.ppu_data_buffer
                    };
    
                    // All reads from PPU data automatically increment the nametable
                    // address depending upon the mode set in the control register.
                    // If set to vertical mode, the increment is 32, so it skips
                    // one whole nametable row; in horizontal mode it just increments
                    // by 1, moving to the next column
                    let mut v: u16 = 1;
                    if self.ctrl.increment_mode.get_as_value() > 0 {
                        v = 32;   
                    }
    
                    let (reg, _) = self.vram_addr.get_register().overflowing_add(v);
                    self.vram_addr.set_register(reg);
    
                    return data;
                } // PPU Data
                _ => {
                    return 0;
                } //
            }
        }
    
        pub fn cpu_write(&mut self, addr: u16, data: u8) {
            let local_addr = addr & 0x0007;
    
    
            // if self.clock_count < PPU_CTRL_IGNORE_CYCLES { return; }
            match local_addr {
                0 => {
                    self.ctrl.set_register(data);
                    // println!("Setting ctrl to {:0>8b} -> {:0>8b}", data, self.ctrl.get_register());
                    self.tram_addr.nametable_x.set_with_unshifted(self.ctrl.nametable_x.get_as_value() as u16);
                    self.tram_addr.nametable_y.set_with_unshifted(self.ctrl.nametable_y.get_as_value() as u16);
                }
                1 => self.mask.set_register(data),
                3 => self.oam_addr = data, // OAM Address
                4 => {
                    set_oam_field(&mut self.oam, self.oam_addr, data);
                } // OAM Data
                5 => {
                    if self.ppu_first_write {
                        self.fine_x = data & 0x07;
                        self.tram_addr.coarse_x.set_with_unshifted((data >> 3) as u16);
                        self.ppu_first_write = false;
                    } else {
                        self.tram_addr.fine_y.set_with_unshifted((data & 0x07) as u16);
                        self.tram_addr.coarse_y.set_with_unshifted((data >> 3) as u16);
                        self.ppu_first_write = true;
                    }
                } // Scroll
                6 => {
                    if self.ppu_first_write {
                        // println!("setting tram {:04X}", ((data & 0x3F) as u16) << 8 | self.tram_addr.get_register() & 0x00FF);
                        self.tram_addr.set_register((data as u16) << 8 | (self.tram_addr.get_register() & 0x00FF));
                        self.ppu_first_write = false;
                    } else {
                        // println!("setting tram 2 {:04X}", (self.tram_addr.get_register() & 0xFF00) | data as u16);
                        self.tram_addr.set_register((self.tram_addr.get_register() & 0xFF00) | data as u16);
                        self.vram_addr.set_register(self.tram_addr.get_register()); // <---- I got my eyes on you
                        self.ppu_first_write = true;
                    }
                } // PPU Address
                7 => {
                    // println!("vram: {:04X} -> {:02X}", self.vram_addr, data);
    
                    self.ppu_write(
                        self.vram_addr.get_register(), data);
                    let mut v: u16 = 1;
                    if self.ctrl.increment_mode.get_as_value() > 0 { v = 32; }
                    let (reg, _) = self.vram_addr.get_register().overflowing_add(v);
                    self.vram_addr.set_register(reg);
                } // PPU Data
                _ => {}
            }
        }
    
        pub fn ppu_read(&self, addr: u16) -> u8 {
            let mut local_addr: u16 = addr & 0x3FFF;
    
            if let Ok(x) = self.cart.borrow().ppu_read(local_addr) {
                return x; // If the cart intercepts it we just ship that back
                
                // Pattern table 1 and 2
            } else if (0x0000..=0x1FFF).contains(&local_addr) { 
                return self.pattern_table[((local_addr & 0x1000) >> 12) as usize]
                    [(local_addr & 0x0FFF) as usize];
    
            // Name tables 1-4
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
            if self.mask.grayscale.get_as_value() > 0 {
                0x30
            } else {
                0x3F
            }
        }
    
        pub fn get_screen(&self) -> ScreenT {
            self.screen
        }
    
        pub fn debug_get_status(&self) -> u8 {
            self.status.get_register()
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
            self.tram_addr.get_register()
        }
        pub fn debug_get_vram_addr(&self) -> u16 {
            self.vram_addr.get_register()
        }
        pub fn debug_get_ctrl(&self) -> u8 {
            self.ctrl.get_register()
        }
        pub fn debug_get_mask(&self) -> u8 {
            self.mask.get_register()
        }
    
        /// if true set to 32 else 1
        pub fn debug_set_ctrl_increment(&mut self, b: bool) {
            self.ctrl.increment_mode.set_with_unshifted(b as u8);
        }
    
        pub fn reset(&mut self) {
            self.fine_x = 0x00;
            self.ppu_first_write = true;
            self.ppu_data_buffer = 0x00;
            self.scanline = 241;
            self.cycle = 0;
            self.bg_next_tile_id = 0x00;
            self.bg_next_tile_attrib = 0x00;
            self.bg_next_tile_lsb = 0x00;
            self.bg_next_tile_msb = 0x00;
            self.bg_shifter_pattern_lo = 0x0000;
            self.bg_shifter_pattern_hi = 0x0000;
            self.bg_shifter_attrib_lo = 0x0000;
            self.bg_shifter_attrib_hi = 0x0000;
            self.status = 0x00.into();
            self.mask = 0x00.into();
            self.ctrl = 0x00.into();
            self.vram_addr = 0x0000.into();
            self.tram_addr = 0x0000.into();
            self.clock_count = 0;
            self.frame_complete_count = 0;
            self.screen = generate_dummy_screen();
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
            let idx = self.ppu_read(0x3F00 + (((palette << 2) + pixel) & 0x3F) as u16) ;
            let possible_colors: [Pixel; 0x40] = include!("tables/colors.rs");
            possible_colors[idx as usize]
        }
    }
    
    enum LineState {
        Visible,    // 0..=239
        PostRender, // 240
        VBlank,     // 241..=260
        PreRender   // 261
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
    
    fn write_pixel_to_output(pos: usize, spr: &mut [u8], pix: Pixel) {
        spr[pos + 0] = pix.0;
        spr[pos + 1] = pix.1;
        spr[pos + 2] = pix.2;
    }
    
    pub fn generate_dummy_screen() -> ScreenT {
        [0; NUM_CYCLES_PER_SCANLINE * NUM_SCANLINES_RENDERED * COLOR_CHANNELS]
    }
}


