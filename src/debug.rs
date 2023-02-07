use crate::consts::{
    debug_consts::*,
    emulation_consts::{CPU_DEBUG, EMU_DEBUG},
    ppu_consts::*,
};
use crate::cpu::CPUFlags;
use crate::emulator::{EmulationState, FrameSync};
use crate::nes::Nes;

use imgui::*;


use std::time::Instant;

///
/// The goal with this is to have everything controlled in debug consts
/// Maybe later I can add a yaml file or something to read/write positions
///

pub fn draw_debug(nes: &mut Nes, state: &mut EmulationState, ui: &Ui) {
    draw_cpu(nes, ui);
    draw_ppu_buffer(nes, state, ui);
    draw_ppu_tables(nes, state, ui);
    emulation_control(nes, state, ui);
    draw_ppu_status(nes, ui);

    if PPU_NAME_TABLE_WINDOW_ENABLE {
        draw_ppu_name_tables(nes, ui);
    }

    draw_oam(nes, ui);
    // draw_ram("Stack".to_string(), 16, 0x100, nes, ui);
    if CPU_DEBUG {
        draw_ram("Debug Results".to_string(), 16, 0x0000, nes, ui);
    }

    draw_code(10, nes, ui);
}

fn emulation_control(nes: &mut Nes, state: &mut EmulationState, ui: &Ui) {
    ui.window("Emulation Control.")
        .position(EMULATION_CONTROLS_POS, Condition::Appearing)
        .build(|| {
            let mut cycles: String = String::new();
            let mut watch_addr: String = String::new();
            ui.text(format!(
                "FPS: {:.02}",
                1f32 / state.last_frame_time.elapsed().as_secs_f32()
            ));
            state.last_frame_time = Instant::now();

            ui.separator();
            ui.text("Run without stopping.");
            if ui.button("Normal") {
                state.frame_sync = FrameSync::Run;
            };
            ui.same_line();
            if ui.button("Clock cycle") {
                state.frame_sync = FrameSync::OneCycle;
            };
            ui.same_line();
            if ui.button("Scanline") {
                // state.frame_sync = FrameSync::OneCycle;
            };

            ui.separator();
            ui.text("Run then stop.");
            if ui.button("Cycle") {
                state.frame_sync = FrameSync::StepOneCycle;
            };
            ui.same_line();
            if ui.button("Instruction") {
                state.frame_sync = FrameSync::OneInstruction;
            };
            ui.same_line();
            if ui.button("Scanline") {
                state.frame_sync = FrameSync::OneScanline;
            };
            ui.same_line();
            if ui.button("Frame") {
                state.frame_sync = FrameSync::OneFrame;
            };
            ui.separator();
            ui.text("PPU Control");
            if ui.button("Inc palette") {
                state.increment_palette_id();
            }
            ui.same_line();
            if ui.button("Dec palette") {
                state.decrement_palette_id();
            }

            ui.separator();
            ui.text("Manual breakpoint set");
            if ui
                .input_text("Ins", &mut cycles)
                .enter_returns_true(true)
                .chars_decimal(true)
                .hint("Run X Instructions.")
                .build()
            {
                state.frame_sync = FrameSync::XCycles;
                state.cycles = cycles.parse::<usize>().unwrap_or(0);
            };
            if ui
                .input_text("PC Watch", &mut watch_addr)
                .enter_returns_true(true)
                .hint("Set PC Watch.") /* halt execution when the program counter hits this point */
                .build()
            {
                state.frame_sync = FrameSync::PCWatch;
                state.watch_addr = u16::from_str_radix(&watch_addr, 16).unwrap_or(0);
            };

            ui.separator();
            if ui.button("Stop.") {
                state.frame_sync = FrameSync::Stop;
            };
            ui.same_line();
            if ui.button("Reset.") {
                state.frame_sync = FrameSync::Reset;
            };
        });
}

fn draw_cpu(nes: &mut Nes, ui: &Ui) {
    ui.window("CPU Debug Info")
        .position(CPU_POS, Condition::Always)
        .size(CPU_SIZE, Condition::Always)
        .resizable(CPU_RESIZABLE)
        .scroll_bar(CPU_SCROLLBAR)
        .collapsible(CPU_COLLAPSIBLE)
        .build(|| {
            {
                fn f(ui: &Ui, cond: u8, c: &'static str) {
                    ui.same_line();
                    ui.text_colored(color(cond > 0), c);
                }
                ui.text("Status:");

                f(ui, nes.cpu.status & CPUFlags::N, "N");
                f(ui, nes.cpu.status & CPUFlags::V, "V");
                f(ui, nes.cpu.status & CPUFlags::U, "U");
                f(ui, nes.cpu.status & CPUFlags::B, "B");
                f(ui, nes.cpu.status & CPUFlags::D, "D");
                f(ui, nes.cpu.status & CPUFlags::I, "I");
                f(ui, nes.cpu.status & CPUFlags::Z, "Z");
                f(ui, nes.cpu.status & CPUFlags::C, "C");
            }
            if let Some(_t) = ui.begin_table_header(
                "table-headers",
                [
                    TableColumnSetup::new("Register"),
                    TableColumnSetup::new("Value"),
                ],
            ) {
                ui.table_next_column();
                ui.text("PC: ");

                ui.table_next_column();
                ui.text(format!("{:04X}", nes.cpu.pc));

                ui.table_next_column();
                ui.text("A: ");

                ui.table_next_column();
                ui.text(format!("{:02X}", nes.cpu.acc));

                ui.table_next_column();
                ui.text("X: ");

                ui.table_next_column();
                ui.text(format!("{:02X}", nes.cpu.x_reg));

                ui.table_next_column();
                ui.text("Y: ");

                ui.table_next_column();
                ui.text(format!("{:02X}", nes.cpu.y_reg));

                ui.table_next_column();
                ui.text("SP: ");

                ui.table_next_column();
                ui.text(format!("{:02X}", nes.cpu.stack_pointer));

                ui.table_next_column();
                ui.text("F: ");

                ui.table_next_column();
                ui.text(format!("{:02X}", nes.cpu.fetched));

                ui.table_next_column();
                ui.text("#I: ");

                ui.table_next_column();
                ui.text(format!("{:}", nes.cpu.instruction_count));

                ui.table_next_column();
                ui.text("Clock: ");

                ui.table_next_column();
                ui.text(format!("{:}", nes.cpu.clock_count));
            }
        });
}

fn draw_ram(title: String, rows: usize, addr: u16, nes: &mut Nes, ui: &Ui) {
    ui.window(title)
        /* Fix me*/
        .size([800f32, 400f32], Condition::Appearing)
        .position([150f32, 400f32], Condition::Appearing)
        .build(|| {
            if let Some(_t) = ui.begin_table_header(
                "table-headers",
                [
                    TableColumnSetup::new("Page"),
                    TableColumnSetup::new("00"),
                    TableColumnSetup::new("01"),
                    TableColumnSetup::new("02"),
                    TableColumnSetup::new("03"),
                    TableColumnSetup::new("04"),
                    TableColumnSetup::new("05"),
                    TableColumnSetup::new("06"),
                    TableColumnSetup::new("07"),
                    TableColumnSetup::new("08"),
                    TableColumnSetup::new("09"),
                    TableColumnSetup::new("0A"),
                    TableColumnSetup::new("0B"),
                    TableColumnSetup::new("0C"),
                    TableColumnSetup::new("0D"),
                    TableColumnSetup::new("0E"),
                    TableColumnSetup::new("0F"),
                ],
            ) {
                // ui.table_next_column();
                let root_addr = addr & !0x000F;
                let bottom = root_addr + (rows as u16) * 0xF0;
                for row in (0..(rows * 0x10)).step_by(0x10) {
                    let row16 = row as u16;
                    ui.table_next_column();
                    ui.text(format!("{:04X}", root_addr + row16));
                    for col in 0..=0x0F {
                        let col16 = col as u16;
                        ui.table_next_column();
                        ui.text(format!(
                            "{:02X}",
                            nes.cpu.bus.cpu_read(root_addr + row16 + col16, true)
                        ));
                    }
                }
            }
        });
}

fn draw_code(num_lines: usize, nes: &mut Nes, ui: &Ui) {
    let lines = {
        let start: usize = nes.cpu.pc as usize;
        let end: usize = start + num_lines;
        let mut r: Vec<(u16, &String)> = vec![];
        for i in start..end {
            let i_16 = i as u16;
            if let Some(x) = nes.decoded_rom.get(&i_16) {
                r.push((i_16, x));
            }
        }
        r
    };
    ui.window("Cpu Instructions")
        .position(CODE_POS, CODE_POSITION_COND)
        .size(CODE_SIZE, CODE_SIZE_COND)
        .resizable(CODE_RESIZABLE)
        .scroll_bar(CODE_SCROLLBAR)
        .collapsible(CODE_COLLAPSIBLE)
        .build(|| {
            if let Some(_t) =
                ui.begin_table_header("table-headers", [TableColumnSetup::new("Instruction")])
            {
                for (addr, instruction) in lines.iter() {
                    ui.table_next_column();
                    if addr == &nes.cpu.pc {
                        ui.text_colored(debug_color::GREEN, instruction);
                    } else {
                        ui.text(instruction);
                    }
                }
            }
        });
}

fn draw_ppu_buffer(nes: &mut Nes, state: &EmulationState, ui: &Ui) {
    ui.window("NES")
        .position(PPU_SCREEN_POS, PPU_SCREEN_POSITION_COND)
        .size(PPU_GAME_WINDOW_SIZE, PPU_SCREEN_SIZE_COND)
        .resizable(PPU_SCREEN_RESIZABLE)
        .scroll_bar(PPU_SCREEN_SCROLLBAR)
        .collapsible(PPU_SCREEN_COLLAPSIBLE)
        .build(|| {
            if let Some(tex_id) = state.nes_texture_id {
                Image::new(tex_id, PPU_SCREEN_SIZE).build(ui);
            } else {
                ui.text("DA MONKE AR WORG");
            }
        });
}

fn draw_ppu_tables(nes: &mut Nes, state: &EmulationState, ui: &Ui) {
    ui.window("PPU Sprite sheets")
        .position(PPU_PALLET_WINDOW_POS, PPU_NAME_TABLE_WINDOW_POSITION_COND)
        .size(PPU_PALLET_WINDOW_SIZE, PPU_NAME_TABLE_WINDOW_SIZE_COND)
        .resizable(PPU_NAME_TABLE_WINDOW_RESIZABLE)
        .scroll_bar(PPU_NAME_TABLE_WINDOW_SCROLLBAR)
        .collapsible(PPU_NAME_TABLE_WINDOW_COLLAPSIBLE)
        .build(|| {
            if let Some(debug_tex) = &state.debug_textures {
                ui.text("Here's some palettes!");
                {
                    let draw_list = ui.get_window_draw_list();
                    ui.invisible_button("Boring Button", PPU_PALLET_IMAGE_SIZE);
                    draw_list
                        .add_image(
                            debug_tex.palette_one,
                            ui.item_rect_min(),
                            ui.item_rect_max(),
                        )
                        .build();
                }
                {
                    ui.same_line();
                    let draw_list = ui.get_window_draw_list();
                    ui.invisible_button("Boring Button", PPU_PALLET_IMAGE_SIZE);
                    draw_list
                        .add_image(
                            debug_tex.palette_two,
                            ui.item_rect_min(),
                            ui.item_rect_max(),
                        )
                        .build();
                }
            }
        });
}

fn draw_oam(nes: &mut Nes, ui: &Ui) {
    ui.window("OAM")
        .position(OAM_WINDOW_POS, OAM_POSITION_COND)
        .size(OAM_WINDOW_SIZE, OAM_SIZE_COND)
        .resizable(OAM_RESIZABLE)
        .scroll_bar(OAM_SCROLLBAR)
        .collapsible(OAM_COLLAPSIBLE)
        .build(|| {
            if let Some(_t) = ui.begin_table_header(
                "table-headers",
                [
                    TableColumnSetup::new("ID"),
                    TableColumnSetup::new("x"),
                    TableColumnSetup::new("y"),
                    TableColumnSetup::new("id"),
                    TableColumnSetup::new("attr"),
                ],
            ) {
                for (id, sprite) in nes.get_oam().iter().enumerate() {
                    ui.table_next_column();
                    ui.text(format!("{:}", id));

                    ui.table_next_column();
                    ui.text(format!("{:02X}", sprite.x));

                    ui.table_next_column();
                    ui.text(format!("{:02X}", sprite.y));

                    ui.table_next_column();
                    ui.text(format!("{:02X}", sprite.id));

                    ui.table_next_column();
                    ui.text(format!("{:02X}", sprite.attribute));
                }
            }
        });
}

fn draw_ppu_status(nes: &mut Nes, ui: &Ui) {
    ui.window("PPU Debug Info")
        .position(PPU_STATUS_WINDOW_POS, PPU_STATUS_POSITION_COND)
        .size(PPU_STATUS_WINDOW_SIZE, PPU_STATUS_SIZE_COND)
        .resizable(PPU_STATUS_RESIZABLE)
        .scroll_bar(PPU_STATUS_SCROLLBAR)
        .collapsible(PPU_STATUS_COLLAPSIBLE)
        .build(|| {
            {
                fn f(ui: &Ui, cond: u8, c: &'static str) {
                    ui.same_line();
                    ui.text_colored(color(cond > 0), c);
                }
                ui.text("Status: ");

                let status = nes.get_ppu_status();
                let unused_status = format!("{:0>5b}", status & STATUS_UNUSED_MASK);

                f(ui, status & STATUS_VERTICAL_BLANK_MASK, "VB");
                f(ui, status & STATUS_SPRT_HIT_ZERO_MASK, "HZ");
                f(ui, status & STATUS_SPRT_OVERFLOW_MASK, "O");
                ui.same_line();
                ui.text_colored(debug_color::GREEN, unused_status);

                ui.text("Control:");

                let ctrl = nes.get_ppu_ctrl();
                f(ui, ctrl & CTRL_NAMETABLE_X, "NX");
                f(ui, ctrl & CTRL_NAMETABLE_Y, "NY");
                f(ui, ctrl & CTRL_INCREMENT_MODE, "IM");
                f(ui, ctrl & CTRL_PATTERN_SPRITE, "PS");
                f(ui, ctrl & CTRL_PATTERN_BACKGROUND, "PB");
                f(ui, ctrl & CTRL_SPRITE_SIZE, "SS");
                f(ui, ctrl & CTRL_SLAVE_MODE, "SM");
                f(ui, ctrl & CTRL_ENABLE_NMI, "NMI");

                ui.text("Mask:");

                let mask = nes.get_ppu_mask();
                f(ui, mask & MASK_GRAYSCALE, "G");
                f(ui, mask & MASK_RENDER_BACKGROUND_LEFT, "BG-L");
                f(ui, mask & MASK_RENDER_SPRITES_LEFT, "SPR-L");
                f(ui, mask & MASK_RENDER_BACKGROUND, "BG");
                f(ui, mask & MASK_RENDER_SPRITES, "SPR");
                f(ui, mask & MASK_ENHANCE_RED, "R");
                f(ui, mask & MASK_ENHANCE_GREEN, "G");
                f(ui, mask & MASK_ENHANCE_BLUE, "B");
            }
            if let Some(_t) = ui.begin_table_header(
                "table-headers",
                [
                    TableColumnSetup::new("Register"),
                    TableColumnSetup::new("Value"),
                ],
            ) {
                let vram = nes.get_ppu_vram();

                ui.table_next_column();
                ui.text("Scanline: ");

                ui.table_next_column();
                ui.text(format!("{:}", nes.get_ppu_scanline()));

                ui.table_next_column();
                ui.text("Cycle: ");

                ui.table_next_column();
                ui.text(format!("{:}", nes.get_ppu_cycle()));

                ui.table_next_column();
                ui.text("Vram: ");

                ui.table_next_column();
                ui.text(format!("{:0>16b}", vram));

                ui.table_next_column();
                ui.text("Coarse X: ");

                ui.table_next_column();
                ui.text(format!(
                    "  {:}",
                    (vram & REG_COARSE_X) >> REG_COARSE_X.trailing_zeros()
                ));

                ui.table_next_column();
                ui.text("Coarse Y: ");

                ui.table_next_column();
                ui.text(format!(
                    "  {:}",
                    (vram & REG_COARSE_Y) >> REG_COARSE_Y.trailing_zeros()
                ));

                ui.table_next_column();
                ui.text("Nametable X: ");

                ui.table_next_column();
                ui.text(format!(
                    "  {:}",
                    (vram & REG_NAMETABLE_X) >> REG_NAMETABLE_X.trailing_zeros()
                ));

                ui.table_next_column();
                ui.text("Nametable Y: ");

                ui.table_next_column();
                ui.text(format!(
                    "  {:}",
                    (vram & REG_NAMETABLE_Y) >> REG_NAMETABLE_Y.trailing_zeros()
                ));

                ui.table_next_column();
                ui.text("Fine X: ");

                ui.table_next_column();
                ui.text(format!("  {:}", nes.get_ppu_fine_x()));

                ui.table_next_column();
                ui.text("Fine Y: ");

                ui.table_next_column();
                ui.text(format!(
                    "  {:}",
                    (vram & REG_FINE_Y) >> REG_FINE_Y.trailing_zeros()
                ));

                ui.table_next_column();
                ui.text("Tram: ");

                ui.table_next_column();
                ui.text(format!("{:0>16b}", nes.get_ppu_tram()));

                ui.table_next_column();
                ui.text("Frame: ");

                ui.table_next_column();
                ui.text(format!("{:}", nes.get_ppu_frame_count()));
            }
        });
}

fn draw_ppu_name_tables(nes: &mut Nes, ui: &Ui) {
    ui.window("Name Tables")
        .size(PPU_NAME_TABLE_WINDOW_SIZE, PPU_NAME_TABLE_WINDOW_SIZE_COND)
        .position(PPU_NAME_TABLE_WINDOW_POS, PPU_NAME_TABLE_WINDOW_POSITION_COND)
        .resizable(PPU_NAME_TABLE_WINDOW_RESIZABLE)
        .scroll_bar(PPU_NAME_TABLE_WINDOW_SCROLLBAR)
        .collapsible(PPU_NAME_TABLE_WINDOW_COLLAPSIBLE)
        .build(|| {
            if let Some(_t) = ui.begin_table_header(
                "table-headers",
                [
                    TableColumnSetup::new("0"),
                    TableColumnSetup::new("1"),
                    TableColumnSetup::new("2"),
                    TableColumnSetup::new("3"),
                    TableColumnSetup::new("4"),
                    TableColumnSetup::new("5"),
                    TableColumnSetup::new("6"),
                    TableColumnSetup::new("7"),
                    TableColumnSetup::new("8"),
                    TableColumnSetup::new("9"),
                    TableColumnSetup::new("10"),
                    TableColumnSetup::new("11"),
                    TableColumnSetup::new("12"),
                    TableColumnSetup::new("13"),
                    TableColumnSetup::new("14"),
                    TableColumnSetup::new("15"),
                    TableColumnSetup::new("16"),
                    TableColumnSetup::new("17"),
                    TableColumnSetup::new("18"),
                    TableColumnSetup::new("19"),
                    TableColumnSetup::new("20"),
                    TableColumnSetup::new("21"),
                    TableColumnSetup::new("22"),
                    TableColumnSetup::new("23"),
                    TableColumnSetup::new("24"),
                    TableColumnSetup::new("25"),
                    TableColumnSetup::new("26"),
                    TableColumnSetup::new("27"),
                    TableColumnSetup::new("28"),
                    TableColumnSetup::new("29"),
                    TableColumnSetup::new("30"),
                    TableColumnSetup::new("31"),
                ],
            ) {
                for x in nes.get_ppu_name_table(0) {
                    ui.table_next_column();
                    ui.text_colored(COLORS[(x % 64) as usize], format!("{:02X}", x));
                }
            }
        });
}

static COLORS: [[f32; 4]; 64] = [
    [0.3294f32, 0.3294f32, 0.3294f32, 1.0000f32],
    [0.0000f32, 0.1176f32, 0.4549f32, 1.0000f32],
    [0.0314f32, 0.0627f32, 0.5647f32, 1.0000f32],
    [0.1882f32, 0.0000f32, 0.5333f32, 1.0000f32],
    [0.2667f32, 0.0000f32, 0.3922f32, 1.0000f32],
    [0.3608f32, 0.0000f32, 0.1882f32, 1.0000f32],
    [0.3294f32, 0.0157f32, 0.0000f32, 1.0000f32],
    [0.2353f32, 0.0941f32, 0.0000f32, 1.0000f32],
    [0.1255f32, 0.1647f32, 0.0000f32, 1.0000f32],
    [0.0314f32, 0.2275f32, 0.0000f32, 1.0000f32],
    [0.0000f32, 0.2510f32, 0.0000f32, 1.0000f32],
    [0.0000f32, 0.2353f32, 0.0000f32, 1.0000f32],
    [0.0000f32, 0.1961f32, 0.2353f32, 1.0000f32],
    [0.0000f32, 0.0000f32, 0.0000f32, 1.0000f32],
    [0.0000f32, 0.0000f32, 0.0000f32, 1.0000f32],
    [0.0000f32, 0.0000f32, 0.0000f32, 1.0000f32],
    [0.5961f32, 0.5882f32, 0.5961f32, 1.0000f32],
    [0.0314f32, 0.2980f32, 0.7686f32, 1.0000f32],
    [0.1882f32, 0.1961f32, 0.9255f32, 1.0000f32],
    [0.3608f32, 0.1176f32, 0.8941f32, 1.0000f32],
    [0.5333f32, 0.0784f32, 0.6902f32, 1.0000f32],
    [0.6275f32, 0.0784f32, 0.3922f32, 1.0000f32],
    [0.5961f32, 0.1333f32, 0.1255f32, 1.0000f32],
    [0.4706f32, 0.2353f32, 0.0000f32, 1.0000f32],
    [0.3294f32, 0.3529f32, 0.0000f32, 1.0000f32],
    [0.1569f32, 0.4471f32, 0.0000f32, 1.0000f32],
    [0.0314f32, 0.4863f32, 0.0000f32, 1.0000f32],
    [0.0000f32, 0.4627f32, 0.1569f32, 1.0000f32],
    [0.0000f32, 0.4000f32, 0.4706f32, 1.0000f32],
    [0.0000f32, 0.0000f32, 0.0000f32, 1.0000f32],
    [0.0000f32, 0.0000f32, 0.0000f32, 1.0000f32],
    [0.0000f32, 0.0000f32, 0.0000f32, 1.0000f32],
    [0.9255f32, 0.9333f32, 0.9255f32, 1.0000f32],
    [0.2980f32, 0.6039f32, 0.9255f32, 1.0000f32],
    [0.4706f32, 0.4863f32, 0.9255f32, 1.0000f32],
    [0.6902f32, 0.3843f32, 0.9255f32, 1.0000f32],
    [0.8941f32, 0.3294f32, 0.9255f32, 1.0000f32],
    [0.9255f32, 0.3451f32, 0.7059f32, 1.0000f32],
    [0.9255f32, 0.4157f32, 0.3922f32, 1.0000f32],
    [0.8314f32, 0.5333f32, 0.1255f32, 1.0000f32],
    [0.6275f32, 0.6667f32, 0.0000f32, 1.0000f32],
    [0.4549f32, 0.7686f32, 0.0000f32, 1.0000f32],
    [0.2980f32, 0.8157f32, 0.1255f32, 1.0000f32],
    [0.2196f32, 0.8000f32, 0.4235f32, 1.0000f32],
    [0.2196f32, 0.7059f32, 0.8000f32, 1.0000f32],
    [0.2353f32, 0.2353f32, 0.2353f32, 1.0000f32],
    [0.0000f32, 0.0000f32, 0.0000f32, 1.0000f32],
    [0.0000f32, 0.0000f32, 0.0000f32, 1.0000f32],
    [0.9255f32, 0.9333f32, 0.9255f32, 1.0000f32],
    [0.6588f32, 0.8000f32, 0.9255f32, 1.0000f32],
    [0.7373f32, 0.7373f32, 0.9255f32, 1.0000f32],
    [0.8314f32, 0.6980f32, 0.9255f32, 1.0000f32],
    [0.9255f32, 0.6824f32, 0.9255f32, 1.0000f32],
    [0.9255f32, 0.6824f32, 0.8314f32, 1.0000f32],
    [0.9255f32, 0.7059f32, 0.6902f32, 1.0000f32],
    [0.8941f32, 0.7686f32, 0.5647f32, 1.0000f32],
    [0.8000f32, 0.8235f32, 0.4706f32, 1.0000f32],
    [0.7059f32, 0.8706f32, 0.4706f32, 1.0000f32],
    [0.6588f32, 0.8863f32, 0.5647f32, 1.0000f32],
    [0.5961f32, 0.8863f32, 0.7059f32, 1.0000f32],
    [0.6275f32, 0.8392f32, 0.8941f32, 1.0000f32],
    [0.6275f32, 0.6353f32, 0.6275f32, 1.0000f32],
    [0.0000f32, 0.0000f32, 0.0000f32, 1.0000f32],
    [0.0000f32, 0.0000f32, 0.0000f32, 1.0000f32],
];

fn color(f: bool) -> [f32; 4] {
    if f {
        return debug_color::GREEN;
    }
    debug_color::RED
}

