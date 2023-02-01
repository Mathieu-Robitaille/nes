use crate::bus::{Bus, BusReader, BusWriter};
use crate::disassembler::decode_bytes_used;
use crate::instructions::{
    instruction::{process_instruction_addressing_mode, AddressingMode, Instruction},
    instruction_table::instruction_table::INSTRUCTIONS_ARR,
};
use bitflags::bitflags;
use std::io;
use std::{
    cell::RefCell,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign},
    rc::Rc,
};

bitflags! {
    pub struct CPUFlags: u8 {
        const C = 1 << 0; // 0x01 // Carry Bit
        const Z = 1 << 1; // 0x02 // Zero
        const I = 1 << 2; // 0x04 // Disable Interupts
        const D = 1 << 3; // 0x08 // Decimal Mode
        const B = 1 << 4; // 0x10 // Break
        const U = 1 << 5; // 0x20 // Unused
        const V = 1 << 6; // 0x40 // Overflow
        const N = 1 << 7; // 0x80 // Negative
    }
}

impl BitAnd<CPUFlags> for u8 {
    type Output = u8;
    fn bitand(self, rhs: CPUFlags) -> Self::Output {
        self & rhs.bits
    }
}

impl BitAndAssign<CPUFlags> for u8 {
    fn bitand_assign(&mut self, rhs: CPUFlags) {
        *self = *self & rhs.bits
    }
}

impl BitOr<CPUFlags> for u8 {
    type Output = u8;
    fn bitor(self, rhs: CPUFlags) -> Self::Output {
        self | rhs.bits
    }
}

impl BitOrAssign<CPUFlags> for u8 {
    fn bitor_assign(&mut self, rhs: CPUFlags) {
        *self = *self | rhs.bits
    }
}

impl BitXor<CPUFlags> for u8 {
    type Output = u8;
    fn bitxor(self, rhs: CPUFlags) -> Self::Output {
        self ^ rhs.bits
    }
}

impl BitXorAssign<CPUFlags> for u8 {
    fn bitxor_assign(&mut self, rhs: CPUFlags) {
        *self = *self ^ rhs.bits
    }
}

pub struct Cpu6502 {
    pub(crate) acc: u8,
    pub(crate) x_reg: u8,
    pub(crate) y_reg: u8,
    pub(crate) stack_pointer: u8,
    pub(crate) pc: u16,
    pub(crate) status: u8,

    pub(crate) fetched: u8,
    pub(crate) temp: u16,
    pub(crate) addr_abs: u16,
    pub(crate) addr_rel: u16,
    pub(crate) addressing_mode: AddressingMode,
    pub(crate) cycles: u8,
    pub(crate) opcode: u8,
    pub(crate) clock_count: u32,
    pub(crate) instruction_count: usize,
    pub instruction_complete: bool,

    pub(crate) bus: Bus,
    pc_before: u16,
    a_before: u8,
    x_before: u8,
    y_before: u8,
}

impl Cpu6502 {
    pub(crate) fn read_bus(&mut self, addr: u16) -> u8 {
        self.bus_read(addr, false)
    }
    pub(crate) fn write_bus(&mut self, addr: u16, data: u8) {
        self.bus_write(addr, data)
    }

    pub(crate) fn get_flag(&self, flag: CPUFlags) -> u8 {
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

    pub(crate) fn set_flag(&mut self, flag: CPUFlags, v: bool) {
        if v {
            self.status |= flag;
        } else {
            self.status &= !flag;
        }
    }

    pub fn generate_debug_string(&self) -> String {
        let Cpu6502 {
            acc,
            x_reg,
            y_reg,
            stack_pointer,
            clock_count,
            instruction_count,
            opcode,
            pc_before,
            a_before,
            x_before,
            y_before,
            ..
        } = self;
        let mut ins_str: String = format!(
            "c{:} i{:} A:{:02X} X:{:02X} Y:{:02X} S:{:02X} ${:04X}: {:02X}",
            clock_count, instruction_count, a_before, x_before, y_before, stack_pointer, pc_before, opcode
        );
        ins_str
    }

    pub(crate) fn clock(&mut self) -> Option<String> {
        let mut ret: Option<String> = None;
        self.clock_count += 1;
        self.cycles -= 1;
        if self.cycles == 0 {

            self.a_before = self.acc;
            self.x_before = self.x_reg;
            self.y_before = self.y_reg;

            self.pc_before = self.pc;
            self.opcode = self.read_bus(self.pc);
            // make suuuuuuure its set
            self.set_flag(CPUFlags::U, true);

            self.pc = self.pc.wrapping_add(1); // fix

            let ins: &Instruction = &INSTRUCTIONS_ARR[self.opcode as usize];

            self.addressing_mode = ins.addr_mode.clone();
            let page_change_additional_cycle: u8 = process_instruction_addressing_mode(ins, self);

            
            // Run the instruction
            (ins.function)(self); /* removed additional clock cycles as it was wroooong */ 
            
            self.cycles += ins.clock_cycles + page_change_additional_cycle;
            
            // make suuuuuuure its set
            self.set_flag(CPUFlags::U, true);
            self.instruction_count += 1;
            self.instruction_complete = true;
            
            ret = Some(self.generate_debug_string());
        }
        ret
    }

    pub(crate) fn reset(&mut self, testcart: Option<u16>) {
        self.addr_abs = 0xFFFC;
        match testcart {
            Some(x) => self.pc = x,
            None => self.pc = self.read_bus_two_bytes(self.addr_abs),
        };
        self.acc = 0;
        self.x_reg = 0;
        self.y_reg = 0;

        self.stack_pointer = self.stack_pointer.wrapping_sub(3);

        self.status = 0x00 | CPUFlags::U;

        self.addr_rel = 0x0000;
        self.addr_abs = 0x0000;
        self.fetched = 0x0000;
        self.instruction_count = 0;

        self.clock_count = 0;

        self.cycles = 7;
    }

    fn irq(&mut self) {
        if self.get_flag(CPUFlags::I) != 0 {
            return;
        }
        self.write_bus_two_bytes(0x0100 + (self.stack_pointer as u16), self.pc);
        self.stack_pointer = self.stack_pointer.wrapping_sub(2);

        self.set_flag(CPUFlags::B, false);
        self.set_flag(CPUFlags::U, true);
        self.set_flag(CPUFlags::I, true);

        self.write_bus(0x0100 + (self.stack_pointer as u16), self.status);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);

        self.addr_abs = 0xFFFE;
        self.pc = self.read_bus_two_bytes(self.addr_abs);
        self.cycles = 7;
    }

    pub fn nmi(&mut self) {
        self.write_bus_two_bytes(0x0100 + (self.stack_pointer as u16), self.pc);
        self.stack_pointer = self.stack_pointer.wrapping_sub(2);

        self.set_flag(CPUFlags::B, false);
        self.set_flag(CPUFlags::U, true);
        self.set_flag(CPUFlags::I, true);

        self.write_bus(0x0100 + (self.stack_pointer as u16), self.status);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);

        self.addr_abs = 0xFFFA;
        self.pc = self.read_bus_two_bytes(self.addr_abs);
        // println!("NMI -> {:04X?}", self.pc);
        self.cycles = 8;
    }

    pub(crate) fn fetch(&mut self) -> u8 {
        if self.addressing_mode != AddressingMode::IMP {
            self.fetched = self.read_bus(self.addr_abs);
        }
        self.fetched
    }

    pub fn new(b: Bus) -> Self {
        Cpu6502 {
            acc: 0x00,
            x_reg: 0x00,
            y_reg: 0x00,
            stack_pointer: 0xFD,
            pc: 0x0000,
            status: 0x00,
            fetched: 0x00,
            temp: 0x0000,
            addr_abs: 0x0000,
            addr_rel: 0x00,
            addressing_mode: AddressingMode::ABS,
            opcode: 0x00,
            cycles: 1,
            clock_count: 0,
            instruction_count: 0,
            instruction_complete: false,
            bus: b,
            pc_before: 0x0000,
            a_before: 0,
            x_before: 0,
            y_before: 0,
        }
    }
}

impl BusReader for Cpu6502 {
    fn bus_read(&mut self, addr: u16, _read_only: bool) -> u8 {
        self.bus.cpu_read(addr, false)
    }
}

impl BusWriter for Cpu6502 {
    fn bus_write(&mut self, addr: u16, data: u8) {
        self.bus.cpu_write(addr, data);
    }
}
