use crate::cpu::CPUFlags;
use crate::nes::Nes;
use crate::olc_pixel_game_engine as olc;
use crate::ppu::*;

const TEXT_HEIGHT: i32 = 10;
const TEXT_WIDTH: i32 = 8;

const RAM_X_POS: i32 = 0;
const RAM_Y_POS: i32 = 2;

const CPU_X_POS: i32 = 512 + TEXT_WIDTH;
const CPU_Y_POS: i32 = TEXT_HEIGHT;
const CPU_DISPLAY_LINES: i32 = 8;
const CPU_Y_SIZE: i32 = CPU_DISPLAY_LINES * TEXT_HEIGHT;

const PPU_STATUS_X_POS: i32 = CPU_X_POS;
const PPU_STATUS_Y_POS: i32 = CPU_Y_POS + CPU_Y_SIZE + TEXT_HEIGHT;
const PPU_DISPLAY_LINES: i32 = 2;
const PPU_Y_SIZE: i32 = 20;

const PPU_SCREEN_X: i32 = 10;
const PPU_SCREEN_Y: i32 = 10;

const PPU_CHR_X: i32 = 0;
const PPU_CHR_Y: i32 = 0;

const PPU_SPR_X: i32 = 0;
const PPU_SPR_Y: i32 = 0;

const PPU_PALLET_X_POS: i32 = CPU_X_POS;
const PPU_PALLET_Y_POS: i32 = PPU_STATUS_Y_POS + PPU_Y_SIZE + 10;

const CODE_X_POS: i32 = PPU_PALLET_X_POS;
const CODE_Y_POS: i32 = PPU_PALLET_Y_POS + 10 + 128;
const CODE_DISPLAY_LINES: i32 = 7;
const CODE_Y_SIZE: i32 = CODE_DISPLAY_LINES * TEXT_HEIGHT;

pub fn draw_debug(nes: &mut Nes) -> Result<(), olc::Error> {
    draw_cpu(CPU_X_POS, CPU_Y_POS, nes)?;
    draw_ppu_status(PPU_STATUS_X_POS, PPU_STATUS_Y_POS, nes);
    draw_swatches(PPU_STATUS_X_POS, PPU_STATUS_Y_POS - 8, nes);
    
    // draw_ppu_screen(PPU_SCREEN_X, PPU_SCREEN_Y, nes);
    draw_ppu_bg_ids(PPU_SCREEN_X, PPU_SCREEN_Y, nes);
    draw_ppu_tables(PPU_PALLET_X_POS, PPU_PALLET_Y_POS, nes);

    // draw_ram(RAM_X_POS, RAM_Y_POS, 2, 0x0000, nes)?;
    // draw_ram(
    //     RAM_X_POS,
    //     RAM_Y_POS + 30,
    //     16,
    //     nes.cpu.pc & 0xFF0F, /* add filter logic here to page less */
    //     nes,
    // )?;

    draw_code(CODE_X_POS, CODE_Y_POS, CODE_DISPLAY_LINES, nes);
    Ok(())
}

fn draw_cpu(x: i32, y: i32, nes: &mut Nes) -> Result<(), olc::Error> {
    // Helper just to clean up the draw calls
    fn cpu_color(f: u8) -> olc::Pixel {
        if f > 0 {
            return olc::GREEN;
        }
        olc::RED
    }
    let offset: i32 = 64;
    olc::draw_string(x, y, "Status:", olc::WHITE);
    olc::draw_string(
        x + offset + 0 * TEXT_WIDTH,
        y,
        "N",
        cpu_color(nes.cpu.status & CPUFlags::N),
    )?;
    olc::draw_string(
        x + offset + 2 * TEXT_WIDTH,
        y,
        "V",
        cpu_color(nes.cpu.status & CPUFlags::V),
    )?;
    olc::draw_string(
        x + offset + 4 * TEXT_WIDTH,
        y,
        "U",
        cpu_color(nes.cpu.status & CPUFlags::U),
    )?;
    olc::draw_string(
        x + offset + 6 * TEXT_WIDTH,
        y,
        "B",
        cpu_color(nes.cpu.status & CPUFlags::B),
    )?;
    olc::draw_string(
        x + offset + 8 * TEXT_WIDTH,
        y,
        "D",
        cpu_color(nes.cpu.status & CPUFlags::D),
    )?;
    olc::draw_string(
        x + offset + 10 * TEXT_WIDTH,
        y,
        "I",
        cpu_color(nes.cpu.status & CPUFlags::I),
    )?;
    olc::draw_string(
        x + offset + 12 * TEXT_WIDTH,
        y,
        "Z",
        cpu_color(nes.cpu.status & CPUFlags::Z),
    )?;
    olc::draw_string(
        x + offset + 14 * TEXT_WIDTH,
        y,
        "C",
        cpu_color(nes.cpu.status & CPUFlags::C),
    )?;
    olc::draw_string(
        x,
        y + 1 * TEXT_HEIGHT,
        format!("PCounter    : {:04X?}", nes.cpu.pc).as_str(),
        olc::WHITE,
    )?;
    olc::draw_string(
        x,
        y + 2 * TEXT_HEIGHT,
        format!("Accumulator : {:02X?} [{:?}]", nes.cpu.acc, nes.cpu.acc).as_str(),
        olc::WHITE,
    )?;
    olc::draw_string(
        x,
        y + 3 * TEXT_HEIGHT,
        format!("X Register  : {:02X?} [{:?}]", nes.cpu.x_reg, nes.cpu.x_reg).as_str(),
        olc::WHITE,
    )?;
    olc::draw_string(
        x,
        y + 4 * TEXT_HEIGHT,
        format!("Y Register  : {:02X?} [{:?}]", nes.cpu.y_reg, nes.cpu.y_reg).as_str(),
        olc::WHITE,
    )?;
    olc::draw_string(
        x,
        y + 5 * TEXT_HEIGHT,
        format!("Stack ptr   : {:02X?}", nes.cpu.stack_pointer).as_str(),
        olc::WHITE,
    )?;
    olc::draw_string(
        x,
        y + 6 * TEXT_HEIGHT,
        format!("Fetched     : {:02X?}", nes.cpu.fetched).as_str(),
        olc::WHITE,
    )?;
    olc::draw_string(
        x,
        y + 7 * TEXT_HEIGHT,
        format!("Ins count   : {}", nes.cpu.instruction_count).as_str(),
        olc::WHITE,
    )?;
    Ok(())
}

fn draw_ram(x: i32, y: i32, col: i32, addr: u16, nes: &mut Nes) -> Result<(), olc::Error> {
    if (addr..=(addr + (col * 16) as u16)).contains(&nes.cpu.pc) {
        let base_offset = (6 * 9);
        let current_x = x + base_offset; // + ((nes.cpu.pc % 16) * 16) as i32 + ((nes.cpu.pc % 16) * 8) as i32; // wait what
                                         // un stipod this pls
        let current_y = y + (((nes.cpu.pc - addr) / 16) * 10) as i32 - 1;
        olc::draw_rect(current_x, current_y, 17, 9, olc::GREEN);
    }
    for i in 0..col {
        let mut line = format!("${:04X?}:", addr + (i * 16) as u16);
        for j in 0..16 {
            line.push_str(
                format!(" {:02X?}", nes.cpu.read_bus(addr + (16 * i) as u16 + j)).as_str(),
            );
        }
        olc::draw_string(x, y + (10 * i), &line, olc::WHITE)?;
    }
    Ok(())
}

// This is very sloooooooow
fn draw_code(x: i32, y: i32, num_lines: i32, nes: &mut Nes) -> Result<(), olc::Error> {
    fn cur_ins(a: &u16, b: &u16) -> olc::Pixel {
        if a == b {
            return olc::GREEN;
        }
        olc::WHITE
    }

    let bound = num_lines / 2;
    olc::draw_string(x, y, "Addr       Ins Data   [Rel]   Mode", olc::WHITE)?;

    let mut keys: Vec<&u16> = nes.decoded_rom.keys().collect::<Vec<&u16>>();
    keys.sort();

    if let Some(idx) = keys.iter().position(|&x| x == &nes.cpu.pc) {
        let start: usize = match (idx as u16).overflowing_sub(bound as u16) {
            (_, true) => 0,
            (x, false) => x as usize,
        };

        let mut end: usize = match (idx as u16).overflowing_add(bound as u16) {
            (_, true) => 0xFFFF - 1,
            (x, false) => x as usize,
        };
        if end >= keys.len() {
            end = end -1;
        }

        for (i, e) in keys[start..end].iter().enumerate() {
            if let Some(ins) = nes.decoded_rom.get(e) {
                olc::draw_string(x, y + 10 + (i * 10) as i32, ins, cur_ins(&**e, &nes.cpu.pc))?;
            }
        }
    }

    Ok(())
}

fn draw_ppu_status(x: i32, y: i32, nes: &mut Nes) -> Result<(), olc::Error> {
    fn ppu_color(f: u8) -> olc::Pixel {
        if f > 0 {
            return olc::GREEN;
        }
        olc::RED
    }
    let headder = "PPU:";

    let scanline = format!("Scanline: {:0>3}", nes.cpu.bus.ppu.debug_get_scanline());
    let scanline_len = (scanline.len() as i32) * TEXT_WIDTH;
    let scanline_start = x;
    let scanline_end = scanline_start + scanline_len + 1 * TEXT_WIDTH;

    let cycle = format!("Cycle: {:0>3}", nes.cpu.bus.ppu.debug_get_cycle());
    let cycle_len = (cycle.len() as i32) * TEXT_WIDTH;
    let cycle_start = scanline_end;
    let cycle_end = cycle_start + cycle_len + 1 * TEXT_WIDTH;

    let frame_complete_count = format!("FC: {:0>3}", nes.cpu.bus.ppu.frame_complete_count);
    let frame_complete_count_len = (frame_complete_count.len() as i32) * TEXT_WIDTH;
    let frame_complete_count_start = cycle_end;
    let frame_complete_count_end =
        frame_complete_count_start + frame_complete_count_len + 1 * TEXT_WIDTH;

    let status = nes.cpu.bus.ppu.debug_get_status();
    let offset: i32 = ((headder.len() as i32) * TEXT_HEIGHT);

    olc::draw_string(x, y, headder, olc::WHITE);
    olc::draw_string(
        x + offset,
        y,
        "VB",
        ppu_color(status & STATUS_VERTICAL_BLANK_MASK),
    )?;
    olc::draw_string(
        x + offset + 3 * TEXT_WIDTH,
        y,
        "SZ",
        ppu_color(status & STATUS_SPRT_HIT_ZERO_MASK),
    )?;
    olc::draw_string(
        x + offset + 6 * TEXT_WIDTH,
        y,
        "OV",
        ppu_color(status & STATUS_SPRT_OVERFLOW_MASK),
    )?;
    olc::draw_string(
        x + offset + 9 * TEXT_WIDTH,
        y,
        format!("{:0>5b}", status & STATUS_UNUSED_MASK).as_str(),
        ppu_color(status & STATUS_UNUSED_MASK),
    )?;

    olc::draw_string(
        scanline_start,
        y + TEXT_HEIGHT,
        scanline.as_str(),
        olc::WHITE,
    )?;

    olc::draw_string(cycle_start, y + TEXT_HEIGHT, cycle.as_str(), olc::WHITE)?;

    olc::draw_string(
        frame_complete_count_start,
        y + TEXT_HEIGHT,
        frame_complete_count.as_str(),
        olc::WHITE,
    )?;

    Ok(())
}

fn draw_ppu_screen(x: i32, y: i32, nes: &mut Nes) {
    let mut screen_x: i32 = 0;
    let mut screen_y: i32 = 0;
    if let Ok(spr) = nes.cpu.bus.ppu.get_screen().try_borrow() {
        screen_x = spr.width();
        screen_y = spr.height() + 2;

        olc::draw_rect(x, y, spr.width() +2, spr.height() +2, olc::WHITE);

        let debug_str = "No data.";
        let x_pos_str = x + ((spr.width() / 2) as i32) - (debug_str.len() as i32) * TEXT_WIDTH / 2;
        let y_pos_str = y + ((spr.height() / 2) as i32) - (TEXT_HEIGHT / 2);
        olc::draw_string(
            x_pos_str,
            y_pos_str,
            debug_str,
            olc::WHITE,
        );
        // olc::fill_rect(x + 1, y + 1, spr.width(), spr.height(), olc::RED);
        olc::draw_sprite(x + 1, y + 1, &*spr);
    }
}

fn draw_ppu_tables(x: i32, y: i32, nes: &mut Nes) {
    let mut width = 0;
    let x_offset_table_2 = 0;

    if let Ok(spr) = nes.cpu.bus.ppu.get_pattern_table(0, 4).try_borrow() {
        width = spr.width();

        olc::draw_rect(x, y, spr.width() +2, spr.height() +2, olc::WHITE);
        let debug_str = "No data.";
        let x_pos_str = x + ((spr.width() / 2) as i32) - (debug_str.len() as i32) * TEXT_WIDTH / 2;
        let y_pos_str = y + ((spr.height() / 2) as i32) - (TEXT_HEIGHT / 2);
        olc::draw_string(
            x_pos_str,
            y_pos_str,
            debug_str,
            olc::WHITE,
        );
        olc::draw_sprite(x + 1, y + 1, &*spr);
    } else {
        eprintln!("Could not open pattern table 0");
    }

    if let Ok(spr) = nes.cpu.bus.ppu.get_pattern_table(1, 4).try_borrow() {
        olc::draw_rect(x + width + x_offset_table_2, y, spr.width() +2, spr.height() +2, olc::WHITE);
        let debug_str = "No data.";
        let x_pos_str = x + width + x_offset_table_2 + ((spr.width() / 2) as i32) - (debug_str.len() as i32) * TEXT_WIDTH / 2;
        let y_pos_str = y + ((spr.height() / 2) as i32) - (TEXT_HEIGHT / 2);
        olc::draw_string(
            x_pos_str,
            y_pos_str,
            debug_str,
            olc::WHITE,
        );
        olc::draw_sprite(x + 1 + width + x_offset_table_2, y + 1, &*spr);
    } else {
        eprintln!("Could not open pattern table 1");
    }
}

fn draw_ppu_bg_ids(x: i32, y: i32, nes: &mut Nes) {
    for y_iter in 0..30 {
        for x_iter in 0..32 {
            let s = format!("{:02X?}", nes.cpu.bus.ppu.name_table[0][(y * 32 + x) as usize]);
            olc::draw_string(x_iter * 16, y_iter * 16, s.as_str(), olc::WHITE);
        }
    }
}

fn draw_swatches(x: i32, y: i32, nes: &mut Nes) {
    let swatch_size: i32 = 6;
    for p in 0..8 {
        for s in 0..4 {
            olc::fill_rect(x + p * (swatch_size * 5) + s * swatch_size, y, swatch_size, swatch_size, nes.cpu.bus.ppu.get_color_from_palette_ram(p as u8, s as u8));
        }
    }
}

// Dumps the listed instructions
pub fn dump_code(nes: &mut Nes) {
    let mut keys: Vec<&u16> = nes.decoded_rom.keys().collect::<Vec<&u16>>();
    keys.sort();
    for key in keys {
        if let Some(x) = nes.decoded_rom.get(key) {
            println!("{:?}", x);
        }
    }
}