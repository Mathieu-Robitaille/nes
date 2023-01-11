use crate::cartridge::Cartridge;
use std::{
    rc::Rc,
    cell::RefCell,
};


pub struct PPU {
    cart: Rc<RefCell<Cartridge>>,
    name_table: [[u8; 1024]; 2],
    palette: [u8; 32],
}

impl PPU {
    pub fn new(cart: Rc<RefCell<Cartridge>>) -> Self {
        Self { 
            cart: cart.clone(),
            name_table: [[0; 1024], [0; 1024]],
            palette: [0; 32],
        }
    }

    pub fn clock() {
        todo!()
    }


    pub fn cpu_read(&self, addr: u16) -> u8 {
        match addr {
            0x0000  => { return 0; }, // Control
            0x0001  => { return 0; }, // Mask
            0x0002  => { return 0; }, // Status
            0x0003  => { return 0; }, // OAM Address
            0x0004  => { return 0; }, // OAM Data
            0x0005  => { return 0; }, // Scroll
            0x0006  => { return 0; }, // PPU Address
            0x0007  => { return 0; }, // PPU Data
            _ => { return 0; }        //
        }
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000  => { }, // Control
            0x0001  => { }, // Mask
            0x0002  => { }, // Status
            0x0003  => { }, // OAM Address
            0x0004  => { }, // OAM Data
            0x0005  => { }, // Scroll
            0x0006  => { }, // PPU Address
            0x0007  => { }, // PPU Data
            _ => { }        //
        }
        todo!()
    }

    pub fn ppu_read(&self, addr: u16) -> u8 {
        if let Ok(x) = self.cart.borrow().cpu_read(addr) {
            return x;
        } 
        todo!()
    }

    pub fn ppu_write(&mut self, addr: u16, data: u8) {
        if let Ok(_) = self.cart.borrow_mut().cpu_write(addr, data) {

        } 
        todo!()
    }
}