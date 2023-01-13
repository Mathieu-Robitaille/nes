use crate::cpu::CPUFlags;
use crate::nes::Nes;
use crate::olc_pixel_game_engine as olc;

const RAM_X_POS: i32 = 0;
const RAM_Y_POS: i32 = 2;
const CPU_X_POS: i32 = 450;
const CPU_Y_POS: i32 = 2;

pub fn draw_debug(nes: &mut Nes) -> Result<(), olc::Error> {
    draw_cpu(CPU_X_POS, CPU_Y_POS, nes)?;
    draw_ram(RAM_X_POS, RAM_Y_POS, 2, 0x0000, nes)?;
    draw_ram(
        RAM_X_POS,
        RAM_Y_POS + 30,
        16,
        nes.cpu.pc & 0xFF0F, /* add filter logic here to page less */
        nes,
    )?;
    draw_code(CPU_X_POS, 104, 20, nes);
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
    olc::draw_string(x, y, "Status:", olc::WHITE);
    olc::draw_string(x + 64, y, "N", cpu_color(nes.cpu.status & CPUFlags::N))?;
    olc::draw_string(x + 80, y, "V", cpu_color(nes.cpu.status & CPUFlags::V))?;
    olc::draw_string(x + 96, y, "U", cpu_color(nes.cpu.status & CPUFlags::U))?;
    olc::draw_string(x + 112, y, "B", cpu_color(nes.cpu.status & CPUFlags::B))?;
    olc::draw_string(x + 128, y, "D", cpu_color(nes.cpu.status & CPUFlags::D))?;
    olc::draw_string(x + 144, y, "I", cpu_color(nes.cpu.status & CPUFlags::I))?;
    olc::draw_string(x + 160, y, "Z", cpu_color(nes.cpu.status & CPUFlags::Z))?;
    olc::draw_string(x + 178, y, "C", cpu_color(nes.cpu.status & CPUFlags::C))?;
    olc::draw_string(
        x,
        y + 10,
        format!("PCounter    : {:04X?}", nes.cpu.pc).as_str(),
        olc::WHITE,
    )?;
    olc::draw_string(
        x,
        y + 20,
        format!("Accumulator : {:02X?} [{:?}]", nes.cpu.acc, nes.cpu.acc).as_str(),
        olc::WHITE,
    )?;
    olc::draw_string(
        x,
        y + 30,
        format!("X Register  : {:02X?} [{:?}]", nes.cpu.x_reg, nes.cpu.x_reg).as_str(),
        olc::WHITE,
    )?;
    olc::draw_string(
        x,
        y + 40,
        format!("Y Register  : {:02X?} [{:?}]", nes.cpu.y_reg, nes.cpu.y_reg).as_str(),
        olc::WHITE,
    )?;
    olc::draw_string(
        x,
        y + 50,
        format!("Stack ptr   : {:02X?}", nes.cpu.stack_pointer).as_str(),
        olc::WHITE,
    )?;
    olc::draw_string(
        x,
        y + 60,
        format!("Fetched     : {:02X?}", nes.cpu.fetched).as_str(),
        olc::WHITE,
    )?;
    olc::draw_string(
        x,
        y + 70,
        format!("Ins count   : {}", nes.cpu.instruction_count).as_str(),
        olc::WHITE,
    )?;
    Ok(())
}

fn draw_ram(x: i32, y: i32, col: i32, addr: u16, nes: &mut Nes) -> Result<(), olc::Error> {
    if (addr..=(addr + (col * 16) as u16)).contains(&nes.cpu.pc) {
        let base_offset = (6 * 9);
        let current_x =
            x + base_offset;// + ((nes.cpu.pc % 16) * 16) as i32 + ((nes.cpu.pc % 16) * 8) as i32; // wait what
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

fn draw_code(x: i32, y: i32, num_lines: usize, nes: &mut Nes) -> Result<(), olc::Error> {
    fn cur_ins(a: &u16, b: &u16) -> olc::Pixel {
        if a == b {
            return olc::GREEN;
        }
        olc::WHITE
    }

    let bound = num_lines / 2;
    olc::draw_string(x, y - 10, "Addr       Ins Data   [Rel]   Mode", olc::WHITE)?;

    let mut keys: Vec<&u16> = nes.decoded_rom.keys().collect::<Vec<&u16>>();
    keys.sort();

    if let Some(idx) = keys.iter().position(|&x| x == &nes.cpu.pc) {
        let start: usize = match (idx as u16).overflowing_sub(bound as u16) {
            (_, true) => 0,
            (x, false) => x as usize,
        };

        let end: usize = match (idx as u16).overflowing_add(bound as u16) {
            (_, true) => 0xFFFF,
            (x, false) => x as usize,
        };

        for (i, e) in keys[start..=end].iter().enumerate() {
            if let Some(ins) = nes.decoded_rom.get(e) {
                olc::draw_string(x, y + (i * 10) as i32, ins, cur_ins(&**e, &nes.cpu.pc))?;
            }
        }
    }

    Ok(())
}
