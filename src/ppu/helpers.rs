
use super::structures::{ObjectAttributeEntry, Pixel};


pub(crate) fn set_oam_field(oam: &mut [ObjectAttributeEntry; 64], addr: u8, data: u8) {
    let (idx, remainder) = (addr / 64, addr % 4);
    match remainder {
        0 => oam[idx as usize].y = data,
        1 => oam[idx as usize].id = data,
        2 => oam[idx as usize].attribute = data,
        3 => oam[idx as usize].x = data,
        _ => { /* No */ }
    }
}
pub(crate) fn get_oam_field(oam: &[ObjectAttributeEntry; 64], addr: u8) -> u8 {
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

pub(super) fn write_pixel_to_output(pos: usize, spr: &mut [u8], pix: Pixel) {
    spr[pos + 0] = pix.0;
    spr[pos + 1] = pix.1;
    spr[pos + 2] = pix.2;
}