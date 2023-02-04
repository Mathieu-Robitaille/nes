use super::{helpers::get_oam_field, PPU};
use crate::cartridge::MIRROR;

impl PPU {
    // Plase ignore the cpu read/write sections, the 6502 and 2c02 can eat my shorts.
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
                self.status
                    .unused
                    .set_with_unshifted(self.ppu_data_buffer & 0x1F);
                let ret = self.status.get_register();

                // Clear the vertical blanking flag
                self.status.vertical_blank.zero();

                // Reset Loopy's Address latch flag
                self.ppu_first_write = true;

                return ret;
            } // Status
            (0x0004, _) => {
                return get_oam_field(&self.oam, self.oam_addr);
            } // OAM Data
            (0x0007, false) => {
                // Reads from the NameTable ram get delayed one cycle,
                // so output buffer which contains the data from the
                // previous read request
                let mut data = self.ppu_data_buffer;

                // then update the buffer for next time
                self.ppu_data_buffer = self.ppu_read(self.vram_addr.get_register());

                // However, if the address was in the palette range, the
                // data is not delayed, so it returns immediately
                if self.vram_addr.get_register() >= 0x3F00 {
                    data = self.ppu_data_buffer
                };

                // let mut v: u16 = 1;
                // if self.ctrl.increment_mode.get_as_value() > 0 {
                //     v = 32;
                // }
                // let (reg, _) = self.vram_addr.get_register().overflowing_add(v);
                // self.vram_addr.set_register(reg);

                if self.ctrl.increment_mode.get_as_value() > 0 {
                    self.vram_addr.coarse_y.increment();
                } else {
                    self.vram_addr.coarse_x.increment();
                }

                return data;
            } // PPU Data
            _ => {
                return 0;
            } //
        }
    }

    pub fn ppu_read(&self, addr: u16) -> u8 {
        let mut data: u8 = 0x00;
        let mut local_addr: u16 = addr & 0x3FFF;

        if let Ok(x) = self.cart.borrow().ppu_read(local_addr) {
            data = x;
        // Pattern table 1 and 2
        } else if (0x0000..=0x1FFF).contains(&local_addr) {
            data = self.pattern_table[((local_addr & 0x1000) >> 12) as usize]
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

            data = self.name_table[bank][(local_addr & 0x03FF) as usize];
        } else if (0x3F00..=0x3FFF).contains(&local_addr) {
            local_addr &= 0x001F;
            // Blank cell mirroring for transparancy
            if let 0x0010 | 0x0014 | 0x0018 | 0x001C = local_addr {
                local_addr &= !0x0010
            }
            data = self.palette[local_addr as usize] & self.grayscale();
        }
        data
    }
}
