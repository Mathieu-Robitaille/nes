use crate::consts::{debug_consts::*, emulation_consts::EMU_DEBUG, ppu_consts::*};
use crate::cpu::CPUFlags;
use crate::emulator::{EmulationState, FrameSync};
use crate::nes::Nes;

use anyhow;
use glium::glutin;
use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::window::WindowBuilder;
use imgui::*;
use imgui::{Context, FontConfig, FontGlyphRanges, FontSource, Ui};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use rand::Rng;
use std::borrow::Cow;
use std::error::Error;
use std::io::Cursor;
use std::path::Path;
use std::rc::Rc;
use std::time::Instant;

pub fn draw_debug(nes: &mut Nes, state: &mut EmulationState, ui: &Ui) {
    draw_cpu(nes, ui);
    draw_ppu_buffer(nes, state, ui);
    draw_ppu_tables(nes, state, ui);
    emulation_control(nes, state, ui);
    // draw_oam(nes, ui);
    draw_code(10, nes, ui);
}

fn emulation_control(nes: &mut Nes, state: &mut EmulationState, ui: &Ui) {
    ui.window("EAT ASS.")
        .position(EMULATION_CONTROLS_POS, Condition::Appearing)
        .build(|| {
            let mut cycles: String = String::new();
            let mut watch_addr: String = String::new();
            ui.text(format!("FPS: {:.02}", 1f32 / state.last_frame_time.elapsed().as_secs_f32()));
            state.last_frame_time = Instant::now();
            if ui.button("Run one frame.") {
                state.frame_sync = FrameSync::OneFrame;
            };
            if ui.button("Run one cycle.") {
                state.frame_sync = FrameSync::OneCycle;
            };
            if ui.button("Run one instruction.") {
                state.frame_sync = FrameSync::OneInstruction;
            };

            if EMU_DEBUG { /* Ffffffffffffffffffffffffffffff */
                {
                    if ui.input_text("##", &mut cycles)
                        .enter_returns_true(true)
                        .chars_decimal(true)
                        .hint("Run X Instructions.")
                        .build() {
                            state.frame_sync = FrameSync::XCycles;
                            state.cycles = cycles.parse::<usize>().unwrap_or(0);
                        };
                }
    
                {
                    if ui.input_text("##", &mut watch_addr)
                        .enter_returns_true(true)
                        .hint("Set PC Watch.") /* halt execution when the program counter hits this point */ 
                        .build() {
                            state.frame_sync = FrameSync::XCycles;
                            state.watch_addr = watch_addr.parse::<u16>().unwrap_or(0);
                        };
                }
            }


            if ui.button("Stop.") {
                state.frame_sync = FrameSync::Stop;
            };
            if ui.button("Reset.") {
                state.frame_sync = FrameSync::Reset;
            };
            ui.separator();
            if ui.button("Print palette.") {
                println!("{:?}", nes.cpu.bus.ppu.palette);
            };
        });
}

fn draw_cpu(nes: &mut Nes, ui: &Ui) {
    fn color(f: bool) -> [f32; 4] {
        if f {
            return debug_color::GREEN;
        }
        debug_color::RED
    }

    ui.window("CPU Debug Info")
        .position(CPU_POS, Condition::Always)
        .size(CPU_SIZE, Condition::Always)
        .resizable(CPU_RESIZEABLE)
        .build(|| {
            {
                fn f(ui: &Ui, cond: u8, c: &'static str) {
                    ui.same_line();
                    ui.text_colored(color(cond > 0), c);
                }
                ui.text("Status: ");

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
            }
        });
}

fn draw_ram(rows: usize, addr: u16, nes: &mut Nes, ui: &Ui) {
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
        let root_addr = addr & !0x000F;
        let top = root_addr - (rows as u16) * 0xF;
        let bottom = root_addr + (rows as u16) * 0xF;
        for row in 0..rows {
            let row16 = row as u16;
            ui.table_next_column();
            ui.text(format!("{:04X}", top + (row16 * 0xF)));
            for col in 0..0x0F {
                let col16 = col as u16;
                ui.table_next_column();
                ui.text(format!(
                    "{:02X}",
                    nes.cpu.bus.cpu_read(root_addr + row16 + col16, true)
                ));
            }
        }
    }
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
        .position(CODE_POS, Condition::Appearing)
        .size(CODE_SIZE, Condition::Appearing)
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
        .position(PPU_SCREEN_POS, Condition::Always)
        .size(PPU_GAME_WINDOW_SIZE, Condition::Always)
        .scroll_bar(false)
        .resizable(false)
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
        .position(PPU_PALLET_WINDOW_POS, Condition::Appearing)
        .size(PPU_PALLET_WINDOW_SIZE, Condition::Appearing)
        .scroll_bar(false)
        .resizable(false)
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
        .size([200f32, 600f32], Condition::Appearing)
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

// fn draw_ppu_bg_ids(x: i32, y: i32, nes: &mut Nes) {
//     for y_iter in 0..30 {
//         for x_iter in 0..32 {
//             let s = format!(
//                 "{:02X?}",
//                 nes.cpu.bus.ppu.name_table[0][(y * 32 + x) as usize]
//             );
//             olc::draw_string(x_iter * 16, y_iter * 16, s.as_str(), olc::WHITE);
//         }
//     }
// }

// fn draw_swatches(x: i32, y: i32, nes: &mut Nes) {
//     let swatch_size: i32 = 6;
//     for p in 0..8 {
//         for s in 0..4 {
//             olc::fill_rect(
//                 x + p * (swatch_size * 5) + s * swatch_size,
//                 y,
//                 swatch_size,
//                 swatch_size,
//                 nes.cpu.bus.ppu.get_color_from_palette_ram(p as u8, s as u8),
//             );
//         }
//     }
// }

// // Dumps the listed instructions
// pub fn dump_code(nes: &mut Nes) {
//     let mut keys: Vec<&u16> = nes.decoded_rom.keys().collect::<Vec<&u16>>();
//     keys.sort();
//     for key in keys {
//         if let Some(x) = nes.decoded_rom.get(key) {
//             println!("{:?}", x);
//         }
//     }
// }
