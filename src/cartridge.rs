use std::{
    io,
    io::prelude::*,
    fs::File,
};

pub struct Cartridge {
    pub data: Vec<u8>,
    pub memory_start: u16,
    pub program_start: u16,
    pub range_low: u16,
    pub range_high: u16,
}

impl Cartridge {
    pub fn new(data: Vec<u8>, memory_start: u16, program_start: u16) -> Self {
        let l = data.len() as u16;
        Self { 
            data, 
            memory_start, 
            program_start,
            range_low: memory_start,
            range_high: memory_start + l
        }
    }
}

pub enum TestCart {
    BASIC,
    ADV,
}


pub fn get_test_bytes(pick: TestCart) -> io::Result<Cartridge> {
    let cart = match pick {
        TestCart::BASIC => ("test-bins/basic/o6502-2023-01-09-100722.bin", 0x8000, 0x8000), 
        TestCart::ADV => ("test-bins/suite/6502_functional_test.bin", 0x0000, 0x0400),
    };
    let mut f = File::open(cart.0)?;
    let mut buffer: Vec<u8> = Vec::new();
    f.read_to_end(&mut buffer)?;
    Ok(Cartridge::new(buffer, cart.1, cart.2))
}