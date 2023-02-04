use super::{helpers::write_pixel_to_output, PPU};
use crate::consts::{emulation_consts::COLOR_CHANNELS, ppu_consts::*};

#[allow(unused)]
impl PPU {
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
    pub fn get_name_table(&self, index: usize) -> [u8; NAME_TABLE_SIZE] {
        self.name_table[index]
    }
    pub fn get_spr_name_table(&self, index: usize) -> SprScreenT {
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
    pub fn debug_get_fine_x(&self) -> u8 {
        self.fine_x
    }
    /// if true set to 32 else 1
    pub fn debug_set_ctrl_increment(&mut self, b: bool) {
        self.ctrl.increment_mode.set_with_unshifted(b as u8);
    }

    pub fn debug_get_pattern_table(
        &mut self,
        index: usize,
        palette_id: u8,
    ) -> SprPatternTableUnitT {
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
}
