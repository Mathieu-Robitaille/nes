use crate::cartridge::Cartridge;
use crate::ppu::PPU;
use std::{
    rc::Rc,
    cell::RefCell,
};

pub trait BusReader {
    fn bus_read(&mut self, addr: u16, read_only: bool) -> u8;
}

pub trait BusWriter {
    fn bus_write(&mut self, addr: u16, data: u8);
}

pub struct Bus {
    ram: [u8; 0x07FF],

    cart: Rc<RefCell<Cartridge>>,
    
    ppu: PPU,
    clock_cycle: u32,
}

impl Bus {
    pub fn new(cart: Rc<RefCell<Cartridge>>) -> Self {
        let ram: [u8; 0x07FF] = [0; 0x07FF];
        let ppu: PPU = PPU::new(cart.clone());
        Bus { 
            ram, 
            cart: cart.clone(),
            ppu,
            clock_cycle: 0 
        }
    }

    pub fn clock(&mut self) {
        todo!()
    }

    pub fn reset(&self) {
        todo!()
    }

    pub(crate) fn cpu_read(&self, addr: u16, _read_only: bool) -> u8 {
        if let Ok(d) = self.cart.borrow().cpu_read(addr) {
            return d;
        } else if (0x0000..=0x1FFF).contains(&addr) {
            self.ram[(addr & 0x07FF) as usize]
        } else if (0x2000..=0x3FFF).contains(&addr) {
            self.ppu.cpu_read(addr & 0x0007)
        } else {
            self.ram[addr as usize]
        }
    }

    pub(crate) fn cpu_write(&mut self, addr: u16, data: u8) {
        if let Ok(_) = self.cart.borrow_mut().cpu_write(addr, data) {
            // nothing to do
        } else if (0x0000..0x1FFF).contains(&addr) {
            self.ram[(addr & 0x07FF) as usize] = data;
        } else if (0x2000..=0x3FFF).contains(&addr) {
            self.ppu.cpu_write(addr & 0x0007, data)
        } else {
            self.ram[addr as usize] = data;
        }
    }
}
