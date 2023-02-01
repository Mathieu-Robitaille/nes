use crate::cartridge::Cartridge;
use crate::ppu::PPU;
use std::{cell::RefCell, rc::Rc};

pub trait BusReader {
    fn bus_read(&mut self, addr: u16, read_only: bool) -> u8;
}

pub trait BusWriter {
    fn bus_write(&mut self, addr: u16, data: u8);
}

pub struct Bus {
    clock_cycle: u32,
    ram: [u8; 0x0800],

    pub dma_page: u8,
    pub dma_addr: u8,
    pub dma_data: u8,

    pub dma_transfer: bool,
    pub dma_dummy: bool,

    cart: Rc<RefCell<Cartridge>>,

    pub ppu: PPU,
}

impl Bus {
    pub fn new(cart: Rc<RefCell<Cartridge>>) -> Self {
        let ram: [u8; 0x0800] = [0; 0x0800];
        let ppu: PPU = PPU::new(cart.clone());
        Bus {
            ram,
            cart: cart.clone(),
            ppu,
            clock_cycle: 0,
            dma_page: 0,
            dma_addr: 0,
            dma_data: 0,
            dma_transfer: false,
            dma_dummy: true,
        }
    }

    pub fn clock(&mut self) {
        todo!()
    }

    pub fn reset(&self) {
        todo!()
    }

    pub(crate) fn cpu_read(&mut self, addr: u16, _read_only: bool) -> u8 {
        if let Ok(d) = self.cart.borrow().cpu_read(addr) {
            return d;
        } else if (0x0000..=0x1FFF).contains(&addr) {
            return self.ram[(addr & 0x07FF) as usize];
        } else if (0x2000..=0x3FFF).contains(&addr) {
            return self.ppu.cpu_read(addr & 0x0007, _read_only);
        }
        0x00
    }

    pub(crate) fn cpu_write(&mut self, addr: u16, data: u8) {
        // if (0x0000..=0x1FFF).contains(&addr) {
        //     println!("Writing to ppu {addr:04X?}");
        // }

        if let Ok(_) = self.cart.borrow_mut().cpu_write(addr, data) {
            return;
        }
        if (0x0000..=0x1FFF).contains(&addr) {
            self.ram[(addr & 0x07FF) as usize] = data;
        } else if (0x2000..=0x3FFF).contains(&addr) {
            self.ppu.cpu_write(addr & 0x0007, data)
        } else if addr == 0x4014 {
            self.dma_page = data;
            self.dma_addr = 0x00;
            self.dma_transfer = true;
        } else if (0x4016..=0x4017).contains(&addr) {
        }
    }
}
