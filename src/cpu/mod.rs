pub mod opcode;
pub mod ram;
pub mod bus;
mod instruction;
mod address_mode;

use crate::cpu::bus::Bus;


pub struct Cpu {
    pub accumulator: u8,
    pub x_index: u8,
    pub y_index: u8,
    pub status: Status,
    pub program_counter: u16,
    pub stack_pointer: u8,
    pub skip_cycles: u8,
    pub bus: Bus,
    pub cycle: u64,
    pub page_crossed: bool,
    oamdma_cycles_left: u16,
}

pub struct Status {
    pub carry: bool,
    pub zero: bool,
    pub interrupt: bool,
    pub decimal: bool,
    pub something1: bool, // According to nesdev.com/6502.txt this is set when BRK instruction is executed.
    pub something2: bool,
    pub overflow: bool,
    pub negative: bool,
}

impl Default for Status {
    fn default() -> Status {
        Status {
            carry: false,
            zero: false,
            interrupt: true,
            decimal: false,
            something1: true,
            something2: true,
            overflow: false,
            negative: false,
        }
    }
}
impl Status {
    pub fn get_as_byte(&self) -> u8 {
        let mut result: u8 = self.negative as u8;
        result = (result << 1) | self.overflow as u8;
        result = (result << 1) | 1;
        result <<= 1;
        result = (result << 1) | self.decimal as u8;
        result = (result << 1) | self.interrupt as u8;
        result = (result << 1) | self.zero as u8;
        result = (result << 1) | self.carry as u8;
        result
    }
}


impl Cpu {
    pub fn new(bus: Bus) -> Cpu {
        let mut cpu =
            Cpu {
                accumulator: 0,
                x_index: 0,
                y_index: 0,
                status: { Status::default() },
                program_counter: 0,
                stack_pointer: 0xFD,
                skip_cycles: 0,
                bus,
                cycle: 7, // TODO: fix cpu so that this can init as 0.
                page_crossed: false,
                oamdma_cycles_left: 0,
            };
        cpu.reset_program_counter();
        cpu
    }

    fn read_8(&mut self, address: u16) -> u8 {
        self.bus.read(address)
    }

    fn read_16(&mut self, address: u16) -> u16 {
        let lower_byte = self.bus.read(address) as u16;
        let higher_byte = self.bus.read(address + 1) as u16;
        let a = higher_byte << 8;
        lower_byte | a
    }

    fn write_8(&mut self, address: u16, value: u8) {
        self.bus.write(address, value);
    }

    fn stack_push_8(&mut self, value: u8) {
        self.write_8(0x0100 + self.stack_pointer as u16, value);
        self.stack_pointer= self.stack_pointer.wrapping_sub(1);
    }

    fn stack_push_16(&mut self, value: u16) {
        self.stack_push_8((value >> 8)   as u8);
        self.stack_push_8((value & 0xFF) as u8);
    }

    fn stack_pop_8(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.read_8(0x0100 + self.stack_pointer as u16)
    }

    fn stack_pop_16(&mut self) -> u16 {
        (self.stack_pop_8() as u16) | ((self.stack_pop_8() as u16) << 8)
    }

    fn reset_program_counter(&mut self) {
        self.program_counter = self.read_16(0xFFFC);
    }

    pub fn set_program_counter(&mut self, new_count: u16) {
        self.program_counter = new_count;
    }

    pub fn step(&mut self) {
        self.cycle +=1;
        if self.is_interrupted_by_nmi() {
            self.handle_nmi();
        }
        
        if self.skip_cycles > 0 {
            self.skip_cycles -= 1;
            return;
        }

        if self.bus.oamdma_occurred {
            self.bus.oamdma_occurred = false;
            self.oamdma_cycles_left = 513 + ((self.cycle & 1) as u16); // TODO timing of 1st (and 2nd idle cycle)
        }

        if self.oamdma_cycles_left >= 513 {
            self.oamdma_cycles_left -= 1;
            return;
        }
        else if self.oamdma_cycles_left > 0 {
            if (1..=512).contains(&self.oamdma_cycles_left) && self.oamdma_cycles_left % 2 == 1 {
                let address = self.bus.oamdma_high_byte | ((0x200 - self.oamdma_cycles_left) >> 1);
                let value = self.read_8(address);
                self.bus.ppu.as_mut().unwrap().write_oamdma(value);
            }
            self.oamdma_cycles_left -= 1;
            return;
        }
        self.execute_next_opcode();
    }

    pub fn is_interrupted_by_nmi(&mut self) -> bool {
        match self.bus.ppu {
            Some(ref mut ppu) => ppu.is_nmi(),
            None => false
        }
    }

    pub fn handle_nmi(&mut self) {
        self.stack_push_16(self.program_counter); // TODO: Check if stack pushes are in correct order
        self.stack_push_8(self.status.get_as_byte());
        let nmi_address = self.read_16(0xFFFA);
        self.set_program_counter(nmi_address);
    }

    pub fn get_next_opcode(&mut self) -> u8 {
        self.read_8(self.program_counter)
    }

    pub fn execute_next_opcode(&mut self) {
        let next_opcode = self.get_next_opcode();
        let op = opcode::opcode_mapper(next_opcode);
        self.program_counter += 1;
        self.skip_cycles += op.cycles - 1;
        self.page_crossed = false; // Reset page_crossed flag
        let address = self.execute_address_mode(&op.address_mode);
        self.execute_instruction(&op, address);
    }

    fn crossing_page(&mut self, address_1: u16, address_2: u16) -> bool {
        address_1 & 0xFF00 != address_2 & 0xFF00
    }

    fn execute_address_mode(&mut self, address_mode: &address_mode::AddressMode) -> u16 {
        use address_mode::AddressMode::*;
        match address_mode {
            Abs  => self.abs(),
            AbsX => self.abs_x(),
            AbsY => self.abs_y(),
            Ind  => self.ind(),
            IndX => self.ind_x(),
            IndY => self.ind_y(),
            Zpg  => self.zpg(),
            ZpgX => self.zpg_x(),
            ZpgY => self.zpg_y(),
            Imp  => self.imp(),
            Rel  => self.rel(),
            Acc  => self.acc(),
            Imm  => self.imm(),
            Invalid => panic!("Unsupported address mode.")
        }
    }

    fn execute_instruction(&mut self, opcode: &opcode::Opcode, address: u16) {
        use instruction::Instruction::*;
        use address_mode::AddressMode::Acc;
        match (&opcode.instruction, &opcode.address_mode) {
            (ADC, _)     => self.adc(address),
            (AND, _)     => self.and(address),
            (ASL, Acc)   => self.asl_acc(address),
            (ASL, _)     => self.asl(address),
            (BCC, _)     => self.bcc(address),
            (BCS, _)     => self.bcs(address),
            (BEQ, _)     => self.beq(address),
            (BIT, _)     => self.bit(address),
            (BMI, _)     => self.bmi(address),
            (BNE, _)     => self.bne(address),
            (BPL, _)     => self.bpl(address),
            (BRK, _)     => self.brk(address),
            (BVC, _)     => self.bvc(address),
            (BVS, _)     => self.bvs(address),
            (CLC, _)     => self.clc(address),
            (CLD, _)     => self.cld(address),
            (CLI, _)     => self.cli(address),
            (CLV, _)     => self.clv(address),
            (CMP, _)     => self.cmp(address),
            (CPX, _)     => self.cpx(address),
            (CPY, _)     => self.cpy(address),
            (DEC, _)     => self.dec(address),
            (DEX, _)     => self.dex(address),
            (DEY, _)     => self.dey(address),
            (EOR, _)     => self.eor(address),
            (INC, _)     => self.inc(address),
            (INX, _)     => self.inx(address),
            (INY, _)     => self.iny(address),
            (JMP, _)     => self.jmp(address),
            (JSR, _)     => self.jsr(address),
            (LDA, _)     => self.lda(address),
            (LDX, _)     => self.ldx(address),
            (LDY, _)     => self.ldy(address),
            (LSR, Acc)   => self.lsr_acc(address),
            (LSR, _)     => self.lsr(address),
            (NOP, _)     => self.nop(address),
            (ORA, _)     => self.ora(address),
            (PHA, _)     => self.pha(address),
            (PHP, _)     => self.php(address),
            (PLA, _)     => self.pla(address),
            (PLP, _)     => self.plp(address),
            (ROL, Acc)   => self.rol_acc(address),
            (ROL, _)     => self.rol(address),
            (ROR, Acc)   => self.ror_acc(address),
            (ROR, _)     => self.ror(address),
            (RTI, _)     => self.rti(address),
            (RTS, _)     => self.rts(address),
            (SBC, _)     => self.sbc(address),
            (SEC, _)     => self.sec(address),
            (SED, _)     => self.sed(address),
            (SEI, _)     => self.sei(address),
            (STA, _)     => self.sta(address),
            (STX, _)     => self.stx(address),
            (STY, _)     => self.sty(address),
            (TAX, _)     => self.tax(address),
            (TAY, _)     => self.tay(address),
            (TSX, _)     => self.tsx(address),
            (TXA, _)     => self.txa(address),
            (TXS, _)     => self.txs(address),
            (TYA, _)     => self.tya(address),
            (Invalid, _) => panic!("Unsupported opcode"),
        }
    }
}
