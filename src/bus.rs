use crate::cpu::Cpu6502;
use crate::cartridge::Cartridge;

pub trait BusReader {
    fn bus_read(&mut self, addr: u16, read_only: bool) -> u8;
}

pub trait BusWriter {
    fn bus_write(&mut self, addr: u16, data: u8);
}

pub struct Bus {
    // Fake as fuck for now
    ram: [u8; 64 * 1024],
    
    clock_cycle: u32,
}

impl Bus {
    pub fn new(ram: [u8; 64 * 1024]) -> Self {
        Bus { ram, clock_cycle: 0 }
    }

    pub fn clock(&mut self) {

    }

    pub(crate) fn read(&self, addr: u16, _read_only: bool) -> u8 {
        self.ram[addr as usize]
    }

    pub(crate) fn write(&mut self, addr: u16, data: u8) {
        self.ram[addr as usize] = data;
    }

    pub(crate) fn set_start(&mut self, starting_address: u16) {
        self.write(0xFFFC, starting_address as u8);
        self.write(0xFFFD, (starting_address >> 8) as u8);
    }

    pub(crate) fn load_program(&mut self, cart: &Cartridge) -> Result<(), &str> {
        // if cart.data.len() > self.ram.len() - cart.start as usize { return Err("Program too large."); }
        // if (cart.start..=(cart.start + cart.data.len() as u16)).contains(&0xFFFC) { return Err("Program too large."); }
        for (i, byte) in cart.data.iter().enumerate() {
            self.write(cart.memory_start + i as u16, *byte)
        }
        self.set_start(cart.program_start);
        Ok(())
    }
}
