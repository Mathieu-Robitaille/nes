pub mod ppu_consts {

    use crate::consts::emulation_consts::COLOR_CHANNELS;
    use crate::consts::screen_consts::{HEIGHT, WIDTH};

    pub type NameTableT = [[u8; NAME_TABLE_SIZE]; 2];
    pub type PatternTableT = [[u8; PATTERN_TABLE_SIZE]; 2];
    pub type PaletteT = [u8; 32];
    pub type ScreenT = [u8; NUM_CYCLES_PER_SCANLINE * NUM_SCANLINES_RENDERED * COLOR_CHANNELS];
    pub type SprScreenT = [u8; WIDTH * HEIGHT * COLOR_CHANNELS];
    pub type SprNameTableT = [SprScreenT; 2];
    pub type SprPatternTableUnitT =
        [u8; SPR_PATTERN_TABLE_SIZE * SPR_PATTERN_TABLE_SIZE * COLOR_CHANNELS];
    pub type SprPatternTableT = [SprPatternTableUnitT; 2];

    pub const NUM_CYCLES_PER_SCANLINE: usize = 341;
    pub const NUM_SCANLINES_RENDERED: usize = 261; // -1 .. = 261

    pub const STARTING_SCANLINE: usize = 241;

    pub const OAM_SIZE: usize = 64;
    pub const SPR_PATTERN_TABLE_SIZE: usize = 128;
    pub const PATTERN_TABLE_SIZE: usize = 4096;
    pub const NAME_TABLE_SIZE: usize = 1024;

    // Status
    pub const STATUS_UNUSED_MASK:         u8 = 0b0001_1111;
    pub const STATUS_SPRT_OVERFLOW_MASK:  u8 = 0b0010_0000;
    pub const STATUS_SPRT_HIT_ZERO_MASK:  u8 = 0b0100_0000;
    pub const STATUS_VERTICAL_BLANK_MASK: u8 = 0b1000_0000;

    // Mask
    pub const MASK_GRAYSCALE:              u8 = 0b0000_0001;
    pub const MASK_RENDER_BACKGROUND_LEFT: u8 = 0b0000_0010;
    pub const MASK_RENDER_SPRITES_LEFT:    u8 = 0b0000_0100;
    pub const MASK_RENDER_BACKGROUND:      u8 = 0b0000_1000;
    pub const MASK_RENDER_SPRITES:         u8 = 0b0001_0000;
    pub const MASK_ENHANCE_RED:            u8 = 0b0010_0000;
    pub const MASK_ENHANCE_GREEN:          u8 = 0b0100_0000;
    pub const MASK_ENHANCE_BLUE:           u8 = 0b1000_0000;

    // CTRL
    pub const CTRL_NAMETABLE_X:        u8 = 0b0000_0001;
    pub const CTRL_NAMETABLE_Y:        u8 = 0b0000_0010;
    pub const CTRL_INCREMENT_MODE:     u8 = 0b0000_0100;
    pub const CTRL_PATTERN_SPRITE:     u8 = 0b0000_1000;
    pub const CTRL_PATTERN_BACKGROUND: u8 = 0b0001_0000;
    pub const CTRL_SPRITE_SIZE:        u8 = 0b0010_0000;
    pub const CTRL_SLAVE_MODE:         u8 = 0b0100_0000;
    pub const CTRL_ENABLE_NMI:         u8 = 0b1000_0000;

    // PPU Register
    pub const REG_COARSE_X:    u16 = 0b0000_0000_0001_1111;
    pub const REG_COARSE_Y:    u16 = 0b0000_0011_1110_0000;
                               //           0100_0000_0000
    pub const REG_NAMETABLE_X: u16 = 0b0000_0100_0000_0000;
    pub const REG_NAMETABLE_Y: u16 = 0b0000_1000_0000_0000;
    pub const REG_FINE_Y:      u16 = 0b0111_0000_0000_0000;
    pub const REG_UNUSED:      u16 = 0b1000_0000_0000_0000;

    // 00111111
    // 01000000
    /// https://www.nesdev.org/wiki/PPU_power_up_state
    /// It seems the ppu ignores writes to the control register if
    /// the cpu clock count is under this value
    ///
    /// Since the ppu clocks 3 times per cpu clock
    /// I am going to try to resolve this by adding
    /// an internal clock counter to the ppu and only accepting writes
    /// once this value is passed.
    #[allow(unused)]
    pub const PPU_CTRL_IGNORE_CPU_CYCLES: usize = 29025;
    pub const PPU_CTRL_IGNORE_CYCLES: usize = PPU_CTRL_IGNORE_CPU_CYCLES * 3;
}

pub mod debug_consts {
    use crate::consts::ppu_consts::{SPR_PATTERN_TABLE_SIZE, NUM_SCANLINES_RENDERED, NUM_CYCLES_PER_SCANLINE};
    use imgui::TableFlags;
    /*  */
    pub const PADDING_SIZE: f32 = 10f32;

    /// Cpu flags for winit
    pub const CPU_X_POS: f32 = CODE_X + CODE_SIZE_X + PADDING_SIZE;
    pub const CPU_Y_POS: f32 = PADDING_SIZE;
    pub const CPU_POS: [f32; 2] = [CPU_X_POS, CPU_Y_POS];

    pub const CPU_X_SIZE: f32 = 200f32;
    pub const CPU_Y_SIZE: f32 = 200f32;
    pub const CPU_SIZE: [f32; 2] = [CPU_X_SIZE, CPU_Y_SIZE];

    pub const CPU_RESIZEABLE: bool = false;

    // pub const CPU_TABLE_FLAGS: TableFlags = TableFlags::REORDERABLE | TableFlags::HIDEABLE | TableFlags::RESIZABLE | TableFlags::NO_BORDERS_IN_BODY;
    pub const CPU_TABLE_FLAGS: u32 = 1 | 2 | 4 | 2048; /* Its basically the same thing */

    // pub const CPU_:  = ;

    pub const RAM_X_POS: f32 = 0f32;
    pub const RAM_Y_POS: f32 = 2f32;

    pub const PPU_STATUS_X_POS: f32 = CPU_X_POS;
    pub const PPU_STATUS_Y_POS: f32 = 100f32;

    const PPU_SCREEN_X: f32 = PADDING_SIZE;
    const PPU_SCREEN_Y: f32 = PADDING_SIZE;
    pub const PPU_SCREEN_POS: [f32; 2] = [PPU_SCREEN_X, PPU_SCREEN_Y];

    const PPU_SCREEN_X_SIZE: f32 = NUM_CYCLES_PER_SCANLINE as f32;
    const PPU_SCREEN_Y_SIZE: f32 = NUM_SCANLINES_RENDERED as f32;
    pub const PPU_SCREEN_SIZE: [f32; 2] = [PPU_SCREEN_X_SIZE, PPU_SCREEN_Y_SIZE];

    const PPU_GAME_WINDOW_X_SIZE: f32 = PPU_SCREEN_X_SIZE + (PADDING_SIZE * 2f32);
    const PPU_GAME_WINDOW_Y_SIZE: f32 = PPU_SCREEN_Y_SIZE + (PADDING_SIZE * 4f32);
    pub const PPU_GAME_WINDOW_SIZE: [f32; 2] = [PPU_GAME_WINDOW_X_SIZE, PPU_GAME_WINDOW_Y_SIZE];

    const PPU_STATUS_WINDOW_X_POS: f32 = CPU_X_POS;
    const PPU_STATUS_WINDOW_Y_POS: f32 = CPU_Y_POS + CPU_Y_SIZE + PADDING_SIZE;
    pub const PPU_STATUS_WINDOW_POS: [f32; 2] = [PPU_STATUS_WINDOW_X_POS, PPU_STATUS_WINDOW_Y_POS];

    const PPU_STATUS_WINDOW_X_SIZE: f32 = 250f32;
    const PPU_STATUS_WINDOW_Y_SIZE: f32 = CPU_Y_SIZE;
    pub const PPU_STATUS_WINDOW_SIZE: [f32; 2] =
        [PPU_STATUS_WINDOW_X_SIZE, PPU_STATUS_WINDOW_Y_SIZE];
    pub const PPU_RESIZEABLE: bool = false;

    const PPU_PALLET_WINDOW_X: f32 = CODE_X;
    const PPU_PALLET_WINDOW_Y: f32 = CODE_Y + CODE_SIZE_Y + PADDING_SIZE;
    pub const PPU_PALLET_WINDOW_POS: [f32; 2] = [PPU_PALLET_WINDOW_X, PPU_PALLET_WINDOW_Y];
    pub const PPU_PALLET_IMAGE_SIZE: [f32; 2] =
        [SPR_PATTERN_TABLE_SIZE as f32, SPR_PATTERN_TABLE_SIZE as f32];
    pub const PPU_PALLET_WINDOW_SIZE: [f32; 2] = [
        SPR_PATTERN_TABLE_SIZE as f32 * 2f32 + PADDING_SIZE * 2f32,
        SPR_PATTERN_TABLE_SIZE as f32 + PADDING_SIZE * 5f32,
    ];

    pub const CODE_X: f32 = PPU_SCREEN_X + PPU_GAME_WINDOW_X_SIZE + PADDING_SIZE;
    pub const CODE_Y: f32 = PADDING_SIZE;
    pub const CODE_POS: [f32; 2] = [CODE_X, CODE_Y];

    pub const CODE_SIZE_X: f32 = 300f32;
    pub const CODE_SIZE_Y: f32 = CPU_Y_SIZE;
    pub const CODE_SIZE: [f32; 2] = [CODE_SIZE_X, CODE_SIZE_Y];

    // Emulation section
    pub const EMULATION_CONTROLS_X_POS: f32 = PPU_SCREEN_X;
    pub const EMULATION_CONTROLS_Y_POS: f32 = PPU_SCREEN_Y + PPU_GAME_WINDOW_Y_SIZE + PADDING_SIZE;
    pub const EMULATION_CONTROLS_POS: [f32; 2] =
        [EMULATION_CONTROLS_X_POS, EMULATION_CONTROLS_Y_POS];

    pub mod debug_color {
        pub const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        pub const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        pub const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
        pub const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
    }
}

pub mod screen_consts {
    pub const WIDTH: usize = 256;
    pub const HEIGHT: usize = 240;
}

pub mod render_consts {
    pub const TITLE: &'static str = "NES Emulator";
    pub const VSYNC: bool = true;

    pub const LOGICAL_WIDTH: f64 = 1024f64;
    pub const LOGICAL_HEIGHT: f64 = 768f64;
}

pub mod nes_consts {
    use crate::cartridge::Rom;

    pub const CART: Rom = Rom::DonkeyKong;
}

pub mod emulation_consts {
    use crate::emulator::FrameSync;
    pub const EMU_START_STATE: FrameSync = FrameSync::Stop;
    pub const EMU_DEBUG: bool = true;

    use glium::texture::ClientFormat;
    pub const COLOR_CHANNELS: usize = 3;
    pub const CLIENT_FORMAT: ClientFormat = ClientFormat::U8U8U8;
}
