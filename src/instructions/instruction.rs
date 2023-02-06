use crate::cpu::Cpu6502;
use super::instruction_functions::*;

#[derive(Clone, Copy)]
pub struct Instruction {
    pub(crate) name: &'static str,
    pub(crate) addr_mode: AddressingMode,
    pub(crate) function: for<'r> fn(&'r mut Cpu6502),
    pub(crate) clock_cycles: u8,
}

impl Default for Instruction {
    fn default() -> Self {
        Instruction {
            name: "???",
            addr_mode: AddressingMode::YYY,
            function: XXX,
            clock_cycles: 0,
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum AddressingMode {
    IMP,
    IMM,
    ZP0,
    ZPX,
    ZPY,
    REL,
    ABS,
    ABX,
    ABY,
    IND,
    IZX,
    IZY,
    YYY,
}

// Core: Modify the cpu's registers according to the provided instructions addressing mode
// returns 0 or 1 depending if an extra cycle is required
pub fn process_instruction_addressing_mode(cpu: &mut Cpu6502) -> u8 {
    match cpu.instruction.addr_mode {
        // Address Mode: Implied
        // There is no additional data required for this instruction. The instruction
        // does something very simple like like sets a status bit. However, we will
        // target the accumulator, for instructions like PHA
        AddressingMode::IMP => {
            cpu.fetched = cpu.acc;
            0
        }

        // Address Mode: Immediate
        // The instruction expects the next byte to be used as a value, so we'll prep
        // the read address to point to the next byte
        AddressingMode::IMM => {
            cpu.addr_abs = cpu.pc;
            cpu.pc += 1;
            0
        }

        // Address Mode: Zero Page
        // To save program bytes, zero page addressing allows you to absolutely address
        // a location in first 0xFF bytes of address range. Clearly this only requires
        // one byte instead of the usual two.
        AddressingMode::ZP0 => {
            cpu.addr_abs = cpu.read_bus(cpu.pc) as u16;
            cpu.pc += 1;
            cpu.addr_abs &= 0x00FF;
            0
        }

        // Address Mode: Zero Page with X Offset
        // Fundamentally the same as Zero Page addressing, but the contents of the X Register
        // is added to the supplied single byte address. This is useful for iterating through
        // ranges within the first page.
        AddressingMode::ZPX => {
            let (r, _) = cpu.read_bus(cpu.pc).overflowing_add(cpu.x_reg);
            cpu.addr_abs = r as u16;
            cpu.pc += 1;
            cpu.addr_abs &= 0x00FF;
            0
        }

        // Address Mode: Zero Page with Y Offset
        // Same as above but uses Y Register for offset
        AddressingMode::ZPY => {
            let (r, _) = cpu.read_bus(cpu.pc).overflowing_add(cpu.y_reg);
            cpu.addr_abs = r as u16;
            cpu.pc += 1;
            cpu.addr_abs &= 0x00FF;
            0
        }

        // Address Mode: Relative
        // This address mode is exclusive to branch instructions. The address
        // must reside within -128 to +127 of the branch instruction, i.e.
        // you cant directly branch to any address in the addressable range.
        AddressingMode::REL => {
            cpu.addr_rel = cpu.read_bus(cpu.pc) as u16;
            let (r, _) = cpu.pc.overflowing_add(1);
            cpu.pc = r;
            if cpu.addr_rel & 0x80 > 0 {
                cpu.addr_rel |= 0xFF00;
            }
            0
        }

        // Address Mode: Absolute
        // A full 16-bit address is loaded and used
        AddressingMode::ABS => {
            cpu.addr_abs = cpu.read_bus_two_bytes(cpu.pc);
            cpu.pc += 2;
            0
        }

        // Address Mode: Absolute with X Offset
        // Fundamentally the same as absolute addressing, but the contents of the X Register
        // is added to the supplied two byte address. If the resulting address changes
        // the page, an additional clock cycle is required
        AddressingMode::ABX => {
            let addr: u16 = cpu.read_bus_two_bytes(cpu.pc);
            cpu.pc += 2;

            cpu.addr_abs = addr;
            let (r, _) = cpu.addr_abs.overflowing_add(cpu.x_reg as u16);
            cpu.addr_abs = r;
            check_page_change(&addr, &cpu.addr_abs)
        }

        // Address Mode: Absolute with Y Offset
        // Fundamentally the same as absolute addressing, but the contents of the Y Register
        // is added to the supplied two byte address. If the resulting address changes
        // the page, an additional clock cycle is required
        AddressingMode::ABY => {
            let addr: u16 = cpu.read_bus_two_bytes(cpu.pc);
            cpu.pc += 2;

            cpu.addr_abs = addr;
            let (r, _) = cpu.addr_abs.overflowing_add(cpu.y_reg as u16);
            cpu.addr_abs = r;
            check_page_change(&addr, &cpu.addr_abs)
        }

        // Address Mode: Indirect
        // The supplied 16-bit address is read to get the actual 16-bit address. This is
        // instruction is unusual in that it has a bug in the hardware! To emulate its
        // function accurately, we also need to emulate this bug. If the low byte of the
        // supplied address is 0xFF, then to read the high byte of the actual address
        // we need to cross a page boundary. This doesnt actually work on the chip as
        // designed, instead it wraps back around in the same page, yielding an
        // invalid actual address
        AddressingMode::IND => {
            let addr: u16 = cpu.read_bus_two_bytes(cpu.pc);
            cpu.pc += 2;
            let ptr: u16 = addr;

            if addr & 0x00FF == 0x00FF {
                cpu.addr_abs = (cpu.read_bus(ptr & 0xFF00) as u16) << 8 | cpu.read_bus(ptr) as u16;
            } else {
                cpu.addr_abs = cpu.read_bus_two_bytes(ptr);
            }
            0
        }

        // Address Mode: Indirect X
        // The supplied 8-bit address is offset by X Register to index
        // a location in page 0x00. The actual 16-bit address is read
        // from this location
        AddressingMode::IZX => {
            let t: u16 = cpu.read_bus(cpu.pc) as u16;
            cpu.pc += 1;
            let lo = cpu.read_bus((t + (cpu.x_reg as u16)) & 0x00FF) as u16;
            let hi = cpu.read_bus((t + (cpu.x_reg as u16) + 1) & 0x00FF) as u16;
            cpu.addr_abs = (hi << 8) | lo;
            0
        }

        // Address Mode: Indirect Y
        // The supplied 8-bit address indexes a location in page 0x00. From
        // here the actual 16-bit address is read, and the contents of
        // Y Register is added to it to offset it. If the offset causes a
        // change in page then an additional clock cycle is required.
        AddressingMode::IZY => {
            let t: u16 = cpu.read_bus(cpu.pc) as u16;
            cpu.pc += 1;
            let lo = cpu.read_bus(t & 0x00FF) as u16;
            let hi = cpu.read_bus((t + 1) & 0x00FF) as u16;
            cpu.addr_abs = (hi << 8) | lo;
            let (r, _) = cpu.addr_abs.overflowing_add(cpu.y_reg as u16);
            cpu.addr_abs = r;

            if cpu.addr_abs & 0xFF00 != hi << 8 {
                return 1;
            } else {
                return 0;
            }
        }
        AddressingMode::YYY => 0,
    }
}

// Helper: Check if the page has changed, return an additional clock cycle if yes
fn check_page_change(previous: &u16, current: &u16) -> u8 {
    if current & 0xFF00 != previous & 0xFF00 {
        return 1;
    }
    0
}


// BY THE POWER OF AUTS
pub const INSTRUCTION_LOOKUP: [Instruction; 0xFF +1] = [
    Instruction { name: "BRK", clock_cycles: 7, addr_mode: AddressingMode::IMP, function: BRK},
    Instruction { name: "ORA", clock_cycles: 6, addr_mode: AddressingMode::IZX, function: ORA},
    Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX}, // Unofficial
    Instruction { name: "SLO", clock_cycles: 8, addr_mode: AddressingMode::IZX, function: XXX}, // Unofficial
    Instruction { name: "NOP", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: NOP}, // Unofficial
    Instruction { name: "ORA", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: ORA},
    Instruction { name: "ASL", clock_cycles: 5, addr_mode: AddressingMode::ZP0, function: ASL},
    Instruction { name: "???", clock_cycles: 5, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "PHP", clock_cycles: 3, addr_mode: AddressingMode::IMP, function: PHP},
    Instruction { name: "ORA", clock_cycles: 2, addr_mode: AddressingMode::IMM, function: ORA},
    Instruction { name: "ASL", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: ASL},
    Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP},
    Instruction { name: "ORA", clock_cycles: 4, addr_mode: AddressingMode::ABS, function: ORA},
    Instruction { name: "ASL", clock_cycles: 6, addr_mode: AddressingMode::ABS, function: ASL},
    Instruction { name: "???", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "BPL", clock_cycles: 2, addr_mode: AddressingMode::REL, function: BPL},
    Instruction { name: "ORA", clock_cycles: 5, addr_mode: AddressingMode::IZY, function: ORA},
    Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "???", clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP},
    Instruction { name: "ORA", clock_cycles: 4, addr_mode: AddressingMode::ZPX, function: ORA},
    Instruction { name: "ASL", clock_cycles: 6, addr_mode: AddressingMode::ZPX, function: ASL},
    Instruction { name: "???", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "CLC", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: CLC},
    Instruction { name: "ORA", clock_cycles: 4, addr_mode: AddressingMode::ABY, function: ORA},
    Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP},
    Instruction { name: "???", clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP},
    Instruction { name: "ORA", clock_cycles: 4, addr_mode: AddressingMode::ABX, function: ORA},
    Instruction { name: "ASL", clock_cycles: 7, addr_mode: AddressingMode::ABX, function: ASL},
    Instruction { name: "???", clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "JSR", clock_cycles: 6, addr_mode: AddressingMode::ABS, function: JSR},
    Instruction { name: "AND", clock_cycles: 6, addr_mode: AddressingMode::IZX, function: AND},
    Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "???", clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "BIT", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: BIT},
    Instruction { name: "AND", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: AND},
    Instruction { name: "ROL", clock_cycles: 5, addr_mode: AddressingMode::ZP0, function: ROL},
    Instruction { name: "???", clock_cycles: 5, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "PLP", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: PLP},
    Instruction { name: "AND", clock_cycles: 2, addr_mode: AddressingMode::IMM, function: AND},
    Instruction { name: "ROL", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: ROL},
    Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "BIT", clock_cycles: 4, addr_mode: AddressingMode::ABS, function: BIT},
    Instruction { name: "AND", clock_cycles: 4, addr_mode: AddressingMode::ABS, function: AND},
    Instruction { name: "ROL", clock_cycles: 6, addr_mode: AddressingMode::ABS, function: ROL},
    Instruction { name: "???", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "BMI", clock_cycles: 2, addr_mode: AddressingMode::REL, function: BMI},
    Instruction { name: "AND", clock_cycles: 5, addr_mode: AddressingMode::IZY, function: AND},
    Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "???", clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP},
    Instruction { name: "AND", clock_cycles: 4, addr_mode: AddressingMode::ZPX, function: AND},
    Instruction { name: "ROL", clock_cycles: 6, addr_mode: AddressingMode::ZPX, function: ROL},
    Instruction { name: "???", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "SEC", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: SEC},
    Instruction { name: "AND", clock_cycles: 4, addr_mode: AddressingMode::ABY, function: AND},
    Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP},
    Instruction { name: "???", clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP},
    Instruction { name: "AND", clock_cycles: 4, addr_mode: AddressingMode::ABX, function: AND},
    Instruction { name: "ROL", clock_cycles: 7, addr_mode: AddressingMode::ABX, function: ROL},
    Instruction { name: "???", clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "RTI", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: RTI},
    Instruction { name: "EOR", clock_cycles: 6, addr_mode: AddressingMode::IZX, function: EOR},
    Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "???", clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "???", clock_cycles: 3, addr_mode: AddressingMode::IMP, function: NOP},
    Instruction { name: "EOR", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: EOR},
    Instruction { name: "LSR", clock_cycles: 5, addr_mode: AddressingMode::ZP0, function: LSR},
    Instruction { name: "???", clock_cycles: 5, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "PHA", clock_cycles: 3, addr_mode: AddressingMode::IMP, function: PHA},
    Instruction { name: "EOR", clock_cycles: 2, addr_mode: AddressingMode::IMM, function: EOR},
    Instruction { name: "LSR", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: LSR},
    Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "JMP", clock_cycles: 3, addr_mode: AddressingMode::ABS, function: JMP},
    Instruction { name: "EOR", clock_cycles: 4, addr_mode: AddressingMode::ABS, function: EOR},
    Instruction { name: "LSR", clock_cycles: 6, addr_mode: AddressingMode::ABS, function: LSR},
    Instruction { name: "???", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "BVC", clock_cycles: 2, addr_mode: AddressingMode::REL, function: BVC},
    Instruction { name: "EOR", clock_cycles: 5, addr_mode: AddressingMode::IZY, function: EOR},
    Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "???", clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP},
    Instruction { name: "EOR", clock_cycles: 4, addr_mode: AddressingMode::ZPX, function: EOR},
    Instruction { name: "LSR", clock_cycles: 6, addr_mode: AddressingMode::ZPX, function: LSR},
    Instruction { name: "???", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "CLI", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: CLI},
    Instruction { name: "EOR", clock_cycles: 4, addr_mode: AddressingMode::ABY, function: EOR},
    Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP},
    Instruction { name: "???", clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP},
    Instruction { name: "EOR", clock_cycles: 4, addr_mode: AddressingMode::ABX, function: EOR},
    Instruction { name: "LSR", clock_cycles: 7, addr_mode: AddressingMode::ABX, function: LSR},
    Instruction { name: "???", clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "RTS", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: RTS},
    Instruction { name: "ADC", clock_cycles: 6, addr_mode: AddressingMode::IZX, function: ADC},
    Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "???", clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "???", clock_cycles: 3, addr_mode: AddressingMode::IMP, function: NOP},
    Instruction { name: "ADC", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: ADC},
    Instruction { name: "ROR", clock_cycles: 5, addr_mode: AddressingMode::ZP0, function: ROR},
    Instruction { name: "???", clock_cycles: 5, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "PLA", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: PLA},
    Instruction { name: "ADC", clock_cycles: 2, addr_mode: AddressingMode::IMM, function: ADC},
    Instruction { name: "ROR", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: ROR},
    Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "JMP", clock_cycles: 5, addr_mode: AddressingMode::IND, function: JMP},
    Instruction { name: "ADC", clock_cycles: 4, addr_mode: AddressingMode::ABS, function: ADC},
    Instruction { name: "ROR", clock_cycles: 6, addr_mode: AddressingMode::ABS, function: ROR},
    Instruction { name: "???", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "BVS", clock_cycles: 2, addr_mode: AddressingMode::REL, function: BVS},
    Instruction { name: "ADC", clock_cycles: 5, addr_mode: AddressingMode::IZY, function: ADC},
    Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "???", clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP},
    Instruction { name: "ADC", clock_cycles: 4, addr_mode: AddressingMode::ZPX, function: ADC},
    Instruction { name: "ROR", clock_cycles: 6, addr_mode: AddressingMode::ZPX, function: ROR},
    Instruction { name: "???", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "SEI", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: SEI},
    Instruction { name: "ADC", clock_cycles: 4, addr_mode: AddressingMode::ABY, function: ADC},
    Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP},
    Instruction { name: "???", clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP},
    Instruction { name: "ADC", clock_cycles: 4, addr_mode: AddressingMode::ABX, function: ADC},
    Instruction { name: "ROR", clock_cycles: 7, addr_mode: AddressingMode::ABX, function: ROR},
    Instruction { name: "???", clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP},
    Instruction { name: "STA", clock_cycles: 6, addr_mode: AddressingMode::IZX, function: STA},
    Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP},
    Instruction { name: "???", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "STY", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: STY},
    Instruction { name: "STA", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: STA},
    Instruction { name: "STX", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: STX},
    Instruction { name: "???", clock_cycles: 3, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "DEY", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: DEY},
    Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP},
    Instruction { name: "TXA", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: TXA},
    Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "STY", clock_cycles: 4, addr_mode: AddressingMode::ABS, function: STY},
    Instruction { name: "STA", clock_cycles: 4, addr_mode: AddressingMode::ABS, function: STA},
    Instruction { name: "STX", clock_cycles: 4, addr_mode: AddressingMode::ABS, function: STX},
    Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "BCC", clock_cycles: 2, addr_mode: AddressingMode::REL, function: BCC},
    Instruction { name: "STA", clock_cycles: 6, addr_mode: AddressingMode::IZY, function: STA},
    Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "???", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "STY", clock_cycles: 4, addr_mode: AddressingMode::ZPX, function: STY},
    Instruction { name: "STA", clock_cycles: 4, addr_mode: AddressingMode::ZPX, function: STA},
    Instruction { name: "STX", clock_cycles: 4, addr_mode: AddressingMode::ZPY, function: STX},
    Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "TYA", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: TYA},
    Instruction { name: "STA", clock_cycles: 5, addr_mode: AddressingMode::ABY, function: STA},
    Instruction { name: "TXS", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: TXS},
    Instruction { name: "???", clock_cycles: 5, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "???", clock_cycles: 5, addr_mode: AddressingMode::IMP, function: NOP},
    Instruction { name: "STA", clock_cycles: 5, addr_mode: AddressingMode::ABX, function: STA},
    Instruction { name: "???", clock_cycles: 5, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "???", clock_cycles: 5, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "LDY", clock_cycles: 2, addr_mode: AddressingMode::IMM, function: LDY},
    Instruction { name: "LDA", clock_cycles: 6, addr_mode: AddressingMode::IZX, function: LDA},
    Instruction { name: "LDX", clock_cycles: 2, addr_mode: AddressingMode::IMM, function: LDX},
    Instruction { name: "???", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "LDY", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: LDY},
    Instruction { name: "LDA", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: LDA},
    Instruction { name: "LDX", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: LDX},
    Instruction { name: "???", clock_cycles: 3, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "TAY", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: TAY},
    Instruction { name: "LDA", clock_cycles: 2, addr_mode: AddressingMode::IMM, function: LDA},
    Instruction { name: "TAX", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: TAX},
    Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "LDY", clock_cycles: 4, addr_mode: AddressingMode::ABS, function: LDY},
    Instruction { name: "LDA", clock_cycles: 4, addr_mode: AddressingMode::ABS, function: LDA},
    Instruction { name: "LDX", clock_cycles: 4, addr_mode: AddressingMode::ABS, function: LDX},
    Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "BCS", clock_cycles: 2, addr_mode: AddressingMode::REL, function: BCS},
    Instruction { name: "LDA", clock_cycles: 5, addr_mode: AddressingMode::IZY, function: LDA},
    Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "???", clock_cycles: 5, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "LDY", clock_cycles: 4, addr_mode: AddressingMode::ZPX, function: LDY},
    Instruction { name: "LDA", clock_cycles: 4, addr_mode: AddressingMode::ZPX, function: LDA},
    Instruction { name: "LDX", clock_cycles: 4, addr_mode: AddressingMode::ZPY, function: LDX},
    Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "CLV", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: CLV},
    Instruction { name: "LDA", clock_cycles: 4, addr_mode: AddressingMode::ABY, function: LDA},
    Instruction { name: "TSX", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: TSX},
    Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "LDY", clock_cycles: 4, addr_mode: AddressingMode::ABX, function: LDY},
    Instruction { name: "LDA", clock_cycles: 4, addr_mode: AddressingMode::ABX, function: LDA},
    Instruction { name: "LDX", clock_cycles: 4, addr_mode: AddressingMode::ABY, function: LDX},
    Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "CPY", clock_cycles: 2, addr_mode: AddressingMode::IMM, function: CPY},
    Instruction { name: "CMP", clock_cycles: 6, addr_mode: AddressingMode::IZX, function: CMP},
    Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP},
    Instruction { name: "???", clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "CPY", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: CPY},
    Instruction { name: "CMP", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: CMP},
    Instruction { name: "DEC", clock_cycles: 5, addr_mode: AddressingMode::ZP0, function: DEC},
    Instruction { name: "???", clock_cycles: 5, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "INY", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: INY},
    Instruction { name: "CMP", clock_cycles: 2, addr_mode: AddressingMode::IMM, function: CMP},
    Instruction { name: "DEX", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: DEX},
    Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "CPY", clock_cycles: 4, addr_mode: AddressingMode::ABS, function: CPY},
    Instruction { name: "CMP", clock_cycles: 4, addr_mode: AddressingMode::ABS, function: CMP},
    Instruction { name: "DEC", clock_cycles: 6, addr_mode: AddressingMode::ABS, function: DEC},
    Instruction { name: "???", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "BNE", clock_cycles: 2, addr_mode: AddressingMode::REL, function: BNE},
    Instruction { name: "CMP", clock_cycles: 5, addr_mode: AddressingMode::IZY, function: CMP},
    Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "???", clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP},
    Instruction { name: "CMP", clock_cycles: 4, addr_mode: AddressingMode::ZPX, function: CMP},
    Instruction { name: "DEC", clock_cycles: 6, addr_mode: AddressingMode::ZPX, function: DEC},
    Instruction { name: "???", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "CLD", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: CLD},
    Instruction { name: "CMP", clock_cycles: 4, addr_mode: AddressingMode::ABY, function: CMP},
    Instruction { name: "NOP", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP},
    Instruction { name: "???", clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP},
    Instruction { name: "CMP", clock_cycles: 4, addr_mode: AddressingMode::ABX, function: CMP},
    Instruction { name: "DEC", clock_cycles: 7, addr_mode: AddressingMode::ABX, function: DEC},
    Instruction { name: "???", clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "CPX", clock_cycles: 2, addr_mode: AddressingMode::IMM, function: CPX},
    Instruction { name: "SBC", clock_cycles: 6, addr_mode: AddressingMode::IZX, function: SBC},
    Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP},
    Instruction { name: "???", clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "CPX", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: CPX},
    Instruction { name: "SBC", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: SBC},
    Instruction { name: "INC", clock_cycles: 5, addr_mode: AddressingMode::ZP0, function: INC},
    Instruction { name: "???", clock_cycles: 5, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "INX", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: INX},
    Instruction { name: "SBC", clock_cycles: 2, addr_mode: AddressingMode::IMM, function: SBC},
    Instruction { name: "NOP", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP},
    Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: SBC},
    Instruction { name: "CPX", clock_cycles: 4, addr_mode: AddressingMode::ABS, function: CPX},
    Instruction { name: "SBC", clock_cycles: 4, addr_mode: AddressingMode::ABS, function: SBC},
    Instruction { name: "INC", clock_cycles: 6, addr_mode: AddressingMode::ABS, function: INC},
    Instruction { name: "???", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "BEQ", clock_cycles: 2, addr_mode: AddressingMode::REL, function: BEQ},
    Instruction { name: "SBC", clock_cycles: 5, addr_mode: AddressingMode::IZY, function: SBC},
    Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "???", clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP},
    Instruction { name: "SBC", clock_cycles: 4, addr_mode: AddressingMode::ZPX, function: SBC},
    Instruction { name: "INC", clock_cycles: 6, addr_mode: AddressingMode::ZPX, function: INC},
    Instruction { name: "???", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "SED", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: SED},
    Instruction { name: "SBC", clock_cycles: 4, addr_mode: AddressingMode::ABY, function: SBC},
    Instruction { name: "NOP", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP},
    Instruction { name: "???", clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX},
    Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP},
    Instruction { name: "SBC", clock_cycles: 4, addr_mode: AddressingMode::ABX, function: SBC},
    Instruction { name: "INC", clock_cycles: 7, addr_mode: AddressingMode::ABX, function: INC},
    Instruction { name: "???", clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX},
];

