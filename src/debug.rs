use crate::olc_pixel_game_engine as olc;
use crate::cpu::Flags;
use crate::nes::Nes;
use crate::disassembler::{
    get_rom_instructions_from_range,
    HumanInstruction
};

const RAM_X_POS: i32 = 0;
const RAM_Y_POS: i32 = 2;
const CPU_X_POS: i32 = 450;
const CPU_Y_POS: i32 = 2;


pub fn draw_debug(nes: &mut Nes) -> Result<(), olc::Error> {
    draw_cpu(CPU_X_POS, CPU_Y_POS, nes)?;
    draw_ram(RAM_X_POS, RAM_Y_POS, 2, 0x0000, nes)?;
    draw_ram(RAM_X_POS, RAM_Y_POS + 30, 6, nes.cpu.program_counter & 0xFF00 /* add filter logic here to page less */, nes)?;
    draw_code(CPU_X_POS, 84, 20, nes);
    Ok(())
}

fn draw_cpu(x: i32, y: i32, nes: &mut Nes) -> Result<(), olc::Error> {
    fn cpu_color(f: u8) -> olc::Pixel {
        if f > 0 { return olc::GREEN; }
        olc::RED
    }
    olc::draw_string(x, y, "Status:", olc::WHITE);
    olc::draw_string(x + 64, y, "N", cpu_color(nes.cpu.status & Flags::N))?;
    olc::draw_string(x + 80, y, "V", cpu_color(nes.cpu.status & Flags::V))?;
    olc::draw_string(x + 96, y, "U", cpu_color(nes.cpu.status & Flags::U))?;
    olc::draw_string(x + 112, y, "B", cpu_color(nes.cpu.status & Flags::B))?;
    olc::draw_string(x + 128, y, "D", cpu_color(nes.cpu.status & Flags::D))?;
    olc::draw_string(x + 144, y, "I", cpu_color(nes.cpu.status & Flags::I))?;
    olc::draw_string(x + 160, y, "Z", cpu_color(nes.cpu.status & Flags::Z))?;
    olc::draw_string(x + 178, y, "C", cpu_color(nes.cpu.status & Flags::C))?;
    olc::draw_string(x, y + 10, format!("PCounter    : {:04X?}", nes.cpu.program_counter).as_str(), olc::WHITE)?;
    olc::draw_string(x, y + 50, format!("Stack ptr   : {:02X?}", nes.cpu.stack_pointer).as_str(), olc::WHITE)?;
    olc::draw_string(x, y + 20, format!("Accumulator : {:02X?} [{:?}]", nes.cpu.acc, nes.cpu.acc).as_str(), olc::WHITE)?;
    olc::draw_string(x, y + 30, format!("X Register  : {:02X?} [{:?}]", nes.cpu.x_reg, nes.cpu.x_reg).as_str(), olc::WHITE)?;
    olc::draw_string(x, y + 40, format!("Y Register  : {:02X?} [{:?}]", nes.cpu.y_reg, nes.cpu.y_reg).as_str(), olc::WHITE)?;
    Ok(())
}

fn draw_ram(x: i32, y: i32, col: i32, addr: u16, nes: &mut Nes) -> Result<(), olc::Error> {
    if (addr..=(addr + (col * 16) as u16)).contains(&nes.cpu.program_counter) {
        let current_x = x + (6 * 9) + ((nes.cpu.program_counter % 16) * 16) as i32 + ((nes.cpu.program_counter % 16) * 8) as i32;
        let current_y = y + (((nes.cpu.program_counter - addr) / 16) * 10) as i32 -1;
        olc::draw_rect(current_x, current_y, 17, 9, olc::GREEN);
    }
    for i in 0..col {
        let mut line = format!("${:04X?}:", addr + (i * 16) as u16);
        for j in 0..16 {
            line.push_str(format!(" {:02X?}", nes.cpu.read_bus(addr + (16 * i) as u16 + j)).as_str());
        }
        olc::draw_string(x, y + (10 * i), &line, olc::WHITE)?;
    }
    Ok(())
}

fn draw_code(x: i32, y: i32, num_lines: usize, nes: &mut Nes) -> Result<(), olc::Error> {
    let bound = num_lines / 2;
    let ins: Vec<HumanInstruction> = match &nes.decoded_rom {
        Some(rom) =>  {  
            get_rom_instructions_from_range(nes.cpu.program_counter, bound, rom)
        },
        None => { return Ok(());}
    };

    olc::draw_string(x, y - 10, "Addr   Ins Mode     Hex  Dec", olc::WHITE);

    for (idx, elem) in ins.iter().enumerate() {
        //  🤮
        let mut fb: Vec<u8> = elem.following_bytes.iter().filter(|x| x != &&i16::MIN).map(|x| *x as u8).collect::<Vec<u8>>();
        let s: String = match fb.len() {
            0 => format!("0x{:04X?} {} [{}] ->      ", elem.location, elem.name,elem.addr_mode),
            1 => {
                let byte = fb.pop().unwrap();
                format!("0x{:04X?} {} [{}] ->   {:02X?} {:?}", elem.location, elem.name, elem.addr_mode, byte, byte)
            },
            2 => {
                let byte = ((fb.pop().unwrap() as u16) << 8) | (fb.pop().unwrap() as u16);
                format!("0x{:04X?} {} [{}] -> {:04X?} {:?} ", elem.location, elem.name, elem.addr_mode, byte, byte as i16)
            },
            _ => "".to_string(),
        };
        let color = match elem.location {
            x if x == nes.cpu.program_counter => olc::GREEN,
            _ => olc::WHITE,
        };
        olc::draw_string(x, y + (idx * 10) as i32, s.as_str(), color);
    }

    Ok(())
}
