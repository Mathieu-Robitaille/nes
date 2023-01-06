use crate::cpu::{Cpu6502, Flags};
use std::collections::HashMap;
use lazy_static::lazy_static;

pub struct Instruction {
    name: String,
    addr_mode: AddressingMode,
    function: for<'r> fn(&'r mut Cpu6502) -> u8,
    clock_cycles: u8,
}

#[derive(PartialEq)]
pub enum AddressingMode {
    IMP, IMM,
    ZP0, ZPX,
    ZPY, REL,
    ABS, ABX,
    ABY, IND,
    IZX, IZY,
}


fn process_instruction_addressing_mode(instruction: &Instruction, cpu: &mut Cpu6502) -> u8 {
    // returns 0 or 1 depending if an extra cycle is required
    match instruction.addr_mode {

        // Address Mode: Implied
        // There is no additional data required for this instruction. The instruction
        // does something very simple like like sets a status bit. However, we will
        // target the accumulator, for instructions like PHA
        AddressingMode::IMP => { cpu.fetched = cpu.acc; 0 }

        // Address Mode: Immediate
        // The instruction expects the next byte to be used as a value, so we'll prep
        // the read address to point to the next byte
        AddressingMode::IMM => { cpu.addr_abs = cpu.program_counter + 1; 0 }

        // Address Mode: Zero Page
        // To save program bytes, zero page addressing allows you to absolutely address
        // a location in first 0xFF bytes of address range. Clearly this only requires
        // one byte instead of the usual two.
        AddressingMode::ZP0 => { 
            cpu.addr_abs = cpu.read_bus(cpu.program_counter) as u16; 
            cpu.program_counter += 1; 
            cpu.addr_abs &= 0x00FF;
            0
        }

        // Address Mode: Zero Page with X Offset
        // Fundamentally the same as Zero Page addressing, but the contents of the X Register
        // is added to the supplied single byte address. This is useful for iterating through
        // ranges within the first page.
        AddressingMode::ZPX => {
            cpu.addr_abs = (cpu.read_bus(cpu.program_counter) + cpu.x_reg) as u16;
            cpu.program_counter += 1;
            cpu.addr_abs &= 0x00FF;
            0
        }

        // Address Mode: Zero Page with Y Offset
        // Same as above but uses Y Register for offset
        AddressingMode::ZPY => {
            cpu.addr_abs = (cpu.read_bus(cpu.program_counter) + cpu.y_reg) as u16;
            cpu.program_counter += 1;
            cpu.addr_abs &= 0x00FF;
            0
        }

        // Address Mode: Relative
        // This address mode is exclusive to branch instructions. The address
        // must reside within -128 to +127 of the branch instruction, i.e.
        // you cant directly branch to any address in the addressable range.
        AddressingMode::REL => {
            cpu.addr_rel = cpu.read_bus(cpu.program_counter) as u16;
            cpu.program_counter += 1;
            if cpu.addr_rel & 0x80 == 0x80 { cpu.addr_rel |= 0xFF }
            0
        }

        // Address Mode: Absolute 
        // A full 16-bit address is loaded and used
        AddressingMode::ABS => { 
            let low: u16 = cpu.read_bus(cpu.program_counter) as u16;
            cpu.program_counter += 1;
            let high: u16 = (cpu.read_bus(cpu.program_counter) as u16) << 8;
            cpu.program_counter += 1;

            cpu.addr_abs = high | low;
            0
        }

        // Address Mode: Absolute with X Offset
        // Fundamentally the same as absolute addressing, but the contents of the X Register
        // is added to the supplied two byte address. If the resulting address changes
        // the page, an additional clock cycle is required
        AddressingMode::ABX => { 
            let low: u16 = cpu.read_bus(cpu.program_counter) as u16;
            cpu.program_counter += 1;
            let high: u16 = (cpu.read_bus(cpu.program_counter) as u16) << 8;
            cpu.program_counter += 1;
            
            cpu.addr_abs = high | low;
            cpu.addr_abs += cpu.x_reg as u16;
            if cpu.addr_abs & 0xFF00 != high { return 1; }
            else { return 0 }
        }

        // Address Mode: Absolute with Y Offset
        // Fundamentally the same as absolute addressing, but the contents of the Y Register
        // is added to the supplied two byte address. If the resulting address changes
        // the page, an additional clock cycle is required
        AddressingMode::ABY => { 
            let addr = cpu.read_bus_two_bytes(cpu.program_counter);
            cpu.program_counter += 2;

            cpu.addr_abs = addr;
            cpu.addr_abs += cpu.y_reg as u16;
            if cpu.addr_abs & 0xFF00 != addr & 0xFF00 { return 1; }
            else { return 0 }
        }

        // Note: The next 3 address modes use indirection (aka Pointers!)

        // Address Mode: Indirect
        // The supplied 16-bit address is read to get the actual 16-bit address. This is
        // instruction is unusual in that it has a bug in the hardware! To emulate its
        // function accurately, we also need to emulate this bug. If the low byte of the
        // supplied address is 0xFF, then to read the high byte of the actual address
        // we need to cross a page boundary. This doesnt actually work on the chip as 
        // designed, instead it wraps back around in the same page, yielding an 
        // invalid actual address
        AddressingMode::IND => {
            let addr = cpu.read_bus_two_bytes(cpu.program_counter);
            cpu.program_counter += 2;
            let ptr: u16 = addr;

            if addr & 0x00FF == 0x00FF {
                cpu.addr_abs = ((cpu.read_bus(ptr & 0x00FF) as u16) << 8) | cpu.read_bus(ptr + 0) as u16
            } else {
                cpu.addr_abs = ((cpu.read_bus(ptr + 1) as u16) << 8) | cpu.read_bus(ptr + 0) as u16
            }
            0
        }


        // Address Mode: Indirect X
        // The supplied 8-bit address is offset by X Register to index
        // a location in page 0x00. The actual 16-bit address is read 
        // from this location
        AddressingMode::IZX => {
            let t: u16 = cpu.read_bus(cpu.program_counter) as u16;
            cpu.program_counter += 1;

            let lo: u16 = cpu.read_bus((t + cpu.x_reg as u16) & 0x00FF) as u16;
            let hi: u16 = (cpu.read_bus((t + cpu.x_reg as u16 + 1) & 0x00FF) as u16) << 8;
            
            cpu.addr_abs = hi | lo;
            0
        }

        // Address Mode: Indirect Y
        // The supplied 8-bit address indexes a location in page 0x00. From 
        // here the actual 16-bit address is read, and the contents of
        // Y Register is added to it to offset it. If the offset causes a
        // change in page then an additional clock cycle is required.
        AddressingMode::IZY => { 
            let t: u16 = cpu.read_bus(cpu.program_counter) as u16;
            cpu.program_counter += 1;

            let lo: u16 = cpu.read_bus(t & 0x00FF) as u16;
            let hi: u16 = (cpu.read_bus((t + 1) & 0x00FF) as u16) << 8;

            cpu.addr_abs = hi | lo;
            cpu.addr_abs += cpu.y_reg as u16;

            if (cpu.addr_abs & 0xFF00) != hi {
                return 1;
            } else {
                return 0;
            }
        }
    }
}


lazy_static! {
    // BY THE POWER OF AUTS
    static ref INSTRUCTIONS: HashMap<u8, Instruction> = {
        let mut m = HashMap::new();
        m.insert(0x00, Instruction { name: "BRK".to_string(), clock_cycles: 7, addr_mode: AddressingMode::IMM, function: BRK});
//         m.insert(0x01, Instruction { name: "ORA".to_string(), clock_cycles: 6, addr_mode: AddressingMode::IZX, function: ORA});
//         m.insert(0x02, Instruction { name: "???".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x03, Instruction { name: "???".to_string(), clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x04, Instruction { name: "???".to_string(), clock_cycles: 3, addr_mode: AddressingMode::IMP, function: NOP});
//         m.insert(0x05, Instruction { name: "ORA".to_string(), clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: ORA});
//         m.insert(0x06, Instruction { name: "ASL".to_string(), clock_cycles: 5, addr_mode: AddressingMode::ZP0, function: ASL});
//         m.insert(0x07, Instruction { name: "???".to_string(), clock_cycles: 5, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x08, Instruction { name: "PHP".to_string(), clock_cycles: 3, addr_mode: AddressingMode::IMP, function: PHP});
//         m.insert(0x09, Instruction { name: "ORA".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMM, function: ORA});
//         m.insert(0x0A, Instruction { name: "ASL".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: ASL});
//         m.insert(0x0B, Instruction { name: "???".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x0C, Instruction { name: "???".to_string(), clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP});
//         m.insert(0x0D, Instruction { name: "ORA".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABS, function: ORA});
//         m.insert(0x0E, Instruction { name: "ASL".to_string(), clock_cycles: 6, addr_mode: AddressingMode::ABS, function: ASL});
//         m.insert(0x0F, Instruction { name: "???".to_string(), clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x10, Instruction { name: "BPL".to_string(), clock_cycles: 2, addr_mode: AddressingMode::REL, function: BPL});
//         m.insert(0x11, Instruction { name: "ORA".to_string(), clock_cycles: 5, addr_mode: AddressingMode::IZY, function: ORA});
//         m.insert(0x12, Instruction { name: "???".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x13, Instruction { name: "???".to_string(), clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x14, Instruction { name: "???".to_string(), clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP});
//         m.insert(0x15, Instruction { name: "ORA".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ZPX, function: ORA});
//         m.insert(0x16, Instruction { name: "ASL".to_string(), clock_cycles: 6, addr_mode: AddressingMode::ZPX, function: ASL});
//         m.insert(0x17, Instruction { name: "???".to_string(), clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x18, Instruction { name: "CLC".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: CLC});
//         m.insert(0x19, Instruction { name: "ORA".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABY, function: ORA});
//         m.insert(0x1A, Instruction { name: "???".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP});
//         m.insert(0x1B, Instruction { name: "???".to_string(), clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x1C, Instruction { name: "???".to_string(), clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP});
//         m.insert(0x1D, Instruction { name: "ORA".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABX, function: ORA});
//         m.insert(0x1E, Instruction { name: "ASL".to_string(), clock_cycles: 7, addr_mode: AddressingMode::ABX, function: ASL});
//         m.insert(0x1F, Instruction { name: "???".to_string(), clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x20, Instruction { name: "JSR".to_string(), clock_cycles: 6, addr_mode: AddressingMode::ABS, function: JSR});
//         m.insert(0x21, Instruction { name: "AND".to_string(), clock_cycles: 6, addr_mode: AddressingMode::IZX, function: AND});
//         m.insert(0x22, Instruction { name: "???".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x23, Instruction { name: "???".to_string(), clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x24, Instruction { name: "BIT".to_string(), clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: BIT});
//         m.insert(0x25, Instruction { name: "AND".to_string(), clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: AND});
//         m.insert(0x26, Instruction { name: "ROL".to_string(), clock_cycles: 5, addr_mode: AddressingMode::ZP0, function: ROL});
//         m.insert(0x27, Instruction { name: "???".to_string(), clock_cycles: 5, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x28, Instruction { name: "PLP".to_string(), clock_cycles: 4, addr_mode: AddressingMode::IMP, function: PLP});
//         m.insert(0x29, Instruction { name: "AND".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMM, function: AND});
//         m.insert(0x2A, Instruction { name: "ROL".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: ROL});
//         m.insert(0x2B, Instruction { name: "???".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x2C, Instruction { name: "BIT".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABS, function: BIT});
//         m.insert(0x2D, Instruction { name: "AND".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABS, function: AND});
//         m.insert(0x2E, Instruction { name: "ROL".to_string(), clock_cycles: 6, addr_mode: AddressingMode::ABS, function: ROL});
//         m.insert(0x2F, Instruction { name: "???".to_string(), clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x30, Instruction { name: "BMI".to_string(), clock_cycles: 2, addr_mode: AddressingMode::REL, function: BMI});
//         m.insert(0x31, Instruction { name: "AND".to_string(), clock_cycles: 5, addr_mode: AddressingMode::IZY, function: AND});
//         m.insert(0x32, Instruction { name: "???".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x33, Instruction { name: "???".to_string(), clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x34, Instruction { name: "???".to_string(), clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP});
//         m.insert(0x35, Instruction { name: "AND".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ZPX, function: AND});
//         m.insert(0x36, Instruction { name: "ROL".to_string(), clock_cycles: 6, addr_mode: AddressingMode::ZPX, function: ROL});
//         m.insert(0x37, Instruction { name: "???".to_string(), clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x38, Instruction { name: "SEC".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: SEC});
//         m.insert(0x39, Instruction { name: "AND".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABY, function: AND});
//         m.insert(0x3A, Instruction { name: "???".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP});
//         m.insert(0x3B, Instruction { name: "???".to_string(), clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x3C, Instruction { name: "???".to_string(), clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP});
//         m.insert(0x3D, Instruction { name: "AND".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABX, function: AND});
//         m.insert(0x3E, Instruction { name: "ROL".to_string(), clock_cycles: 7, addr_mode: AddressingMode::ABX, function: ROL});
//         m.insert(0x3F, Instruction { name: "???".to_string(), clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x40, Instruction { name: "RTI".to_string(), clock_cycles: 6, addr_mode: AddressingMode::IMP, function: RTI});
//         m.insert(0x41, Instruction { name: "EOR".to_string(), clock_cycles: 6, addr_mode: AddressingMode::IZX, function: EOR});
//         m.insert(0x42, Instruction { name: "???".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x43, Instruction { name: "???".to_string(), clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x44, Instruction { name: "???".to_string(), clock_cycles: 3, addr_mode: AddressingMode::IMP, function: NOP});
//         m.insert(0x45, Instruction { name: "EOR".to_string(), clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: EOR});
//         m.insert(0x46, Instruction { name: "LSR".to_string(), clock_cycles: 5, addr_mode: AddressingMode::ZP0, function: LSR});
//         m.insert(0x47, Instruction { name: "???".to_string(), clock_cycles: 5, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x48, Instruction { name: "PHA".to_string(), clock_cycles: 3, addr_mode: AddressingMode::IMP, function: PHA});
//         m.insert(0x49, Instruction { name: "EOR".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMM, function: EOR});
//         m.insert(0x4A, Instruction { name: "LSR".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: LSR});
//         m.insert(0x4B, Instruction { name: "???".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x4C, Instruction { name: "JMP".to_string(), clock_cycles: 3, addr_mode: AddressingMode::ABS, function: JMP});
//         m.insert(0x4D, Instruction { name: "EOR".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABS, function: EOR});
//         m.insert(0x4E, Instruction { name: "LSR".to_string(), clock_cycles: 6, addr_mode: AddressingMode::ABS, function: LSR});
//         m.insert(0x4F, Instruction { name: "???".to_string(), clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x50, Instruction { name: "BVC".to_string(), clock_cycles: 2, addr_mode: AddressingMode::REL, function: BVC});
//         m.insert(0x51, Instruction { name: "EOR".to_string(), clock_cycles: 5, addr_mode: AddressingMode::IZY, function: EOR});
//         m.insert(0x52, Instruction { name: "???".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x53, Instruction { name: "???".to_string(), clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x54, Instruction { name: "???".to_string(), clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP});
//         m.insert(0x55, Instruction { name: "EOR".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ZPX, function: EOR});
//         m.insert(0x56, Instruction { name: "LSR".to_string(), clock_cycles: 6, addr_mode: AddressingMode::ZPX, function: LSR});
//         m.insert(0x57, Instruction { name: "???".to_string(), clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x58, Instruction { name: "CLI".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: CLI});
//         m.insert(0x59, Instruction { name: "EOR".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABY, function: EOR});
//         m.insert(0x5A, Instruction { name: "???".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP});
//         m.insert(0x5B, Instruction { name: "???".to_string(), clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x5C, Instruction { name: "???".to_string(), clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP});
//         m.insert(0x5D, Instruction { name: "EOR".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABX, function: EOR});
//         m.insert(0x5E, Instruction { name: "LSR".to_string(), clock_cycles: 7, addr_mode: AddressingMode::ABX, function: LSR});
//         m.insert(0x5F, Instruction { name: "???".to_string(), clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x60, Instruction { name: "RTS".to_string(), clock_cycles: 6, addr_mode: AddressingMode::IMP, function: RTS});
//         m.insert(0x61, Instruction { name: "ADC".to_string(), clock_cycles: 6, addr_mode: AddressingMode::IZX, function: ADC});
//         m.insert(0x62, Instruction { name: "???".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x63, Instruction { name: "???".to_string(), clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x64, Instruction { name: "???".to_string(), clock_cycles: 3, addr_mode: AddressingMode::IMP, function: NOP});
//         m.insert(0x65, Instruction { name: "ADC".to_string(), clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: ADC});
//         m.insert(0x66, Instruction { name: "ROR".to_string(), clock_cycles: 5, addr_mode: AddressingMode::ZP0, function: ROR});
//         m.insert(0x67, Instruction { name: "???".to_string(), clock_cycles: 5, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x68, Instruction { name: "PLA".to_string(), clock_cycles: 4, addr_mode: AddressingMode::IMP, function: PLA});
//         m.insert(0x69, Instruction { name: "ADC".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMM, function: ADC});
//         m.insert(0x6A, Instruction { name: "ROR".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: ROR});
//         m.insert(0x6B, Instruction { name: "???".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x6C, Instruction { name: "JMP".to_string(), clock_cycles: 5, addr_mode: AddressingMode::IND, function: JMP});
//         m.insert(0x6D, Instruction { name: "ADC".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABS, function: ADC});
//         m.insert(0x6E, Instruction { name: "ROR".to_string(), clock_cycles: 6, addr_mode: AddressingMode::ABS, function: ROR});
//         m.insert(0x6F, Instruction { name: "???".to_string(), clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x70, Instruction { name: "BVS".to_string(), clock_cycles: 2, addr_mode: AddressingMode::REL, function: BVS});
//         m.insert(0x71, Instruction { name: "ADC".to_string(), clock_cycles: 5, addr_mode: AddressingMode::IZY, function: ADC});
//         m.insert(0x72, Instruction { name: "???".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x73, Instruction { name: "???".to_string(), clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x74, Instruction { name: "???".to_string(), clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP});
//         m.insert(0x75, Instruction { name: "ADC".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ZPX, function: ADC});
//         m.insert(0x76, Instruction { name: "ROR".to_string(), clock_cycles: 6, addr_mode: AddressingMode::ZPX, function: ROR});
//         m.insert(0x77, Instruction { name: "???".to_string(), clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x78, Instruction { name: "SEI".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: SEI});
//         m.insert(0x79, Instruction { name: "ADC".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABY, function: ADC});
//         m.insert(0x7A, Instruction { name: "???".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP});
//         m.insert(0x7B, Instruction { name: "???".to_string(), clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x7C, Instruction { name: "???".to_string(), clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP});
//         m.insert(0x7D, Instruction { name: "ADC".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABX, function: ADC});
//         m.insert(0x7E, Instruction { name: "ROR".to_string(), clock_cycles: 7, addr_mode: AddressingMode::ABX, function: ROR});
//         m.insert(0x7F, Instruction { name: "???".to_string(), clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x80, Instruction { name: "???".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP});
//         m.insert(0x81, Instruction { name: "STA".to_string(), clock_cycles: 6, addr_mode: AddressingMode::IZX, function: STA});
//         m.insert(0x82, Instruction { name: "???".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP});
//         m.insert(0x83, Instruction { name: "???".to_string(), clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x84, Instruction { name: "STY".to_string(), clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: STY});
//         m.insert(0x85, Instruction { name: "STA".to_string(), clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: STA});
//         m.insert(0x86, Instruction { name: "STX".to_string(), clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: STX});
//         m.insert(0x87, Instruction { name: "???".to_string(), clock_cycles: 3, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x88, Instruction { name: "DEY".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: DEY});
//         m.insert(0x89, Instruction { name: "???".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP});
//         m.insert(0x8A, Instruction { name: "TXA".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: TXA});
//         m.insert(0x8B, Instruction { name: "???".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x8C, Instruction { name: "STY".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABS, function: STY});
//         m.insert(0x8D, Instruction { name: "STA".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABS, function: STA});
//         m.insert(0x8E, Instruction { name: "STX".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABS, function: STX});
//         m.insert(0x8F, Instruction { name: "???".to_string(), clock_cycles: 4, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x90, Instruction { name: "BCC".to_string(), clock_cycles: 2, addr_mode: AddressingMode::REL, function: BCC});
//         m.insert(0x91, Instruction { name: "STA".to_string(), clock_cycles: 6, addr_mode: AddressingMode::IZY, function: STA});
//         m.insert(0x92, Instruction { name: "???".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x93, Instruction { name: "???".to_string(), clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x94, Instruction { name: "STY".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ZPX, function: STY});
//         m.insert(0x95, Instruction { name: "STA".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ZPX, function: STA});
//         m.insert(0x96, Instruction { name: "STX".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ZPY, function: STX});
//         m.insert(0x97, Instruction { name: "???".to_string(), clock_cycles: 4, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x98, Instruction { name: "TYA".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: TYA});
//         m.insert(0x99, Instruction { name: "STA".to_string(), clock_cycles: 5, addr_mode: AddressingMode::ABY, function: STA});
//         m.insert(0x9A, Instruction { name: "TXS".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: TXS});
//         m.insert(0x9B, Instruction { name: "???".to_string(), clock_cycles: 5, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x9C, Instruction { name: "???".to_string(), clock_cycles: 5, addr_mode: AddressingMode::IMP, function: NOP});
//         m.insert(0x9D, Instruction { name: "STA".to_string(), clock_cycles: 5, addr_mode: AddressingMode::ABX, function: STA});
//         m.insert(0x9E, Instruction { name: "???".to_string(), clock_cycles: 5, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0x9F, Instruction { name: "???".to_string(), clock_cycles: 5, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0xA0, Instruction { name: "LDY".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMM, function: LDY});
//         m.insert(0xA1, Instruction { name: "LDA".to_string(), clock_cycles: 6, addr_mode: AddressingMode::IZX, function: LDA});
//         m.insert(0xA2, Instruction { name: "LDX".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMM, function: LDX});
//         m.insert(0xA3, Instruction { name: "???".to_string(), clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0xA4, Instruction { name: "LDY".to_string(), clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: LDY});
//         m.insert(0xA5, Instruction { name: "LDA".to_string(), clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: LDA});
//         m.insert(0xA6, Instruction { name: "LDX".to_string(), clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: LDX});
//         m.insert(0xA7, Instruction { name: "???".to_string(), clock_cycles: 3, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0xA8, Instruction { name: "TAY".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: TAY});
//         m.insert(0xA9, Instruction { name: "LDA".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMM, function: LDA});
//         m.insert(0xAA, Instruction { name: "TAX".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: TAX});
//         m.insert(0xAB, Instruction { name: "???".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0xAC, Instruction { name: "LDY".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABS, function: LDY});
//         m.insert(0xAD, Instruction { name: "LDA".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABS, function: LDA});
//         m.insert(0xAE, Instruction { name: "LDX".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABS, function: LDX});
//         m.insert(0xAF, Instruction { name: "???".to_string(), clock_cycles: 4, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0xB0, Instruction { name: "BCS".to_string(), clock_cycles: 2, addr_mode: AddressingMode::REL, function: BCS});
//         m.insert(0xB1, Instruction { name: "LDA".to_string(), clock_cycles: 5, addr_mode: AddressingMode::IZY, function: LDA});
//         m.insert(0xB2, Instruction { name: "???".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0xB3, Instruction { name: "???".to_string(), clock_cycles: 5, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0xB4, Instruction { name: "LDY".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ZPX, function: LDY});
//         m.insert(0xB5, Instruction { name: "LDA".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ZPX, function: LDA});
//         m.insert(0xB6, Instruction { name: "LDX".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ZPY, function: LDX});
//         m.insert(0xB7, Instruction { name: "???".to_string(), clock_cycles: 4, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0xB8, Instruction { name: "CLV".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: CLV});
//         m.insert(0xB9, Instruction { name: "LDA".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABY, function: LDA});
//         m.insert(0xBA, Instruction { name: "TSX".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: TSX});
//         m.insert(0xBB, Instruction { name: "???".to_string(), clock_cycles: 4, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0xBC, Instruction { name: "LDY".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABX, function: LDY});
//         m.insert(0xBD, Instruction { name: "LDA".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABX, function: LDA});
//         m.insert(0xBE, Instruction { name: "LDX".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABY, function: LDX});
//         m.insert(0xBF, Instruction { name: "???".to_string(), clock_cycles: 4, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0xC0, Instruction { name: "CPY".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMM, function: CPY});
//         m.insert(0xC1, Instruction { name: "CMP".to_string(), clock_cycles: 6, addr_mode: AddressingMode::IZX, function: CMP});
//         m.insert(0xC2, Instruction { name: "???".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP});
//         m.insert(0xC3, Instruction { name: "???".to_string(), clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0xC4, Instruction { name: "CPY".to_string(), clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: CPY});
//         m.insert(0xC5, Instruction { name: "CMP".to_string(), clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: CMP});
//         m.insert(0xC6, Instruction { name: "DEC".to_string(), clock_cycles: 5, addr_mode: AddressingMode::ZP0, function: DEC});
//         m.insert(0xC7, Instruction { name: "???".to_string(), clock_cycles: 5, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0xC8, Instruction { name: "INY".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: INY});
//         m.insert(0xC9, Instruction { name: "CMP".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMM, function: CMP});
//         m.insert(0xCA, Instruction { name: "DEX".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: DEX});
//         m.insert(0xCB, Instruction { name: "???".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0xCC, Instruction { name: "CPY".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABS, function: CPY});
//         m.insert(0xCD, Instruction { name: "CMP".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABS, function: CMP});
//         m.insert(0xCE, Instruction { name: "DEC".to_string(), clock_cycles: 6, addr_mode: AddressingMode::ABS, function: DEC});
//         m.insert(0xCF, Instruction { name: "???".to_string(), clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0xD0, Instruction { name: "BNE".to_string(), clock_cycles: 2, addr_mode: AddressingMode::REL, function: BNE});
//         m.insert(0xD1, Instruction { name: "CMP".to_string(), clock_cycles: 5, addr_mode: AddressingMode::IZY, function: CMP});
//         m.insert(0xD2, Instruction { name: "???".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0xD3, Instruction { name: "???".to_string(), clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0xD4, Instruction { name: "???".to_string(), clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP});
//         m.insert(0xD5, Instruction { name: "CMP".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ZPX, function: CMP});
//         m.insert(0xD6, Instruction { name: "DEC".to_string(), clock_cycles: 6, addr_mode: AddressingMode::ZPX, function: DEC});
//         m.insert(0xD7, Instruction { name: "???".to_string(), clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0xD8, Instruction { name: "CLD".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: CLD});
//         m.insert(0xD9, Instruction { name: "CMP".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABY, function: CMP});
//         m.insert(0xDA, Instruction { name: "NOP".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP});
//         m.insert(0xDB, Instruction { name: "???".to_string(), clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0xDC, Instruction { name: "???".to_string(), clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP});
//         m.insert(0xDD, Instruction { name: "CMP".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABX, function: CMP});
//         m.insert(0xDE, Instruction { name: "DEC".to_string(), clock_cycles: 7, addr_mode: AddressingMode::ABX, function: DEC});
//         m.insert(0xDF, Instruction { name: "???".to_string(), clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0xE0, Instruction { name: "CPX".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMM, function: CPX});
//         m.insert(0xE1, Instruction { name: "SBC".to_string(), clock_cycles: 6, addr_mode: AddressingMode::IZX, function: SBC});
//         m.insert(0xE2, Instruction { name: "???".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP});
//         m.insert(0xE3, Instruction { name: "???".to_string(), clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0xE4, Instruction { name: "CPX".to_string(), clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: CPX});
//         m.insert(0xE5, Instruction { name: "SBC".to_string(), clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: SBC});
//         m.insert(0xE6, Instruction { name: "INC".to_string(), clock_cycles: 5, addr_mode: AddressingMode::ZP0, function: INC});
//         m.insert(0xE7, Instruction { name: "???".to_string(), clock_cycles: 5, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0xE8, Instruction { name: "INX".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: INX});
//         m.insert(0xE9, Instruction { name: "SBC".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMM, function: SBC});
//         m.insert(0xEA, Instruction { name: "NOP".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP});
//         m.insert(0xEB, Instruction { name: "???".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: SBC});
//         m.insert(0xEC, Instruction { name: "CPX".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABS, function: CPX});
//         m.insert(0xED, Instruction { name: "SBC".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABS, function: SBC});
//         m.insert(0xEE, Instruction { name: "INC".to_string(), clock_cycles: 6, addr_mode: AddressingMode::ABS, function: INC});
//         m.insert(0xEF, Instruction { name: "???".to_string(), clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0xF0, Instruction { name: "BEQ".to_string(), clock_cycles: 2, addr_mode: AddressingMode::REL, function: BEQ});
//         m.insert(0xF1, Instruction { name: "SBC".to_string(), clock_cycles: 5, addr_mode: AddressingMode::IZY, function: SBC});
//         m.insert(0xF2, Instruction { name: "???".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0xF3, Instruction { name: "???".to_string(), clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0xF4, Instruction { name: "???".to_string(), clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP});
//         m.insert(0xF5, Instruction { name: "SBC".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ZPX, function: SBC});
//         m.insert(0xF6, Instruction { name: "INC".to_string(), clock_cycles: 6, addr_mode: AddressingMode::ZPX, function: INC});
//         m.insert(0xF7, Instruction { name: "???".to_string(), clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0xF8, Instruction { name: "SED".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: SED});
//         m.insert(0xF9, Instruction { name: "SBC".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABY, function: SBC});
//         m.insert(0xFA, Instruction { name: "NOP".to_string(), clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP});
//         m.insert(0xFB, Instruction { name: "???".to_string(), clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX});
//         m.insert(0xFC, Instruction { name: "???".to_string(), clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP});
//         m.insert(0xFD, Instruction { name: "SBC".to_string(), clock_cycles: 4, addr_mode: AddressingMode::ABX, function: SBC});
//         m.insert(0xFE, Instruction { name: "INC".to_string(), clock_cycles: 7, addr_mode: AddressingMode::ABX, function: INC});
//         m.insert(0xFF, Instruction { name: "???".to_string(), clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX});
        m
    };
}


#[allow(non_snake_case)]
fn ADC(cpu :&mut Cpu6502) -> u8 {
    // Grab the data we're adding to the acc
    cpu.fetch();

    // Add is performed in 16-bit domain for emulation to capture any
	// carry bit, which will exist in bit 8 of the 16-bit word
    cpu.temp = cpu.acc as u16 + cpu.fetched as u16 + cpu.get_flag(Flags::C) as u16;
    

    // The carry flag out exists in the high order byte bit 0
    cpu.set_flag(Flags::C, cpu.temp > 255);

	// The Zero flag is set if the result is 0
	cpu.set_flag(Flags::Z, (cpu.temp & 0x00FF) == 0);
	
	// The signed Overflow flag is set based on all that up there! :D
	cpu.set_flag(Flags::V, (!(cpu.acc as u16 ^ cpu.fetched as u16) & (cpu.acc as u16 ^ cpu.temp as u16)) & 0x0080 == 0x0080);
	
	// The negative flag is set to the most significant bit of the result
	cpu.set_flag(Flags::N, cpu.temp & 0x80 == 0x80);

	// Load the result into the accumulator (it's 8-bit dont forget!)
	cpu.acc = (cpu.temp & 0x00FF) as u8;
	
	// This instruction has the potential to require an additional clock cycle
	1
}

// Instruction: Bitwise Logic AND
// Function:    A = A & M
// Flags Out:   N, Z
#[allow(non_snake_case)]
fn AND(cpu :&mut Cpu6502) -> u8 {
    cpu.fetch();
    cpu.acc = cpu.acc & cpu.fetched;
    cpu.set_flag(Flags::Z, cpu.acc == 0x00);
    cpu.set_flag(Flags::N, cpu.acc & 0x80 == 0x80);
    1
}

// Instruction: Arithmetic Shift Left
// Function:    A = C <- (A << 1) <- 0
// Flags Out:   N, Z, C
#[allow(non_snake_case)]
fn ASL(cpu :&mut Cpu6502) -> u8 {
    cpu.fetch();
    cpu.temp = (cpu.fetched as u16) << 1;
    cpu.set_flag(Flags::C, (cpu.temp & 0xFF00) > 0);
    cpu.set_flag(Flags::Z, (cpu.temp & 0x00FF) == 0);
    cpu.set_flag(Flags::N, (cpu.temp & 0x80) == 0x80);
    if cpu.addressing_mode == AddressingMode::IMP {
        cpu.acc = (cpu.temp & 0x00FF) as u8;
    } else {
        cpu.write_bus(cpu.addr_abs, (cpu.temp & 0x00FF) as u8)
    }
    0
}


// Instruction: Branch if Carry Clear
// Function:    if(C == 0) pc = address 
#[allow(non_snake_case)]
fn BCC(cpu :&mut Cpu6502) -> u8 {
    if cpu.get_flag(Flags::C) == 0 {
        cpu.cycles += 1;
        cpu.addr_abs = cpu.program_counter + cpu.addr_rel;

        if ((cpu.addr_abs & 0xFF00) != (cpu.program_counter & 0xFF00)) {
            cpu.cycles += 1;
        }
        cpu.program_counter = cpu.addr_abs;
    }
    0
}


// Instruction: Branch if Carry Set
// Function:    if(C == 1) pc = address
#[allow(non_snake_case)]
fn BCS(cpu :&mut Cpu6502) -> u8 {
    if cpu.get_flag(Flags::C) == 1 {
        cpu.cycles += 1;
        cpu.addr_abs = cpu.program_counter + cpu.addr_rel;

        if ((cpu.addr_abs & 0xFF00) != (cpu.program_counter & 0xFF00)) {
            cpu.cycles += 1;
        }
        cpu.program_counter = cpu.addr_abs;
    }
    0
}

// Instruction: Branch if Equal
// Function:    if(Z == 1) pc = address
#[allow(non_snake_case)]
fn BEQ(cpu :&mut Cpu6502) -> u8 {
    if cpu.get_flag(Flags::Z) == 1 {
        cpu.cycles += 1;
        cpu.addr_abs = cpu.program_counter + cpu.addr_rel;

        if ((cpu.addr_abs & 0xFF00) != (cpu.program_counter & 0xFF00)) {
            cpu.cycles += 1;
        }
        cpu.program_counter = cpu.addr_abs;
    }
    0
}

#[allow(non_snake_case)]
fn BIT(cpu :&mut Cpu6502) -> u8 {
    cpu.fetch();
    cpu.temp = (cpu.acc & cpu.fetched) as u16;
    cpu.set_flag(Flags::Z, (cpu.temp & 0x00FF) == 0x00);
    cpu.set_flag(Flags::N, (cpu.fetched & (1 << 7)) > 0);
    cpu.set_flag(Flags::V, (cpu.fetched & (1 << 6)) > 0);
    0
}


// Instruction: Branch if Negative
// Function:    if(N == 1) pc = address
#[allow(non_snake_case)]
fn BMI(cpu :&mut Cpu6502) -> u8 {
    if cpu.get_flag(Flags::N) == 1 {
        cpu.cycles += 1;
        cpu.addr_abs = cpu.program_counter + cpu.addr_rel;
        if ((cpu.addr_abs & 0xFF00) != (cpu.program_counter & 0xFF00)) {
            cpu.cycles += 1;
        }
        cpu.program_counter = cpu.addr_abs;
    }
    0
}

// Instruction: Branch if Not Equal
// Function:    if(Z == 0) pc = address
#[allow(non_snake_case)]
fn BNE(cpu :&mut Cpu6502) -> u8 {
    if cpu.get_flag(Flags::Z) == 0 {
        cpu.cycles += 1;
        cpu.addr_abs = cpu.program_counter + cpu.addr_rel;
        if ((cpu.addr_abs & 0xFF00) != (cpu.program_counter & 0xFF00)) {
            cpu.cycles += 1;
        }
        cpu.program_counter = cpu.addr_abs;
    }
    0
}

// Instruction: Branch if Positive
// Function:    if(N == 0) pc = address
#[allow(non_snake_case)]
fn BPL(cpu :&mut Cpu6502) -> u8 {
    if cpu.get_flag(Flags::N) == 0 {
        cpu.cycles += 1;
        cpu.addr_abs = cpu.program_counter + cpu.addr_rel;
        if ((cpu.addr_abs & 0xFF00) != (cpu.program_counter & 0xFF00)) {
            cpu.cycles += 1;
        }
        cpu.program_counter = cpu.addr_abs;
    }
    0
}

// Instruction: Break
// Function:    Program Sourced Interrupt
#[allow(non_snake_case)]
fn BRK(cpu :&mut Cpu6502) -> u8 {
    cpu.program_counter += 1;
    cpu.set_flag(Flags::I, true);
    cpu.write_bus(0x0100 + cpu.stack_pointer as u16, ((cpu.program_counter >> 8) & 0x00FF) as u8);
    cpu.stack_pointer -= 1;
    cpu.write_bus(0x0100 + cpu.stack_pointer as u16, (cpu.program_counter & 0x00FF) as u8);
    cpu.stack_pointer -= 1;
    
    cpu.set_flag(Flags::B, true);
    cpu.write_bus(0x0100 + cpu.stack_pointer as u16, cpu.status);
    cpu.stack_pointer -= 1;
    cpu.set_flag(Flags::B, false);
    
    cpu.program_counter = cpu.read_bus_two_bytes(0xFFFE);
    0
}

// Instruction: Branch if Overflow Clear
// Function:    if(V == 0) pc = address
#[allow(non_snake_case)]
fn BVC(cpu :&mut Cpu6502) -> u8 {
    if cpu.get_flag(Flags::V) == 0 {
        cpu.cycles += 1;
        cpu.addr_abs = cpu.program_counter + cpu.addr_rel;
        if ((cpu.addr_abs & 0xFF00) != (cpu.program_counter & 0xFF00)) {
            cpu.cycles += 1;
        }
        cpu.program_counter = cpu.addr_abs;
    }
    0
}

// Instruction: Branch if Overflow Set
// Function:    if(V == 1) pc = address
#[allow(non_snake_case)]
fn BVS(cpu :&mut Cpu6502) -> u8 {
    if cpu.get_flag(Flags::V) == 1 {
        cpu.cycles += 1;
        cpu.addr_abs = cpu.program_counter + cpu.addr_rel;
        if ((cpu.addr_abs & 0xFF00) != (cpu.program_counter & 0xFF00)) {
            cpu.cycles += 1;
        }
        cpu.program_counter = cpu.addr_abs;
    }
    0
}


// Instruction: Clear Carry Flag
// Function:    C = 0
#[allow(non_snake_case)]
fn CLC(cpu :&mut Cpu6502) -> u8 {
    cpu.set_flag(Flags::C, false);
    0
}

// Instruction: Clear Decimal Flag
// Function:    D = 0
#[allow(non_snake_case)]
fn CLD(cpu :&mut Cpu6502) -> u8 {
    cpu.set_flag(Flags::D, false);
    0
}

// Instruction: Disable Interrupts / Clear Interrupt Flag
// Function:    I = 0
#[allow(non_snake_case)]
fn CLI(cpu :&mut Cpu6502) -> u8 {
    cpu.set_flag(Flags::I, false);
    0
}

// Instruction: Clear Overflow Flag
// Function:    V = 0
#[allow(non_snake_case)]
fn CLV(cpu :&mut Cpu6502) -> u8 {
    cpu.set_flag(Flags::V, false);
    0
}

// Instruction: Compare Accumulator
// Function:    C <- A >= M      Z <- (A - M) == 0
// Flags Out:   N, C, Z
#[allow(non_snake_case)]
fn CMP(cpu :&mut Cpu6502) -> u8 {
    cpu.fetch();
    cpu.temp = (cpu.acc - cpu.fetched) as u16;
    cpu.set_flag(Flags::C, cpu.acc >= cpu.fetched);
    cpu.set_flag(Flags::Z, (cpu.temp & 0x00FF) == 0x0000);
    cpu.set_flag(Flags::N, cpu.temp & 0x0080 > 0);
    0
}

// Instruction: Compare X Register
// Function:    C <- X >= M      Z <- (X - M) == 0
// Flags Out:   N, C, Z
#[allow(non_snake_case)]
fn CPX(cpu :&mut Cpu6502) -> u8 {
    cpu.fetch();
    cpu.temp = (cpu.x_reg - cpu.fetched) as u16;
    cpu.set_flag(Flags::C, cpu.x_reg >= cpu.fetched);
    cpu.set_flag(Flags::Z, (cpu.temp & 0x00FF) == 0x0000);
    cpu.set_flag(Flags::N, cpu.temp & 0x0080 > 0);
    0
}

// Instruction: Compare Y Register
// Function:    C <- Y >= M      Z <- (Y - M) == 0
// Flags Out:   N, C, Z
#[allow(non_snake_case)]
fn CPY(cpu :&mut Cpu6502) -> u8 {
    cpu.fetch();
    cpu.temp = (cpu.y_reg - cpu.fetched) as u16;
    cpu.set_flag(Flags::C, cpu.y_reg >= cpu.fetched);
    cpu.set_flag(Flags::Z, (cpu.temp & 0x00FF) == 0x0000);
    cpu.set_flag(Flags::N, cpu.temp & 0x0080 > 0);
    0
}

// Instruction: Decrement Value at Memory Location
// Function:    M = M - 1
// Flags Out:   N, Z
#[allow(non_snake_case)]
fn DEC(cpu :&mut Cpu6502) -> u8 {
    cpu.fetch();
    cpu.temp = (cpu.fetched - 1) as u16;
    cpu.write_bus(cpu.addr_abs, (cpu.temp & 0x00FF) as u8);
    cpu.set_flag(Flags::Z, (cpu.temp & 0x00FF) == 0x0000);
    cpu.set_flag(Flags::N, cpu.temp & 0x0080 > 0);
    0
}

// Instruction: Decrement X Register
// Function:    X = X - 1
// Flags Out:   N, Z
#[allow(non_snake_case)]
fn DEX(cpu :&mut Cpu6502) -> u8 {
    cpu.x_reg -= 1;
    cpu.set_flag(Flags::Z, cpu.x_reg == 0x00);
    cpu.set_flag(Flags::N, cpu.x_reg & 0x80 > 0);
    0
}

// Instruction: Decrement Y Register
// Function:    Y = Y - 1
// Flags Out:   N, Z
#[allow(non_snake_case)]
fn DEY(cpu :&mut Cpu6502) -> u8 {
    cpu.y_reg -= 1;
    cpu.set_flag(Flags::Z, cpu.y_reg == 0x00);
    cpu.set_flag(Flags::N, cpu.y_reg & 0x80 > 0);
    0
}

// Instruction: Bitwise Logic XOR
// Function:    A = A xor M
// Flags Out:   N, Z
#[allow(non_snake_case)]
fn EOR(cpu :&mut Cpu6502) -> u8 {
    cpu.fetch();
    cpu.acc = cpu.acc ^ cpu.fetched;
    cpu.set_flag(Flags::Z, cpu.acc == 0x00);
    cpu.set_flag(Flags::N, cpu.acc & 0x80 > 0);
    1
}

// Instruction: Increment Value at Memory Location
// Function:    M = M + 1
// Flags Out:   N, Z
#[allow(non_snake_case)]
fn INC(cpu :&mut Cpu6502) -> u8 {
    cpu.fetch();
    cpu.temp = (cpu.fetched + 1) as u16;
    cpu.write_bus(cpu.addr_abs, (cpu.temp & 0x00FF) as u8);
    cpu.set_flag(Flags::Z, (cpu.temp & 0x00FF) == 0x0000);
    cpu.set_flag(Flags::N, cpu.temp & 0x0080 > 0);
    0
}

// Instruction: Increment X Register
// Function:    X = X + 1
// Flags Out:   N, Z
#[allow(non_snake_case)]
fn INX(cpu :&mut Cpu6502) -> u8 {
    cpu.x_reg += 1;
    cpu.set_flag(Flags::Z, cpu.x_reg == 0x00);
    cpu.set_flag(Flags::N, cpu.x_reg & 0x80 > 0);
    0
}

// Instruction: Increment Y Register
// Function:    Y = Y + 1
// Flags Out:   N, Z
#[allow(non_snake_case)]
fn INY(cpu :&mut Cpu6502) -> u8 {
    cpu.x_reg += 1;
    cpu.set_flag(Flags::Z, cpu.y_reg == 0x00);
    cpu.set_flag(Flags::N, cpu.y_reg & 0x80 > 0);
    0
}

#[allow(non_snake_case)]
fn JMP(cpu :&mut Cpu6502) -> u8 {
    cpu.program_counter = cpu.addr_abs;
    0
}

#[allow(non_snake_case)]
fn JSR(cpu :&mut Cpu6502) -> u8 {
    cpu.program_counter -= 1;
    cpu.write_bus_two_bytes(0x0100 + cpu.stack_pointer as u16, cpu.program_counter);
    cpu.program_counter = cpu.addr_abs;
    0
}

#[allow(non_snake_case)]
fn LDA(cpu :&mut Cpu6502) -> u8 {
    cpu.fetch();
    cpu.acc = cpu.fetched;
    cpu.set_flag(Flags::Z, cpu.acc == 0x00);
    cpu.set_flag(Flags::N, cpu.acc & 0x80 > 1);
    1
}

#[allow(non_snake_case)]
fn LDX(cpu :&mut Cpu6502) -> u8 {
    cpu.fetch();
    cpu.x_reg = cpu.fetched;
    cpu.set_flag(Flags::Z, cpu.x_reg == 0x00);
    cpu.set_flag(Flags::N, cpu.x_reg & 0x80 > 1);
    1
}

#[allow(non_snake_case)]
fn LDY(cpu :&mut Cpu6502) -> u8 {
    cpu.fetch();
    cpu.y_reg = cpu.fetched;
    cpu.set_flag(Flags::Z, cpu.y_reg == 0x00);
    cpu.set_flag(Flags::N, cpu.y_reg & 0x80 > 1);
    1
}

#[allow(non_snake_case)]
fn LSR(cpu :&mut Cpu6502) -> u8 {
    cpu.fetch();
    cpu.set_flag(Flags::C, cpu.fetched & 1 > 0);
    cpu.temp = (cpu.fetched >> 1) as u16;
    cpu.set_flag(Flags::Z, (cpu.temp & 0x00FF) == 0x0000);
    cpu.set_flag(Flags::N, cpu.temp & 0x0080 > 0);
    if cpu.addressing_mode == AddressingMode::IMP {
        cpu.acc = (cpu.temp & 0x00FF) as u8;
    } else {
        cpu.write_bus(cpu.addr_abs, (cpu.temp & 0x00FF) as u8)
    }
    0
}

#[allow(non_snake_case)]
fn NOP(cpu :&mut Cpu6502) -> u8 {
    match cpu.opcode {
        0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => { return 1; }
        _ => { return 0; }
    }
}

// Instruction: Bitwise Logic OR
// Function:    A = A | M
// Flags Out:   N, Z
#[allow(non_snake_case)]
fn ORA(cpu :&mut Cpu6502) -> u8 {
0
}

// Instruction: Push Accumulator to Stack
// Function:    A -> stack
#[allow(non_snake_case)]
fn PHA(cpu :&mut Cpu6502) -> u8 {
0
}

// Instruction: Push Status Register to Stack
// Function:    status -> stack
// Note:        Break flag is set to 1 before push
#[allow(non_snake_case)]
fn PHP(cpu :&mut Cpu6502) -> u8 {
0
}

// Instruction: Pop Accumulator off Stack
// Function:    A <- stack
// Flags Out:   N, Z
#[allow(non_snake_case)]
fn PLA(cpu :&mut Cpu6502) -> u8 {
0
}

#[allow(non_snake_case)]
fn PLP(cpu :&mut Cpu6502) -> u8 {
0
}

#[allow(non_snake_case)]
fn ROL(cpu :&mut Cpu6502) -> u8 {
0
}

#[allow(non_snake_case)]
fn ROR(cpu :&mut Cpu6502) -> u8 {
0
}

#[allow(non_snake_case)]
fn RTI(cpu :&mut Cpu6502) -> u8 {
0
}

#[allow(non_snake_case)]
fn RTS(cpu :&mut Cpu6502) -> u8 {
0
}

#[allow(non_snake_case)]
fn SBC(cpu :&mut Cpu6502) -> u8 {
0
}

// Instruction: Set Carry Flag
// Function:    C = 1
#[allow(non_snake_case)]
fn SEC(cpu :&mut Cpu6502) -> u8 {
0
}

// Instruction: Set Decimal Flag
// Function:    D = 1
#[allow(non_snake_case)]
fn SED(cpu :&mut Cpu6502) -> u8 {
0
}

// Instruction: Set Interrupt Flag / Enable Interrupts
// Function:    I = 1
#[allow(non_snake_case)]
fn SEI(cpu :&mut Cpu6502) -> u8 {
0
}

// Instruction: Store Accumulator at Address
// Function:    M = A
#[allow(non_snake_case)]
fn STA(cpu :&mut Cpu6502) -> u8 {
0
}

// Instruction: Store X Register at Address
// Function:    M = X
#[allow(non_snake_case)]
fn STX(cpu :&mut Cpu6502) -> u8 {
0
}

// Instruction: Store Y Register at Address
// Function:    M = Y
#[allow(non_snake_case)]
fn STY(cpu :&mut Cpu6502) -> u8 {
0
}

// Instruction: Transfer Accumulator to X Register
// Function:    X = A
// Flags Out:   N, Z
#[allow(non_snake_case)]
fn TAX(cpu :&mut Cpu6502) -> u8 {
0
}

// Instruction: Transfer Accumulator to Y Register
// Function:    Y = A
// Flags Out:   N, Z
#[allow(non_snake_case)]
fn TAY(cpu :&mut Cpu6502) -> u8 {
0
}

// Instruction: Transfer Stack Pointer to X Register
// Function:    X = stack pointer
// Flags Out:   N, Z
#[allow(non_snake_case)]
fn TSX(cpu :&mut Cpu6502) -> u8 {
0
}

// Instruction: Transfer X Register to Accumulator
// Function:    A = X
// Flags Out:   N, Z
#[allow(non_snake_case)]
fn TXA(cpu :&mut Cpu6502) -> u8 {
0
}


// Instruction: Transfer X Register to Stack Pointer
// Function:    stack pointer = X
#[allow(non_snake_case)]
fn TXS(cpu :&mut Cpu6502) -> u8 {
0
}

// Instruction: Transfer Y Register to Accumulator
// Function:    A = Y
// Flags Out:   N, Z
#[allow(non_snake_case)]
fn TYA(cpu :&mut Cpu6502) -> u8 {
    0
}

#[allow(non_snake_case)]
fn XXX(cpu :&mut Cpu6502) -> u8 {
    0
}

fn process_instruction_type(instruction: &Instruction, cpu: &mut Cpu6502) {

}