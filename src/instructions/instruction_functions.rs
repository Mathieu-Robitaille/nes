use crate::cpu::{Cpu6502, Flags};
use crate::instructions::instruction::AddressingMode;

#[allow(non_snake_case)]
pub fn ADC(cpu: &mut Cpu6502) -> u8 {
    // Grab the data we're adding to the acc
    cpu.fetch();

    // Add is performed in 16-bit domain for emulation to capture any
    // carry bit, which will exist in bit 8 of the 16-bit word
    cpu.temp = cpu.acc as u16 + cpu.fetched as u16 + cpu.get_flag(Flags::C) as u16;
    cpu.set_flag(
        Flags::V,
        (!(cpu.acc ^ cpu.fetched) as u16 & (cpu.acc as u16 ^ cpu.temp)) & 0x0080 > 0,
    );
    set_carry(cpu, cpu.temp);
    set_nz_flags(cpu, cpu.temp as u8);
    
    cpu.acc = cpu.temp as u8;
    1
}

// Instruction: Bitwise Logic AND
// Function:    A = A & M
// Flags Out:   N, Z
#[allow(non_snake_case)]
pub fn AND(cpu: &mut Cpu6502) -> u8 {
    cpu.fetch();
    cpu.acc = cpu.acc & cpu.fetched;
    set_nz_flags(cpu, cpu.acc);
    1
}

// Instruction: Arithmetic Shift Left
// Function:    A = C <- (A << 1) <- 0
// Flags Out:   N, Z, C
#[allow(non_snake_case)]
pub fn ASL(cpu: &mut Cpu6502) -> u8 {
    cpu.fetch();
    cpu.temp = (cpu.fetched as u16) << 1;
    set_carry(cpu, cpu.temp);
    set_nz_flags(cpu, cpu.temp as u8);
    if cpu.addressing_mode == AddressingMode::IMP {
        cpu.acc = (cpu.temp & 0x00FF) as u8;
    } else {
        cpu.write_bus(cpu.addr_abs, cpu.temp as u8)
    }
    0
}

// Instruction: Branch if Carry Clear
// Function:    if(C == 0) pc = address
#[allow(non_snake_case)]
pub fn BCC(cpu: &mut Cpu6502) -> u8 {
    if cpu.get_flag(Flags::C) == 0 {
        cpu.cycles += 1;
        cpu.addr_abs = branch_add(cpu.pc, cpu.addr_rel);

        if ((cpu.addr_abs & 0xFF00) != (cpu.pc & 0xFF00)) {
            cpu.cycles += 1;
        }
        cpu.pc = cpu.addr_abs;
    }
    0
}

// Instruction: Branch if Carry Set
// Function:    if(C == 1) pc = address
#[allow(non_snake_case)]
pub fn BCS(cpu: &mut Cpu6502) -> u8 {
    if cpu.get_flag(Flags::C) == 1 {
        cpu.cycles += 1;
        cpu.addr_abs = branch_add(cpu.pc, cpu.addr_rel);

        if ((cpu.addr_abs & 0xFF00) != (cpu.pc & 0xFF00)) {
            cpu.cycles += 1;
        }
        cpu.pc = cpu.addr_abs;
    }
    0
}

// Instruction: Branch if Equal
// Function:    if(Z == 1) pc = address
#[allow(non_snake_case)]
pub fn BEQ(cpu: &mut Cpu6502) -> u8 {
    if cpu.get_flag(Flags::Z) == 1 {
        cpu.cycles += 1;
        cpu.addr_abs = branch_add(cpu.pc, cpu.addr_rel);

        if ((cpu.addr_abs & 0xFF00) != (cpu.pc & 0xFF00)) {
            cpu.cycles += 1;
        }
        cpu.pc = cpu.addr_abs;
    }
    0
}

#[allow(non_snake_case)]
pub fn BIT(cpu: &mut Cpu6502) -> u8 {
    cpu.fetch();
    cpu.temp = (cpu.acc & cpu.fetched) as u16;
    cpu.set_flag(Flags::Z, cpu.temp as u8 == 0x00);
    cpu.set_flag(Flags::N, (cpu.fetched & Flags::N) > 0);
    cpu.set_flag(Flags::V, (cpu.fetched & Flags::V) > 0);
    0
}

// Instruction: Branch if Negative
// Function:    if(N == 1) pc = address
#[allow(non_snake_case)]
pub fn BMI(cpu: &mut Cpu6502) -> u8 {
    if cpu.get_flag(Flags::N) == 1 {
        cpu.cycles += 1;
        cpu.addr_abs = branch_add(cpu.pc, cpu.addr_rel);
        if ((cpu.addr_abs & 0xFF00) != (cpu.pc & 0xFF00)) {
            cpu.cycles += 1;
        }
        cpu.pc = cpu.addr_abs;
    }
    0
}

// Instruction: Branch if Not Equal
// Function:    if(Z == 0) pc = address
#[allow(non_snake_case)]
pub fn BNE(cpu: &mut Cpu6502) -> u8 {
    if cpu.get_flag(Flags::Z) == 0 {
        cpu.cycles += 1;

        
        // println!("PROGRAM_COUNTER: {:04X?} | addr_rel: {:04X?}", cpu.program_counter, cpu.addr_rel);
        // println!("PROGRAM_COUNTER: {:b} | addr_rel: {:b}", cpu.program_counter, cpu.addr_rel);
        // Gross hack
        // let (a, _) = cpu.program_counter.overflowing_add(cpu.addr_rel);

        // println!("x: {:04X?}", a);
        cpu.addr_abs = branch_add(cpu.pc, cpu.addr_rel);
        if ((cpu.addr_abs & 0xFF00) != (cpu.pc & 0xFF00)) {
            cpu.cycles += 1;
        }
        cpu.pc = cpu.addr_abs;
    }
    0
}

// Instruction: Branch if Positive
// Function:    if(N == 0) pc = address
#[allow(non_snake_case)]
pub fn BPL(cpu: &mut Cpu6502) -> u8 {
    if cpu.get_flag(Flags::N) == 0 {
        cpu.cycles += 1;
        cpu.addr_abs = branch_add(cpu.pc, cpu.addr_rel);
        if ((cpu.addr_abs & 0xFF00) != (cpu.pc & 0xFF00)) {
            cpu.cycles += 1;
        }
        cpu.pc = cpu.addr_abs;
    }
    0
}

// Instruction: Break
// Function:    Program Sourced Interrupt
#[allow(non_snake_case)]
pub fn BRK(cpu: &mut Cpu6502) -> u8 {
    cpu.pc += 1;
    cpu.set_flag(Flags::I, true);
    cpu.write_bus_two_bytes(0x0100 + cpu.stack_pointer as u16, cpu.pc);
    cpu.stack_pointer -= 2;

    cpu.set_flag(Flags::B, true);
    cpu.write_bus(0x0100 + cpu.stack_pointer as u16, cpu.status);
    cpu.stack_pointer -= 1;
    cpu.set_flag(Flags::B, false);

    cpu.pc = cpu.read_bus_two_bytes(0xFFFE);
    0
}

// Instruction: Branch if Overflow Clear
// Function:    if(V == 0) pc = address
#[allow(non_snake_case)]
pub fn BVC(cpu: &mut Cpu6502) -> u8 {
    if cpu.get_flag(Flags::V) == 0 {
        cpu.cycles += 1;
        cpu.addr_abs = branch_add(cpu.pc, cpu.addr_rel);
        if ((cpu.addr_abs & 0xFF00) != (cpu.pc & 0xFF00)) {
            cpu.cycles += 1;
        }
        cpu.pc = cpu.addr_abs;
    }
    0
}

// Instruction: Branch if Overflow Set
// Function:    if(V == 1) pc = address
#[allow(non_snake_case)]
pub fn BVS(cpu: &mut Cpu6502) -> u8 {
    if cpu.get_flag(Flags::V) == 1 {
        cpu.cycles += 1;
        cpu.addr_abs = branch_add(cpu.pc, cpu.addr_rel);
        if ((cpu.addr_abs & 0xFF00) != (cpu.pc & 0xFF00)) {
            cpu.cycles += 1;
        }
        cpu.pc = cpu.addr_abs;
    }
    0
}

// Instruction: Clear Carry Flag
// Function:    C = 0
#[allow(non_snake_case)]
pub fn CLC(cpu: &mut Cpu6502) -> u8 {
    cpu.set_flag(Flags::C, false);
    0
}

// Instruction: Clear Decimal Flag
// Function:    D = 0
#[allow(non_snake_case)]
pub fn CLD(cpu: &mut Cpu6502) -> u8 {
    cpu.set_flag(Flags::D, false);
    0
}

// Instruction: Disable Interrupts / Clear Interrupt Flag
// Function:    I = 0
#[allow(non_snake_case)]
pub fn CLI(cpu: &mut Cpu6502) -> u8 {
    cpu.set_flag(Flags::I, false);
    0
}

// Instruction: Clear Overflow Flag
// Function:    V = 0
#[allow(non_snake_case)]
pub fn CLV(cpu: &mut Cpu6502) -> u8 {
    cpu.set_flag(Flags::V, false);
    0
}

// Instruction: Compare Accumulator
// Function:    C <- A >= M      Z <- (A - M) == 0
// Flags Out:   N, C, Z
#[allow(non_snake_case)]
pub fn CMP(cpu: &mut Cpu6502) -> u8 {
    cpu.fetch();
    cpu.temp = (cpu.acc - cpu.fetched) as u16;
    cpu.set_flag(Flags::C, cpu.acc >= cpu.fetched);
    set_nz_flags(cpu, cpu.temp as u8);
    0
}

// Instruction: Compare X Register
// Function:    C <- X >= M      Z <- (X - M) == 0
// Flags Out:   N, C, Z
#[allow(non_snake_case)]
pub fn CPX(cpu: &mut Cpu6502) -> u8 {
    cpu.fetch();
    cpu.temp = (cpu.x_reg - cpu.fetched) as u16;
    cpu.set_flag(Flags::C, cpu.x_reg >= cpu.fetched);
    set_nz_flags(cpu, cpu.temp as u8);
    0
}

// Instruction: Compare Y Register
// Function:    C <- Y >= M      Z <- (Y - M) == 0
// Flags Out:   N, C, Z
#[allow(non_snake_case)]
pub fn CPY(cpu: &mut Cpu6502) -> u8 {
    cpu.fetch();
    cpu.temp = (cpu.y_reg - cpu.fetched) as u16;
    cpu.set_flag(Flags::C, cpu.y_reg >= cpu.fetched);
    set_nz_flags(cpu, cpu.temp as u8);
    0
}

// Instruction: Decrement Value at Memory Location
// Function:    M = M - 1
// Flags Out:   N, Z
#[allow(non_snake_case)]
pub fn DEC(cpu: &mut Cpu6502) -> u8 {
    cpu.fetch();
    cpu.temp = (cpu.fetched - 1) as u16;
    cpu.write_bus(cpu.addr_abs, cpu.temp as u8);
    set_nz_flags(cpu, cpu.temp as u8);
    0
}

// Instruction: Decrement X Register
// Function:    X = X - 1
// Flags Out:   N, Z
#[allow(non_snake_case)]
pub fn DEX(cpu: &mut Cpu6502) -> u8 {
    cpu.x_reg -= 1;
    set_nz_flags(cpu, cpu.x_reg);
    0
}

// Instruction: Decrement Y Register
// Function:    Y = Y - 1
// Flags Out:   N, Z
#[allow(non_snake_case)]
pub fn DEY(cpu: &mut Cpu6502) -> u8 {
    cpu.y_reg -= 1;
    set_nz_flags(cpu, cpu.y_reg);
    0
}

// Instruction: Bitwise Logic XOR
// Function:    A = A xor M
// Flags Out:   N, Z
#[allow(non_snake_case)]
pub fn EOR(cpu: &mut Cpu6502) -> u8 {
    cpu.fetch();
    cpu.acc = cpu.acc ^ cpu.fetched;
    set_nz_flags(cpu, cpu.acc);
    1
}

// Instruction: Increment Value at Memory Location
// Function:    M = M + 1
// Flags Out:   N, Z
#[allow(non_snake_case)]
pub fn INC(cpu: &mut Cpu6502) -> u8 {
    cpu.fetch();
    cpu.temp = (cpu.fetched + 1) as u16;
    cpu.write_bus(cpu.addr_abs, (cpu.temp & 0x00FF) as u8);
    set_nz_flags(cpu, cpu.temp as u8);
    0
}

// Instruction: Increment X Register
// Function:    X = X + 1
// Flags Out:   N, Z
#[allow(non_snake_case)]
pub fn INX(cpu: &mut Cpu6502) -> u8 {
    cpu.x_reg += 1;
    set_nz_flags(cpu, cpu.x_reg);
    0
}

// Instruction: Increment Y Register
// Function:    Y = Y + 1
// Flags Out:   N, Z
#[allow(non_snake_case)]
pub fn INY(cpu: &mut Cpu6502) -> u8 {
    cpu.x_reg += 1;
    set_nz_flags(cpu, cpu.y_reg);
    0
}

#[allow(non_snake_case)]
pub fn JMP(cpu: &mut Cpu6502) -> u8 {
    cpu.pc = cpu.addr_abs;
    0
}

#[allow(non_snake_case)]
pub fn JSR(cpu: &mut Cpu6502) -> u8 {
    cpu.pc -= 1;
    cpu.write_bus_two_bytes(0x0100 + cpu.stack_pointer as u16, cpu.pc);
    cpu.pc = cpu.addr_abs;
    0
}

#[allow(non_snake_case)]
pub fn LDA(cpu: &mut Cpu6502) -> u8 {
    cpu.fetch();
    cpu.acc = cpu.fetched;
    set_nz_flags(cpu, cpu.acc);
    1
}

#[allow(non_snake_case)]
pub fn LDX(cpu: &mut Cpu6502) -> u8 {
    cpu.fetch();
    cpu.x_reg = cpu.fetched;
    set_nz_flags(cpu, cpu.x_reg);
    1
}

#[allow(non_snake_case)]
pub fn LDY(cpu: &mut Cpu6502) -> u8 {
    cpu.fetch();
    cpu.y_reg = cpu.fetched;
    set_nz_flags(cpu, cpu.y_reg);
    1
}

#[allow(non_snake_case)]
pub fn LSR(cpu: &mut Cpu6502) -> u8 {
    cpu.fetch();
    cpu.set_flag(Flags::C, cpu.fetched & 1 > 0);
    cpu.temp = (cpu.fetched >> 1) as u16;

    set_nz_flags(cpu, cpu.temp as u8);
    if cpu.addressing_mode == AddressingMode::IMP {
        cpu.acc = (cpu.temp & 0x00FF) as u8;
    } else {
        cpu.write_bus(cpu.addr_abs, cpu.temp as u8)
    }
    0
}

#[allow(non_snake_case)]
pub fn NOP(cpu: &mut Cpu6502) -> u8 {
    match cpu.opcode {
        0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => {
            return 1;
        }
        _ => {
            return 0;
        }
    }
}

// Instruction: Bitwise Logic OR
// Function:    A = A | M
// Flags Out:   N, Z
#[allow(non_snake_case)]
pub fn ORA(cpu: &mut Cpu6502) -> u8 {
    cpu.fetch();
    cpu.acc = cpu.acc | cpu.fetched;
    set_nz_flags(cpu, cpu.acc);
    1
}

// Instruction: Push Accumulator to Stack
// Function:    A -> stack
#[allow(non_snake_case)]
pub fn PHA(cpu: &mut Cpu6502) -> u8 {
    cpu.write_bus(0x0100 + cpu.stack_pointer as u16, cpu.acc);
    cpu.stack_pointer -= 1;
    0
}

// Instruction: Push Status Register to Stack
// Function:    status -> stack
// Note:        Break flag is set to 1 before push
#[allow(non_snake_case)]
pub fn PHP(cpu: &mut Cpu6502) -> u8 {
    cpu.write_bus(
        0x0100 + cpu.stack_pointer as u16,
        cpu.status | Flags::B | Flags::U,
    );
    cpu.set_flag(Flags::B, false);
    cpu.set_flag(Flags::U, false);
    cpu.stack_pointer -= 1;
    0
}

// Instruction: Pop Accumulator off Stack
// Function:    A <- stack
// Flags Out:   N, Z
#[allow(non_snake_case)]
pub fn PLA(cpu: &mut Cpu6502) -> u8 {
    cpu.stack_pointer += 1;
    cpu.acc = cpu.read_bus(0x0100 + cpu.stack_pointer as u16);
    set_nz_flags(cpu, cpu.acc);
    0
}

// Instruction: Pop Status Register off Stack
// Function:    Status <- stack
#[allow(non_snake_case)]
pub fn PLP(cpu: &mut Cpu6502) -> u8 {
    cpu.stack_pointer += 1;
    cpu.status = cpu.read_bus(0x0100 + cpu.stack_pointer as u16);
    cpu.set_flag(Flags::U, true); // not needed??
    0
}

#[allow(non_snake_case)]
pub fn ROL(cpu: &mut Cpu6502) -> u8 {
    cpu.fetch();
    cpu.temp = ((cpu.fetched << 1) | cpu.get_flag(Flags::C)) as u16;
    let x: u8 = (cpu.temp & 0x00FF) as u8;
    
    set_carry(cpu, cpu.temp);
    set_nz_flags(cpu, cpu.temp as u8);
    if cpu.addressing_mode == AddressingMode::IMP {
        cpu.acc = x
    } else {
        cpu.write_bus(cpu.addr_abs, x)
    }
    0
}

#[allow(non_snake_case)]
pub fn ROR(cpu: &mut Cpu6502) -> u8 {
    cpu.fetch();
    cpu.temp = ((cpu.fetched >> 1) | cpu.get_flag(Flags::C) << 7) as u16;
    let x: u8 = (cpu.temp & 0x00FF) as u8;

    // needs a special case
    cpu.set_flag(Flags::C, cpu.fetched & 0x01 > 0);

    set_nz_flags(cpu, cpu.temp as u8);
    if cpu.addressing_mode == AddressingMode::IMP {
        cpu.acc = x
    } else {
        cpu.write_bus(cpu.addr_abs, x)
    }
    0
}

#[allow(non_snake_case)]
pub fn RTI(cpu: &mut Cpu6502) -> u8 {
    cpu.stack_pointer += 1;
    cpu.status = cpu.read_bus(0x0100 + (cpu.stack_pointer as u16));
    cpu.status &= !Flags::B;
    cpu.status &= !Flags::U;
    cpu.stack_pointer += 1;
    cpu.pc = cpu.read_bus_two_bytes(0x0100 + (cpu.stack_pointer as u16));
    cpu.stack_pointer += 1;
    0
}

#[allow(non_snake_case)]
pub fn RTS(cpu: &mut Cpu6502) -> u8 {
    cpu.stack_pointer += 1;
    cpu.pc = cpu.read_bus_two_bytes(0x0100 + (cpu.stack_pointer as u16));
    cpu.stack_pointer += 1;
    cpu.pc += 1;
    0
}

// Instruction: Subtraction with Borrow In
// Function:    A = A - M - (1 - C)
// Flags Out:   C, V, N, Z
#[allow(non_snake_case)]
pub fn SBC(cpu: &mut Cpu6502) -> u8 {
    cpu.fetch();
    let v: u16 = (cpu.fetched as u16) ^ 0x00FF;
    cpu.temp = (cpu.acc as u16) + v + (cpu.get_flag(Flags::C) as u16);

    set_carry(cpu, cpu.temp);
    set_nz_flags(cpu, cpu.temp as u8);

    cpu.set_flag(
        Flags::V,
        ((cpu.temp ^ (cpu.acc as u16)) & (cpu.temp ^ v) & 0x0080) > 0,
    );
    cpu.acc = (cpu.temp & 0x00FF) as u8;
    0
}

// Instruction: Set Carry Flag
// Function:    C = 1
#[allow(non_snake_case)]
pub fn SEC(cpu: &mut Cpu6502) -> u8 {
    cpu.set_flag(Flags::C, true);
    0
}

// Instruction: Set Decimal Flag
// Function:    D = 1
#[allow(non_snake_case)]
pub fn SED(cpu: &mut Cpu6502) -> u8 {
    cpu.set_flag(Flags::D, true);
    0
}

// Instruction: Set Interrupt Flag / Enable Interrupts
// Function:    I = 1
#[allow(non_snake_case)]
pub fn SEI(cpu: &mut Cpu6502) -> u8 {
    cpu.set_flag(Flags::I, true);
    0
}

// Instruction: Store Accumulator at Address
// Function:    M = A
#[allow(non_snake_case)]
pub fn STA(cpu: &mut Cpu6502) -> u8 {
    cpu.write_bus(cpu.addr_abs, cpu.acc);
    0
}

// Instruction: Store X Register at Address
// Function:    M = X
#[allow(non_snake_case)]
pub fn STX(cpu: &mut Cpu6502) -> u8 {
    cpu.write_bus(cpu.addr_abs, cpu.x_reg);
    0
}

// Instruction: Store Y Register at Address
// Function:    M = Y
#[allow(non_snake_case)]
pub fn STY(cpu: &mut Cpu6502) -> u8 {
    cpu.write_bus(cpu.addr_abs, cpu.y_reg);
    0
}

// Instruction: Transfer Accumulator to X Register
// Function:    X = A
// Flags Out:   N, Z
#[allow(non_snake_case)]
pub fn TAX(cpu: &mut Cpu6502) -> u8 {
    cpu.x_reg = cpu.acc;
    set_nz_flags(cpu, cpu.x_reg);
    0
}

// Instruction: Transfer Accumulator to Y Register
// Function:    Y = A
// Flags Out:   N, Z
#[allow(non_snake_case)]
pub fn TAY(cpu: &mut Cpu6502) -> u8 {
    cpu.y_reg = cpu.acc;
    set_nz_flags(cpu, cpu.y_reg);
    0
}

// Instruction: Transfer Stack Pointer to X Register
// Function:    X = stack pointer
// Flags Out:   N, Z
#[allow(non_snake_case)]
pub fn TSX(cpu: &mut Cpu6502) -> u8 {
    cpu.x_reg = cpu.stack_pointer;
    set_nz_flags(cpu, cpu.x_reg);
    0
}

// Instruction: Transfer X Register to Accumulator
// Function:    A = X
// Flags Out:   N, Z
#[allow(non_snake_case)]
pub fn TXA(cpu: &mut Cpu6502) -> u8 {
    cpu.acc = cpu.x_reg;
    set_nz_flags(cpu, cpu.acc);
    0
}

// Instruction: Transfer X Register to Stack Pointer
// Function:    stack pointer = X
#[allow(non_snake_case)]
pub fn TXS(cpu: &mut Cpu6502) -> u8 {
    cpu.stack_pointer = cpu.x_reg;
    0
}

// Instruction: Transfer Y Register to Accumulator
// Function:    A = Y
// Flags Out:   N, Z
#[allow(non_snake_case)]
pub fn TYA(cpu: &mut Cpu6502) -> u8 {
    cpu.acc = cpu.y_reg;
    set_nz_flags(cpu, cpu.acc);
    0
}

#[allow(non_snake_case)]
pub fn XXX(cpu: &mut Cpu6502) -> u8 {
    0
}

fn set_carry(cpu: &mut Cpu6502, reg: u16) {
    cpu.set_flag(Flags::C, reg & 0xFF00 > 0);
}

fn set_z_if_reg_zero(cpu: &mut Cpu6502, reg: u8) {
    cpu.set_flag(Flags::Z, reg == 0);
}

fn set_n_if_bit_set(cpu: &mut Cpu6502, reg: u8) {
    cpu.set_flag(Flags::N, reg & 0x80 > 0);
}

fn set_nz_flags(cpu: &mut Cpu6502, reg: u8) {
    set_z_if_reg_zero(cpu, reg);
    set_n_if_bit_set(cpu, reg);
}

fn branch_add(a: u16, b: u16) -> u16 {
    let (r, _) = a.overflowing_add(b);
    r
}