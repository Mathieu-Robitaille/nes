// This is in its own dir so that vscode can ignore the
// massive table since they only ignore dirs for some reason

use crate::instructions::instruction::{AddressingMode, Instruction};
use crate::instructions::instruction_functions::*;
use lazy_static::lazy_static;

lazy_static! {
    // BY THE POWER OF AUTS
    pub static ref INSTRUCTIONS_ARR: [Instruction; 0xFF +1] = {
        let mut m: [Instruction; 0xFF +1] = [Default::default(); 0xFF +1];
        m[0x00] = Instruction { name: "BRK", clock_cycles: 7, addr_mode: AddressingMode::IMP, function: BRK};
        m[0x01] = Instruction { name: "ORA", clock_cycles: 6, addr_mode: AddressingMode::IZX, function: ORA};
        m[0x02] = Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x03] = Instruction { name: "???", clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x04] = Instruction { name: "???", clock_cycles: 3, addr_mode: AddressingMode::IMP, function: NOP};
        m[0x05] = Instruction { name: "ORA", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: ORA};
        m[0x06] = Instruction { name: "ASL", clock_cycles: 5, addr_mode: AddressingMode::ZP0, function: ASL};
        m[0x07] = Instruction { name: "???", clock_cycles: 5, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x08] = Instruction { name: "PHP", clock_cycles: 3, addr_mode: AddressingMode::IMP, function: PHP};
        m[0x09] = Instruction { name: "ORA", clock_cycles: 2, addr_mode: AddressingMode::IMM, function: ORA};
        m[0x0A] = Instruction { name: "ASL", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: ASL};
        m[0x0B] = Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x0C] = Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP};
        m[0x0D] = Instruction { name: "ORA", clock_cycles: 4, addr_mode: AddressingMode::ABS, function: ORA};
        m[0x0E] = Instruction { name: "ASL", clock_cycles: 6, addr_mode: AddressingMode::ABS, function: ASL};
        m[0x0F] = Instruction { name: "???", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x10] = Instruction { name: "BPL", clock_cycles: 2, addr_mode: AddressingMode::REL, function: BPL};
        m[0x11] = Instruction { name: "ORA", clock_cycles: 5, addr_mode: AddressingMode::IZY, function: ORA};
        m[0x12] = Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x13] = Instruction { name: "???", clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x14] = Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP};
        m[0x15] = Instruction { name: "ORA", clock_cycles: 4, addr_mode: AddressingMode::ZPX, function: ORA};
        m[0x16] = Instruction { name: "ASL", clock_cycles: 6, addr_mode: AddressingMode::ZPX, function: ASL};
        m[0x17] = Instruction { name: "???", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x18] = Instruction { name: "CLC", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: CLC};
        m[0x19] = Instruction { name: "ORA", clock_cycles: 4, addr_mode: AddressingMode::ABY, function: ORA};
        m[0x1A] = Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP};
        m[0x1B] = Instruction { name: "???", clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x1C] = Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP};
        m[0x1D] = Instruction { name: "ORA", clock_cycles: 4, addr_mode: AddressingMode::ABX, function: ORA};
        m[0x1E] = Instruction { name: "ASL", clock_cycles: 7, addr_mode: AddressingMode::ABX, function: ASL};
        m[0x1F] = Instruction { name: "???", clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x20] = Instruction { name: "JSR", clock_cycles: 6, addr_mode: AddressingMode::ABS, function: JSR};
        m[0x21] = Instruction { name: "AND", clock_cycles: 6, addr_mode: AddressingMode::IZX, function: AND};
        m[0x22] = Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x23] = Instruction { name: "???", clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x24] = Instruction { name: "BIT", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: BIT};
        m[0x25] = Instruction { name: "AND", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: AND};
        m[0x26] = Instruction { name: "ROL", clock_cycles: 5, addr_mode: AddressingMode::ZP0, function: ROL};
        m[0x27] = Instruction { name: "???", clock_cycles: 5, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x28] = Instruction { name: "PLP", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: PLP};
        m[0x29] = Instruction { name: "AND", clock_cycles: 2, addr_mode: AddressingMode::IMM, function: AND};
        m[0x2A] = Instruction { name: "ROL", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: ROL};
        m[0x2B] = Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x2C] = Instruction { name: "BIT", clock_cycles: 4, addr_mode: AddressingMode::ABS, function: BIT};
        m[0x2D] = Instruction { name: "AND", clock_cycles: 4, addr_mode: AddressingMode::ABS, function: AND};
        m[0x2E] = Instruction { name: "ROL", clock_cycles: 6, addr_mode: AddressingMode::ABS, function: ROL};
        m[0x2F] = Instruction { name: "???", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x30] = Instruction { name: "BMI", clock_cycles: 2, addr_mode: AddressingMode::REL, function: BMI};
        m[0x31] = Instruction { name: "AND", clock_cycles: 5, addr_mode: AddressingMode::IZY, function: AND};
        m[0x32] = Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x33] = Instruction { name: "???", clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x34] = Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP};
        m[0x35] = Instruction { name: "AND", clock_cycles: 4, addr_mode: AddressingMode::ZPX, function: AND};
        m[0x36] = Instruction { name: "ROL", clock_cycles: 6, addr_mode: AddressingMode::ZPX, function: ROL};
        m[0x37] = Instruction { name: "???", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x38] = Instruction { name: "SEC", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: SEC};
        m[0x39] = Instruction { name: "AND", clock_cycles: 4, addr_mode: AddressingMode::ABY, function: AND};
        m[0x3A] = Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP};
        m[0x3B] = Instruction { name: "???", clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x3C] = Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP};
        m[0x3D] = Instruction { name: "AND", clock_cycles: 4, addr_mode: AddressingMode::ABX, function: AND};
        m[0x3E] = Instruction { name: "ROL", clock_cycles: 7, addr_mode: AddressingMode::ABX, function: ROL};
        m[0x3F] = Instruction { name: "???", clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x40] = Instruction { name: "RTI", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: RTI};
        m[0x41] = Instruction { name: "EOR", clock_cycles: 6, addr_mode: AddressingMode::IZX, function: EOR};
        m[0x42] = Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x43] = Instruction { name: "???", clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x44] = Instruction { name: "???", clock_cycles: 3, addr_mode: AddressingMode::IMP, function: NOP};
        m[0x45] = Instruction { name: "EOR", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: EOR};
        m[0x46] = Instruction { name: "LSR", clock_cycles: 5, addr_mode: AddressingMode::ZP0, function: LSR};
        m[0x47] = Instruction { name: "???", clock_cycles: 5, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x48] = Instruction { name: "PHA", clock_cycles: 3, addr_mode: AddressingMode::IMP, function: PHA};
        m[0x49] = Instruction { name: "EOR", clock_cycles: 2, addr_mode: AddressingMode::IMM, function: EOR};
        m[0x4A] = Instruction { name: "LSR", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: LSR};
        m[0x4B] = Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x4C] = Instruction { name: "JMP", clock_cycles: 3, addr_mode: AddressingMode::ABS, function: JMP};
        m[0x4D] = Instruction { name: "EOR", clock_cycles: 4, addr_mode: AddressingMode::ABS, function: EOR};
        m[0x4E] = Instruction { name: "LSR", clock_cycles: 6, addr_mode: AddressingMode::ABS, function: LSR};
        m[0x4F] = Instruction { name: "???", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x50] = Instruction { name: "BVC", clock_cycles: 2, addr_mode: AddressingMode::REL, function: BVC};
        m[0x51] = Instruction { name: "EOR", clock_cycles: 5, addr_mode: AddressingMode::IZY, function: EOR};
        m[0x52] = Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x53] = Instruction { name: "???", clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x54] = Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP};
        m[0x55] = Instruction { name: "EOR", clock_cycles: 4, addr_mode: AddressingMode::ZPX, function: EOR};
        m[0x56] = Instruction { name: "LSR", clock_cycles: 6, addr_mode: AddressingMode::ZPX, function: LSR};
        m[0x57] = Instruction { name: "???", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x58] = Instruction { name: "CLI", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: CLI};
        m[0x59] = Instruction { name: "EOR", clock_cycles: 4, addr_mode: AddressingMode::ABY, function: EOR};
        m[0x5A] = Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP};
        m[0x5B] = Instruction { name: "???", clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x5C] = Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP};
        m[0x5D] = Instruction { name: "EOR", clock_cycles: 4, addr_mode: AddressingMode::ABX, function: EOR};
        m[0x5E] = Instruction { name: "LSR", clock_cycles: 7, addr_mode: AddressingMode::ABX, function: LSR};
        m[0x5F] = Instruction { name: "???", clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x60] = Instruction { name: "RTS", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: RTS};
        m[0x61] = Instruction { name: "ADC", clock_cycles: 6, addr_mode: AddressingMode::IZX, function: ADC};
        m[0x62] = Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x63] = Instruction { name: "???", clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x64] = Instruction { name: "???", clock_cycles: 3, addr_mode: AddressingMode::IMP, function: NOP};
        m[0x65] = Instruction { name: "ADC", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: ADC};
        m[0x66] = Instruction { name: "ROR", clock_cycles: 5, addr_mode: AddressingMode::ZP0, function: ROR};
        m[0x67] = Instruction { name: "???", clock_cycles: 5, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x68] = Instruction { name: "PLA", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: PLA};
        m[0x69] = Instruction { name: "ADC", clock_cycles: 2, addr_mode: AddressingMode::IMM, function: ADC};
        m[0x6A] = Instruction { name: "ROR", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: ROR};
        m[0x6B] = Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x6C] = Instruction { name: "JMP", clock_cycles: 5, addr_mode: AddressingMode::IND, function: JMP};
        m[0x6D] = Instruction { name: "ADC", clock_cycles: 4, addr_mode: AddressingMode::ABS, function: ADC};
        m[0x6E] = Instruction { name: "ROR", clock_cycles: 6, addr_mode: AddressingMode::ABS, function: ROR};
        m[0x6F] = Instruction { name: "???", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x70] = Instruction { name: "BVS", clock_cycles: 2, addr_mode: AddressingMode::REL, function: BVS};
        m[0x71] = Instruction { name: "ADC", clock_cycles: 5, addr_mode: AddressingMode::IZY, function: ADC};
        m[0x72] = Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x73] = Instruction { name: "???", clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x74] = Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP};
        m[0x75] = Instruction { name: "ADC", clock_cycles: 4, addr_mode: AddressingMode::ZPX, function: ADC};
        m[0x76] = Instruction { name: "ROR", clock_cycles: 6, addr_mode: AddressingMode::ZPX, function: ROR};
        m[0x77] = Instruction { name: "???", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x78] = Instruction { name: "SEI", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: SEI};
        m[0x79] = Instruction { name: "ADC", clock_cycles: 4, addr_mode: AddressingMode::ABY, function: ADC};
        m[0x7A] = Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP};
        m[0x7B] = Instruction { name: "???", clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x7C] = Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP};
        m[0x7D] = Instruction { name: "ADC", clock_cycles: 4, addr_mode: AddressingMode::ABX, function: ADC};
        m[0x7E] = Instruction { name: "ROR", clock_cycles: 7, addr_mode: AddressingMode::ABX, function: ROR};
        m[0x7F] = Instruction { name: "???", clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x80] = Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP};
        m[0x81] = Instruction { name: "STA", clock_cycles: 6, addr_mode: AddressingMode::IZX, function: STA};
        m[0x82] = Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP};
        m[0x83] = Instruction { name: "???", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x84] = Instruction { name: "STY", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: STY};
        m[0x85] = Instruction { name: "STA", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: STA};
        m[0x86] = Instruction { name: "STX", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: STX};
        m[0x87] = Instruction { name: "???", clock_cycles: 3, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x88] = Instruction { name: "DEY", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: DEY};
        m[0x89] = Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP};
        m[0x8A] = Instruction { name: "TXA", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: TXA};
        m[0x8B] = Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x8C] = Instruction { name: "STY", clock_cycles: 4, addr_mode: AddressingMode::ABS, function: STY};
        m[0x8D] = Instruction { name: "STA", clock_cycles: 4, addr_mode: AddressingMode::ABS, function: STA};
        m[0x8E] = Instruction { name: "STX", clock_cycles: 4, addr_mode: AddressingMode::ABS, function: STX};
        m[0x8F] = Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x90] = Instruction { name: "BCC", clock_cycles: 2, addr_mode: AddressingMode::REL, function: BCC};
        m[0x91] = Instruction { name: "STA", clock_cycles: 6, addr_mode: AddressingMode::IZY, function: STA};
        m[0x92] = Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x93] = Instruction { name: "???", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x94] = Instruction { name: "STY", clock_cycles: 4, addr_mode: AddressingMode::ZPX, function: STY};
        m[0x95] = Instruction { name: "STA", clock_cycles: 4, addr_mode: AddressingMode::ZPX, function: STA};
        m[0x96] = Instruction { name: "STX", clock_cycles: 4, addr_mode: AddressingMode::ZPY, function: STX};
        m[0x97] = Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x98] = Instruction { name: "TYA", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: TYA};
        m[0x99] = Instruction { name: "STA", clock_cycles: 5, addr_mode: AddressingMode::ABY, function: STA};
        m[0x9A] = Instruction { name: "TXS", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: TXS};
        m[0x9B] = Instruction { name: "???", clock_cycles: 5, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x9C] = Instruction { name: "???", clock_cycles: 5, addr_mode: AddressingMode::IMP, function: NOP};
        m[0x9D] = Instruction { name: "STA", clock_cycles: 5, addr_mode: AddressingMode::ABX, function: STA};
        m[0x9E] = Instruction { name: "???", clock_cycles: 5, addr_mode: AddressingMode::IMP, function: XXX};
        m[0x9F] = Instruction { name: "???", clock_cycles: 5, addr_mode: AddressingMode::IMP, function: XXX};
        m[0xA0] = Instruction { name: "LDY", clock_cycles: 2, addr_mode: AddressingMode::IMM, function: LDY};
        m[0xA1] = Instruction { name: "LDA", clock_cycles: 6, addr_mode: AddressingMode::IZX, function: LDA};
        m[0xA2] = Instruction { name: "LDX", clock_cycles: 2, addr_mode: AddressingMode::IMM, function: LDX};
        m[0xA3] = Instruction { name: "???", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX};
        m[0xA4] = Instruction { name: "LDY", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: LDY};
        m[0xA5] = Instruction { name: "LDA", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: LDA};
        m[0xA6] = Instruction { name: "LDX", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: LDX};
        m[0xA7] = Instruction { name: "???", clock_cycles: 3, addr_mode: AddressingMode::IMP, function: XXX};
        m[0xA8] = Instruction { name: "TAY", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: TAY};
        m[0xA9] = Instruction { name: "LDA", clock_cycles: 2, addr_mode: AddressingMode::IMM, function: LDA};
        m[0xAA] = Instruction { name: "TAX", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: TAX};
        m[0xAB] = Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX};
        m[0xAC] = Instruction { name: "LDY", clock_cycles: 4, addr_mode: AddressingMode::ABS, function: LDY};
        m[0xAD] = Instruction { name: "LDA", clock_cycles: 4, addr_mode: AddressingMode::ABS, function: LDA};
        m[0xAE] = Instruction { name: "LDX", clock_cycles: 4, addr_mode: AddressingMode::ABS, function: LDX};
        m[0xAF] = Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: XXX};
        m[0xB0] = Instruction { name: "BCS", clock_cycles: 2, addr_mode: AddressingMode::REL, function: BCS};
        m[0xB1] = Instruction { name: "LDA", clock_cycles: 5, addr_mode: AddressingMode::IZY, function: LDA};
        m[0xB2] = Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX};
        m[0xB3] = Instruction { name: "???", clock_cycles: 5, addr_mode: AddressingMode::IMP, function: XXX};
        m[0xB4] = Instruction { name: "LDY", clock_cycles: 4, addr_mode: AddressingMode::ZPX, function: LDY};
        m[0xB5] = Instruction { name: "LDA", clock_cycles: 4, addr_mode: AddressingMode::ZPX, function: LDA};
        m[0xB6] = Instruction { name: "LDX", clock_cycles: 4, addr_mode: AddressingMode::ZPY, function: LDX};
        m[0xB7] = Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: XXX};
        m[0xB8] = Instruction { name: "CLV", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: CLV};
        m[0xB9] = Instruction { name: "LDA", clock_cycles: 4, addr_mode: AddressingMode::ABY, function: LDA};
        m[0xBA] = Instruction { name: "TSX", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: TSX};
        m[0xBB] = Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: XXX};
        m[0xBC] = Instruction { name: "LDY", clock_cycles: 4, addr_mode: AddressingMode::ABX, function: LDY};
        m[0xBD] = Instruction { name: "LDA", clock_cycles: 4, addr_mode: AddressingMode::ABX, function: LDA};
        m[0xBE] = Instruction { name: "LDX", clock_cycles: 4, addr_mode: AddressingMode::ABY, function: LDX};
        m[0xBF] = Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: XXX};
        m[0xC0] = Instruction { name: "CPY", clock_cycles: 2, addr_mode: AddressingMode::IMM, function: CPY};
        m[0xC1] = Instruction { name: "CMP", clock_cycles: 6, addr_mode: AddressingMode::IZX, function: CMP};
        m[0xC2] = Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP};
        m[0xC3] = Instruction { name: "???", clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX};
        m[0xC4] = Instruction { name: "CPY", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: CPY};
        m[0xC5] = Instruction { name: "CMP", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: CMP};
        m[0xC6] = Instruction { name: "DEC", clock_cycles: 5, addr_mode: AddressingMode::ZP0, function: DEC};
        m[0xC7] = Instruction { name: "???", clock_cycles: 5, addr_mode: AddressingMode::IMP, function: XXX};
        m[0xC8] = Instruction { name: "INY", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: INY};
        m[0xC9] = Instruction { name: "CMP", clock_cycles: 2, addr_mode: AddressingMode::IMM, function: CMP};
        m[0xCA] = Instruction { name: "DEX", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: DEX};
        m[0xCB] = Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX};
        m[0xCC] = Instruction { name: "CPY", clock_cycles: 4, addr_mode: AddressingMode::ABS, function: CPY};
        m[0xCD] = Instruction { name: "CMP", clock_cycles: 4, addr_mode: AddressingMode::ABS, function: CMP};
        m[0xCE] = Instruction { name: "DEC", clock_cycles: 6, addr_mode: AddressingMode::ABS, function: DEC};
        m[0xCF] = Instruction { name: "???", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX};
        m[0xD0] = Instruction { name: "BNE", clock_cycles: 2, addr_mode: AddressingMode::REL, function: BNE};
        m[0xD1] = Instruction { name: "CMP", clock_cycles: 5, addr_mode: AddressingMode::IZY, function: CMP};
        m[0xD2] = Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX};
        m[0xD3] = Instruction { name: "???", clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX};
        m[0xD4] = Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP};
        m[0xD5] = Instruction { name: "CMP", clock_cycles: 4, addr_mode: AddressingMode::ZPX, function: CMP};
        m[0xD6] = Instruction { name: "DEC", clock_cycles: 6, addr_mode: AddressingMode::ZPX, function: DEC};
        m[0xD7] = Instruction { name: "???", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX};
        m[0xD8] = Instruction { name: "CLD", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: CLD};
        m[0xD9] = Instruction { name: "CMP", clock_cycles: 4, addr_mode: AddressingMode::ABY, function: CMP};
        m[0xDA] = Instruction { name: "NOP", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP};
        m[0xDB] = Instruction { name: "???", clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX};
        m[0xDC] = Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP};
        m[0xDD] = Instruction { name: "CMP", clock_cycles: 4, addr_mode: AddressingMode::ABX, function: CMP};
        m[0xDE] = Instruction { name: "DEC", clock_cycles: 7, addr_mode: AddressingMode::ABX, function: DEC};
        m[0xDF] = Instruction { name: "???", clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX};
        m[0xE0] = Instruction { name: "CPX", clock_cycles: 2, addr_mode: AddressingMode::IMM, function: CPX};
        m[0xE1] = Instruction { name: "SBC", clock_cycles: 6, addr_mode: AddressingMode::IZX, function: SBC};
        m[0xE2] = Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP};
        m[0xE3] = Instruction { name: "???", clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX};
        m[0xE4] = Instruction { name: "CPX", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: CPX};
        m[0xE5] = Instruction { name: "SBC", clock_cycles: 3, addr_mode: AddressingMode::ZP0, function: SBC};
        m[0xE6] = Instruction { name: "INC", clock_cycles: 5, addr_mode: AddressingMode::ZP0, function: INC};
        m[0xE7] = Instruction { name: "???", clock_cycles: 5, addr_mode: AddressingMode::IMP, function: XXX};
        m[0xE8] = Instruction { name: "INX", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: INX};
        m[0xE9] = Instruction { name: "SBC", clock_cycles: 2, addr_mode: AddressingMode::IMM, function: SBC};
        m[0xEA] = Instruction { name: "NOP", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP};
        m[0xEB] = Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: SBC};
        m[0xEC] = Instruction { name: "CPX", clock_cycles: 4, addr_mode: AddressingMode::ABS, function: CPX};
        m[0xED] = Instruction { name: "SBC", clock_cycles: 4, addr_mode: AddressingMode::ABS, function: SBC};
        m[0xEE] = Instruction { name: "INC", clock_cycles: 6, addr_mode: AddressingMode::ABS, function: INC};
        m[0xEF] = Instruction { name: "???", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX};
        m[0xF0] = Instruction { name: "BEQ", clock_cycles: 2, addr_mode: AddressingMode::REL, function: BEQ};
        m[0xF1] = Instruction { name: "SBC", clock_cycles: 5, addr_mode: AddressingMode::IZY, function: SBC};
        m[0xF2] = Instruction { name: "???", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: XXX};
        m[0xF3] = Instruction { name: "???", clock_cycles: 8, addr_mode: AddressingMode::IMP, function: XXX};
        m[0xF4] = Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP};
        m[0xF5] = Instruction { name: "SBC", clock_cycles: 4, addr_mode: AddressingMode::ZPX, function: SBC};
        m[0xF6] = Instruction { name: "INC", clock_cycles: 6, addr_mode: AddressingMode::ZPX, function: INC};
        m[0xF7] = Instruction { name: "???", clock_cycles: 6, addr_mode: AddressingMode::IMP, function: XXX};
        m[0xF8] = Instruction { name: "SED", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: SED};
        m[0xF9] = Instruction { name: "SBC", clock_cycles: 4, addr_mode: AddressingMode::ABY, function: SBC};
        m[0xFA] = Instruction { name: "NOP", clock_cycles: 2, addr_mode: AddressingMode::IMP, function: NOP};
        m[0xFB] = Instruction { name: "???", clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX};
        m[0xFC] = Instruction { name: "???", clock_cycles: 4, addr_mode: AddressingMode::IMP, function: NOP};
        m[0xFD] = Instruction { name: "SBC", clock_cycles: 4, addr_mode: AddressingMode::ABX, function: SBC};
        m[0xFE] = Instruction { name: "INC", clock_cycles: 7, addr_mode: AddressingMode::ABX, function: INC};
        m[0xFF] = Instruction { name: "???", clock_cycles: 7, addr_mode: AddressingMode::IMP, function: XXX};
        m
    };
}
