enum Instruction {
    INVALID,
    ADC, AND, ASL, BCC, BCS, BEQ, BIT, 
    BMI, BNE, BPL, BRK, BVC, BVS, CLC,
    CLD, CLI, CLV, CMP, CPX, CPY, DEC,
    DEX, DEY, EOR, INC, INX, INY, JMP,
    JSR, LDA, LDX, LDY, LSR, NOP, ORA,
    PHA, PHP, PLA, PLP, ROL, ROR, RTI,
    RTS, SBC, SEC, SED, SEI, STA, STX,
    STY, TAX, TAY, TSX, TXA, TXS, TYA,
}

enum AddressMode {
    INVALID,
    abs, abs_x, abs_y, // Absolute  (indexed)
    ind, ind_x, ind_y, // Indirect  (indexed)
    zpg, zpg_x, zpg_y, // Zero page (indexed)
    imp, // Implied
    rel, // Relative
    acc, // Accumulator
    imm // Immediate
}

