use std::{cell::RefCell, fs::File, io, io::prelude::*, mem, rc::Rc, slice};

use crate::mapper::{Mapper000, MapperTrait};

#[derive(Debug, Clone, Copy)]
pub enum MIRROR {
    HORIZONTAL,
    VERTICAL,
    OnescreenLo,
    OnescreenHi,
}

pub struct Cartridge {
    mapper: Box<dyn MapperTrait>,
    prg_memory: Vec<u8>,
    chr_memory: Vec<u8>,
    mapper_id: u8,
    prg_banks: u8,
    chr_banks: u8,
    pub mirror: MIRROR,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct CartHeadder {
    name: [u8; 4],
    prg_rom_chunks: u8,
    chr_rom_chunks: u8,
    mapper1: u8,
    mapper2: u8,
    prg_ram_size: u8,
    tv_system1: u8,
    tv_system2: u8,
    unused: [u8; 5],
}

impl Cartridge {
    pub fn from(pick: Rom) -> io::Result<Cartridge> {
        let cart_name = match pick {
            // https://www.nesdev.org/wiki/Emulator_tests
            Rom::CPUTest => "test-roms/cpu/nestest.nes".to_string(),
            Rom::PpuColorTest => "test-roms/ppu/color_test.nes".to_string(),
            Rom::Mario => "test-roms/carts/super_mario.nes".to_string(),
            Rom::DonkeyKong => "test-roms/carts/Donkey Kong (World) (Rev A).nes".to_string(),
        };
        Ok(Cartridge::new(cart_name)?)
    }

    pub fn new(file_name: String) -> Result<Self, io::Error> {
        let mut f: File = File::open(file_name)?;
        let mut headder: CartHeadder = unsafe { mem::zeroed() };

        let headder_size = mem::size_of::<CartHeadder>();
        // javid makes a fuckin mistake
        // how could he not know that the size of [char; x] != [u8; x] in rust
        // unbelieeeeeeeeeeeeeeeeeevable.

        unsafe {
            let headder_slice =
                slice::from_raw_parts_mut(&mut headder as *mut _ as *mut u8, headder_size);
            f.read_exact(headder_slice).unwrap();
        }

        // print!("Name: ");
        // for i in headder.name {
        //     print!("{:02X?} ", i);
        // }
        // print!("\n");
        // println!("PRG ROM CHUNKS: {:02X?}", headder.prg_rom_chunks);
        // println!("CHR ROM CHUNKS: {:02X?}", headder.chr_rom_chunks);
        // println!("MAPPER 1: {:02X?}", headder.mapper1);
        // println!("MAPPER 2: {:02X?}", headder.mapper2);
        // println!("prg_ram_size: {:02X?}", headder.prg_ram_size);
        // println!("tv_system1: {:02X?}", headder.tv_system1);
        // println!("tv_system2: {:02X?}", headder.tv_system2);
        // println!("unused: {:?}", headder.unused);

        if headder.mapper1 & 0x04 > 0 {
            f.seek(io::SeekFrom::Current(512))?; // BAD SEEK!
        }
        let mapper_id = (headder.mapper2 & 0b11110000) | (headder.mapper1 >> 4);

        // True for vertical
        // false for horizontal
        let mirror = match (headder.mapper1 & 0x81) > 0 {
            true => MIRROR::VERTICAL,
            false => MIRROR::HORIZONTAL,
        };

        let mut prg_memory: Vec<u8> = vec![];
        let mut chr_memory: Vec<u8> = vec![];

        let file_type = 1;
        match file_type {
            // 0 => { 0 },
            1 => {
                prg_memory.resize((headder.prg_rom_chunks as usize) * 16384, 0);
                f.read_exact(&mut prg_memory)?;

                if headder.chr_rom_chunks == 0 {
                    chr_memory.resize(8192, 0);
                } else {
                    chr_memory.resize((headder.chr_rom_chunks as usize) * 8192, 0);
                }
                f.read_exact(&mut chr_memory)?;
            }
            // 2 => { 0 },
            _ => {}
        }

        let cart = Cartridge {
            mapper: Box::new(Mapper000::new(
                headder.prg_rom_chunks,
                headder.chr_rom_chunks,
            )),
            prg_memory,
            chr_memory,
            mapper_id,
            prg_banks: headder.prg_rom_chunks,
            chr_banks: headder.chr_rom_chunks,
            mirror,
        };
        Ok(cart)
    }

    fn get_mapper(&self) -> &dyn MapperTrait {
        &*self.mapper
    }

    // These 4 functions return kinda gross vars....
    pub fn cpu_read(&self, addr: u16) -> Result<u8, ()> {
        if let Ok(mapped_addr) = self.get_mapper().cpu_map_read(addr) {
            return Ok(self.prg_memory[mapped_addr as usize]);
        }
        Err(())
    }
    pub fn cpu_write(&mut self, addr: u16, data: u8) -> Result<bool, ()> {
        if let Ok(mapped_addr) = self.get_mapper().cpu_map_write(addr) {
            self.prg_memory[mapped_addr as usize] = data;
            return Ok(true);
        }
        Err(())
    }
    pub fn ppu_read(&self, addr: u16) -> Result<u8, ()> {
        if let Ok(mapped_addr) = self.get_mapper().ppu_map_read(addr) {
            return Ok(self.chr_memory[mapped_addr as usize]);
        }
        Err(())
    }
    pub fn ppu_write(&mut self, addr: u16, data: u8) -> Result<bool, ()> {
        if let Ok(mapped_addr) = self.get_mapper().ppu_map_write(addr) {
            self.chr_memory[mapped_addr as usize] = data;
            return Ok(true);
        }
        Err(())
    }
}

pub enum Rom {
    CPUTest,
    PpuColorTest,
    Mario,
    DonkeyKong,
}

