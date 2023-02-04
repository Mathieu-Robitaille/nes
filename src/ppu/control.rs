use super::{generate_dummy_screen, PPU};
use crate::consts::ppu_consts::STARTING_SCANLINE;

impl PPU {
    pub fn reset(&mut self) {
        self.fine_x = 0x00;
        self.ppu_first_write = true;
        self.ppu_data_buffer = 0x00;
        self.scanline = STARTING_SCANLINE;
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
}
