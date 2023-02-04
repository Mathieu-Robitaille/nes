use super::{helpers::set_oam_field, PPU};
use crate::cartridge::MIRROR;

impl PPU {
    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        let local_addr = addr & 0x0007;
        match local_addr {
            0x0000 => {
                self.ctrl.set_register(data);
                self.tram_addr
                    .nametable_x
                    .set_with_unshifted(self.ctrl.nametable_x.get_as_value() as u16);
                self.tram_addr
                    .nametable_y
                    .set_with_unshifted(self.ctrl.nametable_y.get_as_value() as u16);
            }
            0x0001 => self.mask.set_register(data),
            0x0003 => self.oam_addr = data, // OAM Address
            0x0004 => set_oam_field(&mut self.oam, self.oam_addr, data), // OAM Data
            0x0005 => {
                if self.ppu_first_write {
                    self.fine_x = data & 0x07;
                    self.tram_addr
                        .coarse_x
                        .set_with_unshifted((data >> 3) as u16);
                    self.ppu_first_write = false;
                } else {
                    self.tram_addr
                        .fine_y
                        .set_with_unshifted((data & 0x07) as u16);
                    self.tram_addr
                        .coarse_y
                        .set_with_unshifted((data >> 3) as u16);
                    self.ppu_first_write = true;
                }
            }
            0x0006 => {
                if self.ppu_first_write {
                    let msb = ((data & 0b0011_1111) as u16) << 8;
                    let lsb = self.tram_addr.get_register() & 0x00FF;
                    self.tram_addr.set_register(msb | lsb);
                    self.tram_addr.unused.zero();
                    self.ppu_first_write = false;
                } else {
                    let msb = self.tram_addr.get_register() & 0xFF00;
                    let lsb = data as u16;
                    self.tram_addr.set_register(msb | lsb);
                    self.tram_addr.unused.zero();
                    self.vram_addr.set_register(self.tram_addr.get_register()); // <---- I got my eyes on you
                    self.ppu_first_write = true;
                }
            }
            0x0007 => {
                self.ppu_write(self.vram_addr.get_register(), data);
                let mut v: u16 = 1;
                if self.ctrl.increment_mode.get_as_value() > 0 {
                    v = 32;
                }
                let (reg, _) = self.vram_addr.get_register().overflowing_add(v);
                self.vram_addr.set_register(reg);
            } // PPU Data
            _ => {}
        }
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
}
