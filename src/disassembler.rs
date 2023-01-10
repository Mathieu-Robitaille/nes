use crate::{
    cartridge::Cartridge,
    cpu::Cpu6502,
    instructions::{
        instruction::AddressingMode::*,
        instruction::AddressingMode,
        instruction_table::instruction_table::INSTRUCTIONS_ARR,
    },
};
use std::{
    ops::Deref,
    collections::HashMap,
    fmt,
};

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

#[derive(Debug, Clone)]
pub struct HumanInstruction {
    pub name: String,
    pub addr_mode: String,
    pub following_bytes: [i16; 2],
    pub location: u16,
}

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



// impl Deref for HumanInstruction {
//     type Target = str; 
//     fn deref<'a>(&'a self) -> &'a str {
//         format!("{:?} -> {:?}", self.name, self.addr_mode).as_str()
//     }
// }

// Impl A
trait AdvanceByExt: Iterator {
    fn adv_by(&mut self, steps: usize);
}

impl<I> AdvanceByExt for I where I: Iterator {
    fn adv_by(&mut self, steps: usize) {
        for _ in 0..steps { self.next(); }
    }
}

// Impl B
fn advance_by(iter: &mut impl Iterator, steps: usize) {
    for _ in 0..steps { iter.next(); }
}

// #[feature(iter_advance_by)]
pub fn decode_rom(cart: &Cartridge) -> HashMap<u16, HumanInstruction> {
    let mut hm: HashMap<u16, HumanInstruction> = HashMap::new();
    let mut bytes_iter = cart.data.iter().enumerate();
    let mut pos = cart.memory_start;
    while let Some((idx, byte)) = bytes_iter.next() {
        let ins = INSTRUCTIONS_ARR[*byte as usize];
        let adv_by = decode_bytes_used(ins.addr_mode);

        let following_bytes: [i16; 2] = {
            let mut r = [i16::MIN, i16::MIN];
            let i: usize = (pos - cart.memory_start) as usize +1;
            for (i, e) in (i..i+adv_by).enumerate() {
                r[i] = cart.data[e] as i16;
            }
            r
        };

        hm.insert(pos, HumanInstruction { 
            name: ins.name.to_string(), 
            addr_mode: format!("{}", ins.addr_mode),
            location: pos,
            following_bytes
        });
        {
            // Adding needless scops just for clarity that there are multiple 
            // options here

            // bytes_iter.advance_by(adv_by); // Nightly feature atm, lets make this ourselves
            // advance_by(&mut bytes_iter, adv_by);
            bytes_iter.adv_by(adv_by);
        }
        pos += adv_by as u16 +1;
    };
    hm
}

pub fn get_rom_instructions_from_range(pc: u16, bounds: usize, diss: &HashMap<u16, HumanInstruction>) -> Vec<HumanInstruction> {
    if diss.get(&pc).is_none() { return vec![]; }

    // Not really required as u16 will shit its pants from and over/under flow
    //   but you know, safety first!
    for i in [pc-bounds as u16, pc+bounds as u16] {
        if !(0x0000..=0xFFFF).contains(&i) { return vec![]; }
    }

    let start = match pc.overflowing_sub(bounds as u16) {
        (_, true) => { pc },
        (x, false) => { x },
    };
    let end = match pc.overflowing_add(bounds as u16) {
        (_, true) => { pc },
        (x, false) => { x },
    };

    let mut keys: Vec<u16> = diss.keys()
        .into_iter()
        .filter(|x| (start..=end).contains(x))
        .map(|x| *x)
        .collect();

    keys.sort();
    let mut r: Vec<HumanInstruction> = vec![];
    for i in keys.iter() {
        if let Some(v) = diss.get(i) { r.push(v.clone()) }
    }
    r
}
