use crate::cpu::Cpu6502;
use crate::instructions::instruction_functions::XXX;

#[derive(Clone, Copy)]
pub struct Instruction {
    pub(crate) name: &'static str,
    pub(crate) addr_mode: AddressingMode,
    pub(crate) function: for<'r> fn(&'r mut Cpu6502) -> u8,
    pub(crate) clock_cycles: u8,
}

impl Default for Instruction {
    fn default() -> Self {
        Instruction {
            name: "???",
            addr_mode: AddressingMode::XXX,
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
    XXX,
}

// Core: Modify the cpu's registers according to the provided instructions addressing mode
// returns 0 or 1 depending if an extra cycle is required
pub fn process_instruction_addressing_mode(instruction: &Instruction, cpu: &mut Cpu6502) -> u8 {
    match instruction.addr_mode {
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
            cpu.addr_abs = cpu.pc; //  + 1; MAYBE A BUG
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
            cpu.addr_abs = (cpu.read_bus(cpu.pc) + cpu.x_reg) as u16;
            cpu.pc += 1;
            cpu.addr_abs &= 0x00FF;
            0
        }

        // Address Mode: Zero Page with Y Offset
        // Same as above but uses Y Register for offset
        AddressingMode::ZPY => {
            cpu.addr_abs = (cpu.read_bus(cpu.pc) + cpu.y_reg) as u16;
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
            cpu.pc += 1;
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
            cpu.addr_abs += cpu.x_reg as u16;
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
            cpu.addr_abs += cpu.y_reg as u16;
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
            cpu.addr_abs = cpu.read_bus_two_bytes(t + cpu.x_reg as u16);
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
            cpu.temp = cpu.read_bus_two_bytes(t);
            cpu.addr_abs += cpu.y_reg as u16;

            if cpu.addr_abs & 0xFF00 != cpu.temp & 0xFF00 {
                return 1;
            } else {
                return 0;
            }
        }
        AddressingMode::XXX => 0,
    }
}

// Helper: Check if the page has changed, return an additional clock cycle if yes
fn check_page_change(previous: &u16, current: &u16) -> u8 {
    if current & 0xFF00 != previous & 0xFF00 {
        return 1;
    }
    0
}

