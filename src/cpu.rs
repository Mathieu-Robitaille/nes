use crate::bus::{Bus, BusReader, BusWriter};
use crate::instructions::{
    instruction_table::instruction_table::INSTRUCTIONS_ARR,
    instruction::{
        process_instruction_addressing_mode, AddressingMode, Instruction,
    }
};
use bitflags::bitflags;
use std::{
    cell::RefCell,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign},
    rc::Rc,
};

bitflags! {
    pub struct Flags: u8 {
        const C = 1 << 0; // Carry Bit
        const Z = 1 << 1; // Zero
        const I = 1 << 2; // Disable Interupts
        const D = 1 << 3; // Decimal Mode
        const B = 1 << 4; // Break
        const U = 1 << 5; // Unused
        const V = 1 << 6; // Overflow
        const N = 1 << 7; // Negative
    }
}

impl BitAnd<Flags> for u8 {
    type Output = u8;
    fn bitand(self, rhs: Flags) -> Self::Output {
        self & rhs.bits
    }
}

impl BitAndAssign<Flags> for u8 {
    fn bitand_assign(&mut self, rhs: Flags) {
        *self = *self & rhs.bits
    }
}

impl BitOr<Flags> for u8 {
    type Output = u8;
    fn bitor(self, rhs: Flags) -> Self::Output {
        self | rhs.bits
    }
}

impl BitOrAssign<Flags> for u8 {
    fn bitor_assign(&mut self, rhs: Flags) {
        *self = *self | rhs.bits
    }
}

impl BitXor<Flags> for u8 {
    type Output = u8;
    fn bitxor(self, rhs: Flags) -> Self::Output {
        self ^ rhs.bits
    }
}

impl BitXorAssign<Flags> for u8 {
    fn bitxor_assign(&mut self, rhs: Flags) {
        *self = *self ^ rhs.bits
    }
}

pub struct Cpu6502 {
    pub(crate) acc: u8,
    pub(crate) x_reg: u8,
    pub(crate) y_reg: u8,
    pub(crate) stack_pointer: u8,
    pub(crate) program_counter: u16,
    pub(crate) status: u8,

    pub(crate) fetched: u8,
    pub(crate) temp: u16,
    pub(crate) addr_abs: u16,
    pub(crate) addr_rel: u16,
    pub(crate) addressing_mode: AddressingMode,
    pub(crate) cycles: u8,
    pub(crate) opcode: u8,
    pub(crate) clock_count: u32,

    bus_handle: Option<Rc<RefCell<Bus>>>,
}

impl Cpu6502 {
    pub(crate) fn read_bus(&mut self, addr: u16) -> u8 {
        self.bus_read(addr, false)
    }
    pub(crate) fn write_bus(&mut self, addr: u16, data: u8) {
        self.bus_write(addr, data)
    }

    pub(crate) fn get_flag(&self, flag: Flags) -> u8 {
        if self.status & flag > 0 {
            return 1;
        } else {
            return 0;
        }
    }

    pub(crate) fn read_bus_two_bytes(&mut self, addr: u16) -> u16 {
        let lo: u16 = self.bus_read(addr, false) as u16;
        let hi: u16 = (self.bus_read(addr + 1, false) as u16) << 8;
        hi | lo
    }

    pub(crate) fn write_bus_two_bytes(&mut self, addr: u16, data: u16) {
        let hi: u8 = (data >> 8) as u8;
        let lo: u8 = data as u8;
        self.write_bus(addr, hi);
        self.write_bus(addr - 1, lo);
    }

    pub(crate) fn set_flag(&mut self, flag: Flags, v: bool) {
        if v {
            self.status = self.status | flag;
        } else {
            self.status = self.status & !flag;
        }
    }

    pub(crate) fn clock(&mut self) {
        self.clock_count += 1;
        self.cycles -= 1;

        if self.cycles != 0 {
            return;
        }

        self.opcode = self.read_bus(self.program_counter);

        // make suuuuuuure its set
        self.set_flag(Flags::U, true);

        self.program_counter += 1;

        if (0x00..=0xFF).contains(&self.opcode) {
            let ins: &Instruction = &INSTRUCTIONS_ARR[self.opcode as usize];
            let additional_cycle1: u8 = process_instruction_addressing_mode(ins, self);
            let additional_cycle2: u8 = (ins.function)(self);
            self.cycles = ins.clock_cycles + additional_cycle1 + additional_cycle2;
        }

        // make suuuuuuure its set
        self.set_flag(Flags::U, true);
    }

    pub(crate) fn reset(&mut self) {
        self.addr_abs = 0xFFFC;
        self.program_counter = self.read_bus_two_bytes(self.addr_abs);
        self.acc = 0;
        self.x_reg = 0;
        self.y_reg = 0;
        self.stack_pointer = 0xFD;
        self.status = 0x00 | Flags::U;

        self.addr_rel = 0x0000;
        self.addr_abs = 0x0000;
        self.fetched = 0x0000;

        self.cycles = 8;
    }

    fn irq(&mut self) {
        if self.get_flag(Flags::I) != 0 {
            return;
        }
        self.write_bus_two_bytes(0x0100 + (self.stack_pointer as u16), self.program_counter);
        self.stack_pointer -= 2;

        self.set_flag(Flags::B, false);
        self.set_flag(Flags::U, true);
        self.set_flag(Flags::I, true);

        self.write_bus(0x0100 + (self.stack_pointer as u16), self.status);
        self.stack_pointer -= 1;

        self.addr_abs = 0xFFFE;
        self.program_counter = self.read_bus_two_bytes(self.addr_abs);
        self.cycles = 7;
    }

    fn nmi(&mut self) {
        self.write_bus_two_bytes(0x0100 + (self.stack_pointer as u16), self.program_counter);
        self.stack_pointer -= 2;

        self.set_flag(Flags::B, false);
        self.set_flag(Flags::U, true);
        self.set_flag(Flags::I, true);

        self.write_bus(0x0100 + (self.stack_pointer as u16), self.status);
        self.stack_pointer -= 1;

        self.addr_abs = 0xFFFA;
        self.program_counter = self.read_bus_two_bytes(self.addr_abs);
        self.cycles = 8;
    }

    pub(crate) fn fetch(&mut self) -> u8 {
        if self.addressing_mode != AddressingMode::IMP {
            self.fetched = self.read_bus(self.addr_abs);
        }
        self.fetched
    }

    fn new(bh: Rc<RefCell<Bus>>) -> Self {
        Cpu6502 {
            acc: 0x00,
            x_reg: 0x00,
            y_reg: 0x00,
            stack_pointer: 0x00,
            program_counter: 0x0000,
            status: 0x00,
            fetched: 0x00,
            temp: 0x0000,
            addr_abs: 0x0000,
            addr_rel: 0x00,
            addressing_mode: AddressingMode::ABS,
            opcode: 0x00,
            cycles: 0,
            clock_count: 0,
            bus_handle: Some(bh),
        }
    }
}

impl BusReader for Cpu6502 {
    fn bus_read(&mut self, addr: u16, _read_only: bool) -> u8 {
        if let Some(x) = &self.bus_handle {
            return x.borrow_mut().read(addr, false);
        }
        0x00
    }
}

impl BusWriter for Cpu6502 {
    fn bus_write(&mut self, addr: u16, data: u8) {
        if let Some(x) = &self.bus_handle {
            x.borrow_mut().write(addr, data);
        }
    }
}
