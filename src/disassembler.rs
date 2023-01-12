use crate::{
    cartridge::Cartridge,
    instructions::{
        instruction::AddressingMode, instruction::AddressingMode::*,
        instruction_table::instruction_table::INSTRUCTIONS_ARR,
    },
};
use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

impl fmt::Display for AddressingMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IMP => write!(f, "IMP"),
            IMM => write!(f, "IMM"),
            ZP0 => write!(f, "ZP0"),
            ZPX => write!(f, "ZPX"),
            ZPY => write!(f, "ZPY"),
            REL => write!(f, "REL"),
            ABS => write!(f, "ABS"),
            ABX => write!(f, "ABX"),
            ABY => write!(f, "ABY"),
            IND => write!(f, "IND"),
            IZX => write!(f, "IZX"),
            IZY => write!(f, "IZY"),
            XXX => write!(f, "XXX"),
        }
    }
}

fn decode_bytes_used(ins: AddressingMode) -> usize {
    match ins {
        IMP | XXX => 0,
        IMM | IZX | IZY | REL | ZP0 | ZPX | ZPY => 1,
        ABS | ABX | ABY | IND => 2,
    }
}

#[allow(unused)]
macro_rules! name_of_function {
    ($f:ident) => {{
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        match &name[..name.len() - 3].rfind(':') {
            Some(pos) => &name[pos + 1..name.len() - 1],
            None => &name[..name.len() - 3],
        }
    }};
}

// Impl A
// trait AdvanceByExt: Iterator {
//     fn adv_by(&mut self, steps: usize);
// }

// impl<I> AdvanceByExt for I where I: Iterator {
//     fn adv_by(&mut self, steps: usize) {
//         for _ in 0..steps { self.next(); }
//     }
// }

// // Impl B
// fn advance_by(iter: &mut impl Iterator, steps: usize) {
//     for _ in 0..steps { iter.next(); }
// }

pub fn disassemble_rom(
    start: u16,
    stop: u16,
    cart_rc: Rc<RefCell<Cartridge>>,
) -> HashMap<u16, String> {
    let cart = cart_rc.borrow_mut();
    let mut addr: u32 = start as u32;
    let (mut value, mut lo, mut hi): (u8, u8, u8);

    let mut hm: HashMap<u16, String> = HashMap::new();
    let mut line_addr: u16;

    while addr <= (stop as u32) {
        line_addr = addr as u16;
        let mut instruction_string = format!("$0x{:04X?} -> ", addr);
        let opcode = cart.cpu_read(addr as u16).unwrap_or(0x00);
        addr += 1;
        instruction_string
            .push_str(format!("{} ", INSTRUCTIONS_ARR[opcode as usize].name).as_str());

        let addr_mode = INSTRUCTIONS_ARR[opcode as usize].addr_mode;

        match addr_mode {
            IMP => {
                instruction_string.push_str(format!("               {{{}}}", addr_mode).as_str());
            }
            IMM | ZP0 | ZPX | ZPY | IZX | IZY => {
                if let Ok(v) = cart.cpu_read(addr as u16) {
                    instruction_string.push_str(format!("#${:04?}         {{{}}}", v, addr_mode).as_str());
                }
                addr += 1;
            }
            ABS | ABX | ABY | IND => {
                lo = cart.cpu_read(addr as u16).unwrap();
                addr += 1;
                hi = cart.cpu_read(addr as u16).unwrap();
                addr += 1;
                instruction_string.push_str(
                    format!("#${:04X?}         {{{}}}", ((hi as u16) << 8 | lo as u16), addr_mode).as_str(),
                );
            }
            REL => {
                value = cart.cpu_read(addr as u16).unwrap();
                addr += 1;
                let (rel, _) = (addr as u16).overflowing_sub((value as u16) & 0xFF00);
                instruction_string.push_str(
                    format!("#$  {:02X?} [${:04X?}] {{{}}}", value, rel, addr_mode).as_str(),
                );
            }
            XXX => {
                instruction_string.push_str("How?");
            }
        }
        hm.insert(line_addr, instruction_string);
    }
    hm
}
