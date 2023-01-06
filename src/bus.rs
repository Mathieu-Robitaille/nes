use crate::cpu::Cpu6502;

pub trait BusReader {
    fn bus_read(&mut self, addr: u16, read_only: bool) -> u8;
}

pub trait BusWriter { 
    fn bus_write(&mut self, addr: u16, data: u8);
}


pub struct Bus {
    // Actual cpu
    cpu: Cpu6502,

    // Fake as fuck for now
    ram: [u8; 64 * 1024]
}

impl Bus {
    pub(crate) fn read(&self, addr: u16, _read_only: bool) -> u8 {
        self.ram[addr as usize]
    }

    pub(crate) fn write(&mut self, addr: u16, data: u8) {
        self.ram[addr as usize] = data;
    }

}