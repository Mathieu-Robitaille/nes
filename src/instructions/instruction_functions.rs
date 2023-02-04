use crate::cpu::{CPUFlags, Cpu6502};
use crate::instructions::instruction::AddressingMode;

#[allow(non_snake_case)]
pub fn ADC(cpu: &mut Cpu6502) {
    // Grab the data we're adding to the acc
    cpu.fetch();

    // Add is performed in 16-bit domain for emulation to capture any
    // carry bit, which will exist in bit 8 of the 16-bit word
    cpu.temp = cpu.acc as u16 + cpu.fetched as u16 + cpu.get_flag(CPUFlags::C) as u16;
    cpu.set_flag(
        CPUFlags::V,
        (!(cpu.acc ^ cpu.fetched) as u16 & (cpu.acc as u16 ^ cpu.temp)) & 0x0080 > 0,
    );
    set_carry(cpu, cpu.temp);
    set_nz_flags(cpu, cpu.temp as u8);

    cpu.acc = cpu.temp as u8;
}

// Instruction: Bitwise Logic AND
// Function:    A = A & M
// Flags Out:   N, Z
#[allow(non_snake_case)]
pub fn AND(cpu: &mut Cpu6502) {
    cpu.fetch();
    cpu.acc = cpu.acc & cpu.fetched;
    set_nz_flags(cpu, cpu.acc);
}

// Instruction: Arithmetic Shift Left
// Function:    A = C <- (A << 1) <- 0
// Flags Out:   N, Z, C
#[allow(non_snake_case)]
pub fn ASL(cpu: &mut Cpu6502) {
    cpu.fetch();
    cpu.temp = (cpu.fetched as u16) << 1;
    set_carry(cpu, cpu.temp);
    set_nz_flags(cpu, cpu.temp as u8);
    if cpu.addressing_mode == AddressingMode::IMP {
        cpu.acc = (cpu.temp & 0x00FF) as u8;
    } else {
        cpu.write_bus(cpu.addr_abs, cpu.temp as u8);
    }
}

// Instruction: Branch if Carry Clear
// Function:    if(C == 0) pc = address
#[allow(non_snake_case)]
pub fn BCC(cpu: &mut Cpu6502) {
    if cpu.get_flag(CPUFlags::C) == 0 {
        cpu.cycles += 1;
        cpu.addr_abs = cpu.pc.wrapping_add(cpu.addr_rel);

        branch_check_page_change(cpu);
        cpu.pc = cpu.addr_abs;
    }
}

// Instruction: Branch if Carry Set
// Function:    if(C == 1) pc = address
#[allow(non_snake_case)]
pub fn BCS(cpu: &mut Cpu6502) {
    if cpu.get_flag(CPUFlags::C) == 1 {
        cpu.cycles += 1;
        cpu.addr_abs = cpu.pc.wrapping_add(cpu.addr_rel);

        branch_check_page_change(cpu);
        cpu.pc = cpu.addr_abs;
    }
}

// Instruction: Branch if Equal
// Function:    if(Z == 1) pc = address
#[allow(non_snake_case)]
pub fn BEQ(cpu: &mut Cpu6502) {
    if cpu.get_flag(CPUFlags::Z) == 1 {
        cpu.cycles += 1;
        cpu.addr_abs = cpu.pc.wrapping_add(cpu.addr_rel);

        branch_check_page_change(cpu);
        cpu.pc = cpu.addr_abs;
    }
}

#[allow(non_snake_case)]
pub fn BIT(cpu: &mut Cpu6502) {
    cpu.fetch();
    cpu.temp = (cpu.acc & cpu.fetched) as u16;
    cpu.set_flag(CPUFlags::Z, cpu.temp as u8 == 0x00);
    cpu.set_flag(CPUFlags::N, (cpu.fetched & CPUFlags::N) > 0);
    cpu.set_flag(CPUFlags::V, (cpu.fetched & CPUFlags::V) > 0);
}

// Instruction: Branch if Negative
// Function:    if(N == 1) pc = address
#[allow(non_snake_case)]
pub fn BMI(cpu: &mut Cpu6502) {
    if cpu.get_flag(CPUFlags::N) == 1 {
        cpu.cycles += 1;
        cpu.addr_abs = cpu.pc.wrapping_add(cpu.addr_rel);
        branch_check_page_change(cpu);
        cpu.pc = cpu.addr_abs;
    }
}

// Instruction: Branch if Not Equal
// Function:    if(Z == 0) pc = address
#[allow(non_snake_case)]
pub fn BNE(cpu: &mut Cpu6502) {
    if cpu.get_flag(CPUFlags::Z) == 0 {
        cpu.cycles += 1;

        cpu.addr_abs = cpu.pc.wrapping_add(cpu.addr_rel);
        branch_check_page_change(cpu);
        cpu.pc = cpu.addr_abs;
    }
}

// Instruction: Branch if Positive
// Function:    if(N == 0) pc = address
#[allow(non_snake_case)]
pub fn BPL(cpu: &mut Cpu6502) {
    if cpu.get_flag(CPUFlags::N) == 0 {
        cpu.cycles += 1;
        cpu.addr_abs = cpu.pc.wrapping_add(cpu.addr_rel);
        branch_check_page_change(cpu);
        cpu.pc = cpu.addr_abs;
    }
}

// Instruction: Break
// Function:    Program Sourced Interrupt
#[allow(non_snake_case)]
pub fn BRK(cpu: &mut Cpu6502) {
    cpu.pc += 1;
    cpu.set_flag(CPUFlags::I, true);
    cpu.write_bus_two_bytes(0x0100 + cpu.stack_pointer as u16, cpu.pc);
    cpu.stack_pointer = cpu.stack_pointer.wrapping_sub(2);

    cpu.set_flag(CPUFlags::B, true);
    cpu.write_bus(0x0100 + cpu.stack_pointer as u16, cpu.status);
    cpu.stack_pointer = cpu.stack_pointer.wrapping_sub(1);
    cpu.set_flag(CPUFlags::B, false);

    cpu.pc = cpu.read_bus_two_bytes(0xFFFE);
}

// Instruction: Branch if Overflow Clear
// Function:    if(V == 0) pc = address
#[allow(non_snake_case)]
pub fn BVC(cpu: &mut Cpu6502) {
    if cpu.get_flag(CPUFlags::V) == 0 {
        cpu.cycles += 1;
        cpu.addr_abs = cpu.pc.wrapping_add(cpu.addr_rel);
        branch_check_page_change(cpu);
        cpu.pc = cpu.addr_abs;
    }
}

// Instruction: Branch if Overflow Set
// Function:    if(V == 1) pc = address
#[allow(non_snake_case)]
pub fn BVS(cpu: &mut Cpu6502) {
    if cpu.get_flag(CPUFlags::V) == 1 {
        cpu.cycles += 1;
        cpu.addr_abs = cpu.pc.wrapping_add(cpu.addr_rel);
        branch_check_page_change(cpu);
        cpu.pc = cpu.addr_abs;
    }
}

// Instruction: Clear Carry Flag
// Function:    C = 0
#[allow(non_snake_case)]
pub fn CLC(cpu: &mut Cpu6502) {
    cpu.set_flag(CPUFlags::C, false);
}

// Instruction: Clear Decimal Flag
// Function:    D = 0
#[allow(non_snake_case)]
pub fn CLD(cpu: &mut Cpu6502) {
    cpu.set_flag(CPUFlags::D, false);
}

// Instruction: Disable Interrupts / Clear Interrupt Flag
// Function:    I = 0
#[allow(non_snake_case)]
pub fn CLI(cpu: &mut Cpu6502) {
    cpu.set_flag(CPUFlags::I, false);
}

// Instruction: Clear Overflow Flag
// Function:    V = 0
#[allow(non_snake_case)]
pub fn CLV(cpu: &mut Cpu6502) {
    cpu.set_flag(CPUFlags::V, false);
}

// Instruction: Compare Accumulator
// Function:    C <- A >= M      Z <- (A - M) == 0
// Flags Out:   N, C, Z
#[allow(non_snake_case)]
pub fn CMP(cpu: &mut Cpu6502) {
    cpu.fetch();
    let x = cpu.acc.wrapping_sub(cpu.fetched);
    cpu.temp = x as u16;
    cpu.set_flag(CPUFlags::C, cpu.acc >= cpu.fetched);
    set_nz_flags(cpu, x);
}

// Instruction: Compare X Register
// Function:    C <- X >= M      Z <- (X - M) == 0
// Flags Out:   N, C, Z
#[allow(non_snake_case)]
pub fn CPX(cpu: &mut Cpu6502) {
    cpu.fetch();
    let x = cpu.x_reg.wrapping_sub(cpu.fetched);
    cpu.temp = x as u16;
    cpu.set_flag(CPUFlags::C, cpu.x_reg >= cpu.fetched);
    set_nz_flags(cpu, x);
}

// Instruction: Compare Y Register
// Function:    C <- Y >= M      Z <- (Y - M) == 0
// Flags Out:   N, C, Z
#[allow(non_snake_case)]
pub fn CPY(cpu: &mut Cpu6502) {
    cpu.fetch();
    let x = cpu.y_reg.wrapping_sub(cpu.fetched);
    cpu.temp = x as u16;
    cpu.set_flag(CPUFlags::C, cpu.y_reg >= cpu.fetched);
    set_nz_flags(cpu, x);
}

// Instruction: Decrement Value at Memory Location
// Function:    M = M - 1
// Flags Out:   N, Z
#[allow(non_snake_case)]
pub fn DEC(cpu: &mut Cpu6502) {
    cpu.fetch();
    let x = cpu.fetched.wrapping_sub(1);
    cpu.temp = x as u16;
    cpu.write_bus(cpu.addr_abs, x);
    set_nz_flags(cpu, x);
}

// Instruction: Decrement X Register
// Function:    X = X - 1
// Flags Out:   N, Z
#[allow(non_snake_case)]
pub fn DEX(cpu: &mut Cpu6502) {
    cpu.x_reg = cpu.x_reg.wrapping_sub(1);
    set_nz_flags(cpu, cpu.x_reg);
}

// Instruction: Decrement Y Register
// Function:    Y = Y - 1
// Flags Out:   N, Z
#[allow(non_snake_case)]
pub fn DEY(cpu: &mut Cpu6502) {
    cpu.y_reg = cpu.y_reg.wrapping_sub(1);
    set_nz_flags(cpu, cpu.y_reg);
}

// Instruction: Bitwise Logic XOR
// Function:    A = A xor M
// Flags Out:   N, Z
#[allow(non_snake_case)]
pub fn EOR(cpu: &mut Cpu6502) {
    cpu.fetch();
    cpu.acc = cpu.acc ^ cpu.fetched;
    set_nz_flags(cpu, cpu.acc);
}

// Instruction: Increment Value at Memory Location
// Function:    M = M + 1
// Flags Out:   N, Z
#[allow(non_snake_case)]
pub fn INC(cpu: &mut Cpu6502) {
    cpu.fetch();
    let x = cpu.fetched.wrapping_add(1);
    cpu.temp = x as u16;
    cpu.write_bus(cpu.addr_abs, x);
    set_nz_flags(cpu, x);
}

// Instruction: Increment X Register
// Function:    X = X + 1
// Flags Out:   N, Z
#[allow(non_snake_case)]
pub fn INX(cpu: &mut Cpu6502) {
    let x = cpu.x_reg.wrapping_add(1);
    cpu.x_reg = x;
    set_nz_flags(cpu, x);
}

// Instruction: Increment Y Register
// Function:    Y = Y + 1
// Flags Out:   N, Z
#[allow(non_snake_case)]
pub fn INY(cpu: &mut Cpu6502) {
    let x = cpu.y_reg.wrapping_add(1);
    cpu.y_reg = x;
    set_nz_flags(cpu, x);
}

#[allow(non_snake_case)]
pub fn JMP(cpu: &mut Cpu6502) {
    cpu.pc = cpu.addr_abs;
}

#[allow(non_snake_case)]
pub fn JSR(cpu: &mut Cpu6502) {
    cpu.pc -= 1;
    cpu.write_bus_two_bytes(0x0100 + cpu.stack_pointer as u16, cpu.pc);
    cpu.stack_pointer = cpu.stack_pointer.wrapping_sub(2);
    cpu.pc = cpu.addr_abs;
}

#[allow(non_snake_case)]
pub fn LDA(cpu: &mut Cpu6502) {
    cpu.fetch();
    cpu.acc = cpu.fetched;
    set_nz_flags(cpu, cpu.acc);
}

#[allow(non_snake_case)]
pub fn LDX(cpu: &mut Cpu6502) {
    cpu.fetch();
    cpu.x_reg = cpu.fetched;
    set_nz_flags(cpu, cpu.x_reg);
}

#[allow(non_snake_case)]
pub fn LDY(cpu: &mut Cpu6502) {
    cpu.fetch();
    cpu.y_reg = cpu.fetched;
    set_nz_flags(cpu, cpu.y_reg);
}

#[allow(non_snake_case)]
pub fn LSR(cpu: &mut Cpu6502) {
    cpu.fetch();
    cpu.temp = (cpu.fetched >> 1) as u16;

    set_nz_flags(cpu, cpu.temp as u8);
    cpu.set_flag(CPUFlags::C, cpu.fetched & 0x01 > 0);

    if cpu.addressing_mode == AddressingMode::IMP {
        cpu.acc = cpu.temp as u8;
    } else {
        cpu.write_bus(cpu.addr_abs, cpu.temp as u8);
    }
}

#[allow(non_snake_case)]
pub fn NOP(cpu: &mut Cpu6502) {
    match cpu.opcode {
        0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => {
            cpu.cycles += 1;
        }
        _ => {}
    }
}

// Instruction: Bitwise Logic OR
// Function:    A = A | M
// Flags Out:   N, Z
#[allow(non_snake_case)]
pub fn ORA(cpu: &mut Cpu6502) {
    cpu.fetch();
    cpu.acc |= cpu.fetched;
    set_nz_flags(cpu, cpu.acc);
}

// Instruction: Push Accumulator to Stack
// Function:    A -> stack
#[allow(non_snake_case)]
pub fn PHA(cpu: &mut Cpu6502) {
    cpu.write_bus(0x0100 + cpu.stack_pointer as u16, cpu.acc);
    cpu.stack_pointer = cpu.stack_pointer.wrapping_sub(1);
}

// Instruction: Push Status Register to Stack
// Function:    status -> stack
// Note:        Break flag is set to 1 before push
#[allow(non_snake_case)]
pub fn PHP(cpu: &mut Cpu6502) {
    cpu.write_bus(
        0x0100 + cpu.stack_pointer as u16,
        cpu.status | CPUFlags::B | CPUFlags::U,
    );
    cpu.set_flag(CPUFlags::B, false);
    cpu.set_flag(CPUFlags::U, false);
    cpu.stack_pointer = cpu.stack_pointer.wrapping_sub(1);
}

// Instruction: Pop Accumulator off Stack
// Function:    A <- stack
// Flags Out:   N, Z
#[allow(non_snake_case)]
pub fn PLA(cpu: &mut Cpu6502) {
    cpu.stack_pointer = cpu.stack_pointer.wrapping_add(1);
    cpu.acc = cpu.read_bus(0x0100 + cpu.stack_pointer as u16);
    set_nz_flags(cpu, cpu.acc);
}

// Instruction: Pop Status Register off Stack
// Function:    Status <- stack
#[allow(non_snake_case)]
pub fn PLP(cpu: &mut Cpu6502) {
    cpu.stack_pointer = cpu.stack_pointer.wrapping_add(1);
    cpu.status = cpu.read_bus(0x0100 + cpu.stack_pointer as u16);
    cpu.set_flag(CPUFlags::U, true); // not needed??
}

#[allow(non_snake_case)]
pub fn ROL(cpu: &mut Cpu6502) {
    cpu.fetch();
    cpu.temp = (((cpu.fetched as u16) << 1) | (cpu.get_flag(CPUFlags::C)) as u16);
    let x: u8 = (cpu.temp & 0x00FF) as u8;

    set_carry(cpu, cpu.temp);
    set_nz_flags(cpu, cpu.temp as u8);
    if cpu.addressing_mode == AddressingMode::IMP {
        cpu.acc = x
    } else {
        cpu.write_bus(cpu.addr_abs, x)
    }
}

#[allow(non_snake_case)]
pub fn ROR(cpu: &mut Cpu6502) {
    cpu.fetch();
    cpu.temp = ((cpu.fetched >> 1) | cpu.get_flag(CPUFlags::C) << 7) as u16;
    let x: u8 = (cpu.temp & 0x00FF) as u8;

    // needs a special case
    cpu.set_flag(CPUFlags::C, cpu.fetched & 0x01 > 0);

    set_nz_flags(cpu, cpu.temp as u8);
    if cpu.addressing_mode == AddressingMode::IMP {
        cpu.acc = x
    } else {
        cpu.write_bus(cpu.addr_abs, x);
    }
}

#[allow(non_snake_case)]
pub fn RTI(cpu: &mut Cpu6502) {
    cpu.stack_pointer = cpu.stack_pointer.wrapping_add(1);
    cpu.status = cpu.read_bus(0x0100 + (cpu.stack_pointer as u16));
    cpu.status &= !CPUFlags::B;
    cpu.status &= !CPUFlags::U;
    cpu.stack_pointer = cpu.stack_pointer.wrapping_add(1);
    cpu.pc = cpu.read_bus_two_bytes(0x0100 + (cpu.stack_pointer as u16));
    cpu.stack_pointer = cpu.stack_pointer.wrapping_add(1);
}

#[allow(non_snake_case)]
pub fn RTS(cpu: &mut Cpu6502) {
    cpu.stack_pointer = cpu.stack_pointer.wrapping_add(1);
    cpu.pc = cpu.read_bus_two_bytes(0x0100 + (cpu.stack_pointer as u16));
    cpu.stack_pointer = cpu.stack_pointer.wrapping_add(1);
    cpu.pc += 1;
}

// Instruction: Subtraction with Borrow In
// Function:    A = A - M - (1 - C)
// Flags Out:   C, V, N, Z
#[allow(non_snake_case)]
pub fn SBC(cpu: &mut Cpu6502) {
    cpu.fetch();
    let v: u16 = (cpu.fetched as u16) ^ 0x00FF;
    cpu.temp = (cpu.acc as u16) + v + (cpu.get_flag(CPUFlags::C) as u16);

    set_carry(cpu, cpu.temp);
    set_nz_flags(cpu, cpu.temp as u8);

    cpu.set_flag(
        CPUFlags::V,
        ((cpu.temp ^ (cpu.acc as u16)) & (cpu.temp ^ v) & 0x0080) > 0,
    );
    cpu.acc = (cpu.temp & 0x00FF) as u8;
}

// Instruction: Set Carry Flag
// Function:    C = 1
#[allow(non_snake_case)]
pub fn SEC(cpu: &mut Cpu6502) {
    cpu.set_flag(CPUFlags::C, true);
}

// Instruction: Set Decimal Flag
// Function:    D = 1
#[allow(non_snake_case)]
pub fn SED(cpu: &mut Cpu6502) {
    cpu.set_flag(CPUFlags::D, true);
}

// Instruction: Set Interrupt Flag / Enable Interrupts
// Function:    I = 1
#[allow(non_snake_case)]
pub fn SEI(cpu: &mut Cpu6502) {
    cpu.set_flag(CPUFlags::I, true);
}

// Instruction: Store Accumulator at Address
// Function:    M = A
#[allow(non_snake_case)]
pub fn STA(cpu: &mut Cpu6502) {
    cpu.write_bus(cpu.addr_abs, cpu.acc);
}

// Instruction: Store X Register at Address
// Function:    M = X
#[allow(non_snake_case)]
pub fn STX(cpu: &mut Cpu6502) {
    cpu.write_bus(cpu.addr_abs, cpu.x_reg);
}

// Instruction: Store Y Register at Address
// Function:    M = Y
#[allow(non_snake_case)]
pub fn STY(cpu: &mut Cpu6502) {
    cpu.write_bus(cpu.addr_abs, cpu.y_reg);
}

// Instruction: Transfer Accumulator to X Register
// Function:    X = A
// Flags Out:   N, Z
#[allow(non_snake_case)]
pub fn TAX(cpu: &mut Cpu6502) {
    cpu.x_reg = cpu.acc;
    set_nz_flags(cpu, cpu.x_reg);
}

// Instruction: Transfer Accumulator to Y Register
// Function:    Y = A
// Flags Out:   N, Z
#[allow(non_snake_case)]
pub fn TAY(cpu: &mut Cpu6502) {
    cpu.y_reg = cpu.acc;
    set_nz_flags(cpu, cpu.y_reg);
}

// Instruction: Transfer Stack Pointer to X Register
// Function:    X = stack pointer
// Flags Out:   N, Z
#[allow(non_snake_case)]
pub fn TSX(cpu: &mut Cpu6502) {
    cpu.x_reg = cpu.stack_pointer;
    set_nz_flags(cpu, cpu.x_reg);
}

// Instruction: Transfer X Register to Accumulator
// Function:    A = X
// Flags Out:   N, Z
#[allow(non_snake_case)]
pub fn TXA(cpu: &mut Cpu6502) {
    cpu.acc = cpu.x_reg;
    set_nz_flags(cpu, cpu.acc);
}

// Instruction: Transfer X Register to Stack Pointer
// Function:    stack pointer = X
#[allow(non_snake_case)]
pub fn TXS(cpu: &mut Cpu6502) {
    cpu.stack_pointer = cpu.x_reg;
}

// Instruction: Transfer Y Register to Accumulator
// Function:    A = Y
// Flags Out:   N, Z
#[allow(non_snake_case)]
pub fn TYA(cpu: &mut Cpu6502) {
    cpu.acc = cpu.y_reg;
    set_nz_flags(cpu, cpu.acc);
}

#[allow(non_snake_case)]
pub fn XXX(cpu: &mut Cpu6502) {}

fn set_carry(cpu: &mut Cpu6502, reg: u16) {
    cpu.set_flag(CPUFlags::C, reg & 0xFF00 > 0);
}

fn set_z_if_reg_zero(cpu: &mut Cpu6502, reg: u8) {
    cpu.set_flag(CPUFlags::Z, reg == 0);
}

fn set_n_if_bit_set(cpu: &mut Cpu6502, reg: u8) {
    cpu.set_flag(CPUFlags::N, reg & 0x80 > 0);
}

fn set_nz_flags(cpu: &mut Cpu6502, reg: u8) {
    set_z_if_reg_zero(cpu, reg);
    set_n_if_bit_set(cpu, reg);
}

fn branch_check_page_change(cpu: &mut Cpu6502) {
    if ((cpu.addr_abs & 0xFF00) != (cpu.pc & 0xFF00)) {
        cpu.cycles += 2;
    }
}
