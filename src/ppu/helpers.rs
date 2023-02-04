use super::{
    structures::{ObjectAttributeEntry, Pixel},
    PPU,
};
/// Trying to reduce bloat in the ppu file
use crate::consts::ppu_consts::*;

pub fn increment_scroll_x(ppu: &mut PPU) {
    if can_render(ppu) {
        if ppu.vram_addr.coarse_x == REG_COARSE_X {
            // If coarse x is 31 reset it
            ppu.vram_addr.coarse_x.zero(); // wipe coarse X
            ppu.vram_addr.nametable_x.flip_bits(); // toggle table
        } else {
            // Otherwise increment it
            ppu.vram_addr.coarse_x.increment();
        }
    }
}

pub fn increment_scroll_y(ppu: &mut PPU) {
    if can_render(ppu) {
        if ppu.vram_addr.fine_y.get_as_value() < 7 {
            ppu.vram_addr.fine_y.increment();
        } else {
            ppu.vram_addr.fine_y.zero();

            if ppu.vram_addr.coarse_y == 29 {
                // we need to swap vertical nametables
                ppu.vram_addr.coarse_y.zero();
                // flip the nametable bit
                ppu.vram_addr.nametable_y.flip_bits();
            } else if ppu.vram_addr.coarse_y == 31 {
                // if we're in attr mem, reset it
                ppu.vram_addr.coarse_y.zero();
            } else {
                ppu.vram_addr.coarse_y.increment();
            }
        }
    }
}

pub fn transfer_address_x(ppu: &mut PPU) {
    if can_render(ppu) {
        ppu.vram_addr
            .nametable_x
            .set(ppu.tram_addr.nametable_x.get());
        ppu.vram_addr.coarse_x.set(ppu.tram_addr.coarse_x.get());
    }
}

pub fn transfer_address_y(ppu: &mut PPU) {
    if can_render(ppu) {
        ppu.vram_addr.fine_y.set(ppu.tram_addr.fine_y.get());
        ppu.vram_addr
            .nametable_y
            .set(ppu.tram_addr.nametable_y.get());
        ppu.vram_addr.coarse_y.set(ppu.tram_addr.coarse_y.get());
    }
}

pub fn load_background_shifters(ppu: &mut PPU) {
    ppu.bg_shifter_pattern_lo = (ppu.bg_shifter_pattern_lo & 0xFF00) | ppu.bg_next_tile_lsb as u16;
    ppu.bg_shifter_pattern_hi = (ppu.bg_shifter_pattern_hi & 0xFF00) | ppu.bg_next_tile_msb as u16;
    ppu.bg_shifter_attrib_lo = (ppu.bg_shifter_attrib_lo & 0xFF00)
        | if ppu.bg_next_tile_attrib & 0b01 > 0 {
            0x00FF
        } else {
            0x0000
        };
    ppu.bg_shifter_attrib_hi = (ppu.bg_shifter_attrib_hi & 0xFF00)
        | if ppu.bg_next_tile_attrib & 0b10 > 0 {
            0x00FF
        } else {
            0x0000
        };
}

pub fn update_shifters(ppu: &mut PPU) {
    if ppu.mask.render_background.get_as_value() > 0 {
        ppu.bg_shifter_pattern_lo <<= 1;
        ppu.bg_shifter_pattern_hi <<= 1;
        ppu.bg_shifter_attrib_lo <<= 1;
        ppu.bg_shifter_attrib_hi <<= 1;
    }
}

fn can_render(ppu: &PPU) -> bool {
    if ppu.mask.render_background.get_as_value() > 0 || ppu.mask.render_sprites.get_as_value() > 0 {
        return true;
    }
    false
}

pub(super) fn write_pixel_to_output(pos: usize, spr: &mut [u8], pix: Pixel) {
    spr[pos + 0] = pix.0;
    spr[pos + 1] = pix.1;
    spr[pos + 2] = pix.2;
}

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
