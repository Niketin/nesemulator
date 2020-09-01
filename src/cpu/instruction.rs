use super::Cpu;

pub enum Instruction {
    Invalid,
    ADC, AND, ASL, BCC, BCS, BEQ, BIT,
    BMI, BNE, BPL, BRK, BVC, BVS, CLC,
    CLD, CLI, CLV, CMP, CPX, CPY, DEC,
    DEX, DEY, EOR, INC, INX, INY, JMP,
    JSR, LDA, LDX, LDY, LSR, NOP, ORA,
    PHA, PHP, PLA, PLP, ROL, ROR, RTI,
    RTS, SBC, SEC, SED, SEI, STA, STX,
    STY, TAX, TAY, TSX, TXA, TXS, TYA,
}

impl Cpu {
    pub fn adc(&mut self, address: u16) {
        let value = self.read_8(address);
        let values_added = value as u16 + self.accumulator as u16 + self.status.carry as u16;

        self.status.carry = values_added > 0xFF;
        self.status.zero = values_added & 0xFF == 0;
        self.status.negative = values_added & 0x80 == 0x80;
        self.status.overflow = value & 0x80 == self.accumulator & 0x80 && value as u16 & 0x80 != values_added & 0x80;
        self.accumulator = (values_added & 0xFF) as u8;
        if self.page_crossed { self.skip_cycles += 1; }
    }

    pub fn and(&mut self, address: u16) {
        let value = self.read_8(address) & self.accumulator;
        self.status.negative = value & 0x80 == 0x80;
        self.status.zero = value == 0;
        self.accumulator = value;
        if self.page_crossed { self.skip_cycles += 1; }
    }

    pub fn asl(&mut self, address: u16) {
        let mut value = self.read_8(address);
        self.status.carry = value & 0x80 == 0x80;
        value <<= 1;
        self.status.negative = value & 0x80 == 0x80;
        self.status.zero = value == 0;
        self.write_8(address, value);
    }

    pub fn asl_acc(&mut self, _address: u16) {
        let mut value = self.accumulator;
        self.status.carry = value & 0x80 == 0x80;
        value <<= 1;
        self.status.negative = value & 0x80 == 0x80;
        self.status.zero = value == 0;
        self.accumulator = value;
    }

    pub fn bcc(&mut self, address: u16) {
        if !self.status.carry {
            if self.crossing_page(address, self.program_counter) { self.skip_cycles += 1;}
            self.program_counter = address;
            self.skip_cycles += 1;
        }
    }

    pub fn bcs(&mut self, address: u16) {
        if self.status.carry {
            if self.crossing_page(address, self.program_counter) { self.skip_cycles += 1;}
            self.program_counter = address;
            self.skip_cycles += 1;
        }
    }

    pub fn beq(&mut self, address: u16) {
        if self.status.zero {
            if self.crossing_page(address, self.program_counter) { self.skip_cycles += 1;}
            self.program_counter = address;
            self.skip_cycles += 1;
        }
    }

    pub fn bit(&mut self, address: u16) {
        let value = self.read_8(address);
        let value_masked =  value & self.accumulator;
        self.status.zero = value_masked == 0;
        self.status.negative = value & 0x80 == 0x80;
        self.status.overflow = value & 0x40 == 0x40;
    }

    pub fn bmi(&mut self, address: u16) {
        if self.status.negative {
            if self.crossing_page(address, self.program_counter) { self.skip_cycles += 1;}
            self.program_counter = address;
            self.skip_cycles += 1;
        }
    }

    pub fn bne(&mut self, address: u16) {
        if !self.status.zero {
            if self.crossing_page(address, self.program_counter) { self.skip_cycles += 1;}
            self.program_counter = address;
            self.skip_cycles += 1;
        }
    }

    pub fn bpl(&mut self, address: u16) {
        if !self.status.negative {
            if self.crossing_page(address, self.program_counter) { self.skip_cycles += 1;}
            self.program_counter = address;
            self.skip_cycles += 1;
        }
    }

    pub fn brk(&mut self, _address: u16) {
        self.program_counter += 1; // BRK is actually 2-byte instruction
        self.stack_push_16(self.program_counter);
        self.stack_push_8(self.status.get_as_byte());
        self.status.interrupt = true;
        let irq_vector = self.read_16(0xfffe);
        self.program_counter = irq_vector;
    }

    pub fn bvc(&mut self, address: u16) {
        if !self.status.overflow {
            if self.crossing_page(address, self.program_counter) { self.skip_cycles += 1;}
            self.program_counter = address;
            self.skip_cycles += 1;
        }
    }

    pub fn bvs(&mut self, address: u16) {
        if self.status.overflow {
            if self.crossing_page(address, self.program_counter) { self.skip_cycles += 1;}
            self.program_counter = address;
            self.skip_cycles += 1;
        }
    }

    pub fn clc(&mut self, _address: u16) {
        self.status.carry = false;
    }

    pub fn cld(&mut self, _address: u16) {
        self.status.decimal = false;
    }

    pub fn cli(&mut self, _address: u16) {
        self.status.interrupt = false;
    }

    pub fn clv(&mut self, _address: u16) {
        self.status.overflow = false;
    }

    pub fn cmp(&mut self, address: u16) {
        let value = (self.accumulator as u16).wrapping_sub(self.read_8(address) as u16);
        self.status.carry = value < 0x100;
        self.status.zero = value == 0;
        self.status.negative = value & 0x80 == 0x80;
        if self.page_crossed { self.skip_cycles += 1; }
    }

    pub fn cpx(&mut self, address: u16) {
        let value = (self.x_index as u16).wrapping_sub(self.read_8(address) as u16);
        self.status.carry = value < 0x100;
        self.status.zero = value == 0;
        self.status.negative = value & 0x80 == 0x80;
    }

    pub fn cpy(&mut self, address: u16) {
        let value = (self.y_index as u16).wrapping_sub(self.read_8(address) as u16);
        self.status.carry = value < 0x100;
        self.status.zero = value == 0;
        self.status.negative = value & 0x80 == 0x80;
    }

    pub fn dec(&mut self, address: u16) {
        let value = self.read_8(address).wrapping_sub(1);
        self.status.zero = value == 0;
        self.status.negative = value & 0x80 == 0x80;
        self.write_8(address, value);
    }

    pub fn dex(&mut self, _address: u16) {
        self.x_index = self.x_index.wrapping_sub(1);
        self.status.zero = self.x_index == 0;
        self.status.negative = self.x_index & 0x80 == 0x80;
    }

    pub fn dey(&mut self, _address: u16) {
        self.y_index = self.y_index.wrapping_sub(1);
        self.status.zero = self.y_index == 0;
        self.status.negative = self.y_index & 0x80 == 0x80;
    }

    pub fn eor(&mut self, address: u16) {
        self.accumulator ^= self.read_8(address);
        self.status.zero = self.accumulator == 0;
        self.status.negative = self.accumulator & 0x80 == 0x80;
        if self.page_crossed { self.skip_cycles += 1; }
    }

    pub fn inc(&mut self, address: u16) {
        let value = self.read_8(address).wrapping_add(1);
        self.status.zero = value == 0;
        self.status.negative = value & 0x80 == 0x80;
        self.write_8(address, value);
    }

    pub fn inx(&mut self, _address: u16) {
        self.x_index = self.x_index.wrapping_add(1);
        self.status.zero = self.x_index == 0;
        self.status.negative = self.x_index & 0x80 == 0x80;
    }

    pub fn iny(&mut self, _address: u16) {
        self.y_index = self.y_index.wrapping_add(1);
        self.status.zero = self.y_index == 0;
        self.status.negative = self.y_index & 0x80 == 0x80;
    }

    pub fn jmp(&mut self, address: u16) {
        self.program_counter = address;
    }

    pub fn jsr(&mut self, address: u16) {
        self.stack_push_16(self.program_counter - 1);
        self.program_counter = address;
    }

    pub fn lda(&mut self, address: u16) {
        self.accumulator = self.read_8(address);
        self.status.zero = self.accumulator == 0;
        self.status.negative = self.accumulator & 0x80 == 0x80;
        if self.page_crossed { self.skip_cycles += 1; }
    }

    pub fn ldx(&mut self, address: u16) {
        self.x_index = self.read_8(address);
        self.status.zero = self.x_index == 0;
        self.status.negative = self.x_index & 0x80 == 0x80;
        if self.page_crossed { self.skip_cycles += 1; }
    }

    pub fn ldy(&mut self, address: u16) {
        self.y_index = self.read_8(address);
        self.status.zero = self.y_index == 0;
        self.status.negative = self.y_index & 0x80 == 0x80;
        if self.page_crossed { self.skip_cycles += 1; }
    }

    pub fn lsr(&mut self, address: u16) {
        let value = self.read_8(address);
        let value_shifted_right = value >> 1;
        self.status.zero = value_shifted_right == 0;
        self.status.carry = value & 0x01 == 0x01;
        self.status.negative = false;
        self.write_8(address, value_shifted_right);
    }

    pub fn lsr_acc(&mut self, _address: u16) {
        let value = self.accumulator;
        let value_shifted_right = value >> 1;
        self.status.zero = value_shifted_right == 0;
        self.status.carry = value & 0x01 == 0x01;
        self.status.negative = false;
        self.accumulator = value_shifted_right;
    }

    pub fn nop(&mut self, _address: u16) {}

    pub fn ora(&mut self, address: u16) {
        self.accumulator = self.read_8(address) | self.accumulator;
        self.status.zero = self.accumulator == 0;
        self.status.negative = self.accumulator & 0x80 == 0x80;
        if self.page_crossed { self.skip_cycles += 1; }
    }

    pub fn pha(&mut self, _address: u16) {
        self.stack_push_8(self.accumulator);
    }

    pub fn php(&mut self, _address: u16) {
        let mut status: u8 = self.status.negative as u8;
        status = (status << 1) | self.status.overflow as u8;
        status = (status << 1) | 1; // http://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
        status = (status << 1) | 1; // http://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
        status = (status << 1) | self.status.decimal as u8;
        status = (status << 1) | self.status.interrupt as u8;
        status = (status << 1) | self.status.zero as u8;
        status = (status << 1) | self.status.carry as u8;
        self.stack_push_8(status);
    }

    pub fn pla(&mut self, _address: u16) {
        self.accumulator = self.stack_pop_8();
        self.status.zero = self.accumulator == 0;
        self.status.negative = self.accumulator & 0x80 == 0x80;
    }

    pub fn plp(&mut self, _address: u16) {
        let status = self.stack_pop_8();
        self.status.carry = status & 0x01 == 0x01;
        self.status.zero = (status >> 1) & 0x01 == 0x01;
        self.status.interrupt = (status >> 2) & 0x01 == 0x01;
        self.status.decimal = (status >> 3) & 0x01 == 0x01;
        // Bits 5 and 4 are ignored. http://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
        self.status.overflow = (status >> 6) & 0x01 == 0x01;
        self.status.negative = (status >> 7) & 0x01 == 0x01;
    }

    pub fn rol(&mut self, address: u16) {
        let old_carry = self.status.carry as u8;
        let value = self.read_8(address);
        let new_carry = value >> 7 == 1;
        let new_value = (value << 1) | old_carry;
        self.status.carry = new_carry;
        self.status.negative = new_value & 0x80 == 0x80;
        self.write_8(address, new_value);
    }

    pub fn rol_acc(&mut self, _address: u16) {
        let old_carry = self.status.carry as u8;
        let value = self.accumulator;
        let new_carry = value >> 7 == 1;
        let new_value = (value << 1) | old_carry;
        self.status.carry = new_carry;
        self.status.negative = new_value & 0x80 == 0x80;
        self.accumulator = new_value;
    }

    pub fn ror(&mut self, address: u16) {
        let old_carry = self.status.carry as u8;
        let value = self.read_8(address);
        let new_carry = value & 0x01 == 1;
        let new_value = (value >> 1) | (old_carry << 7);
        self.status.carry = new_carry;
        self.status.negative = new_value & 0x80 == 0x80;
        self.write_8(address, new_value);
    }

    pub fn ror_acc(&mut self, _address: u16) {
        let old_carry = self.status.carry as u8;
        let value = self.accumulator;
        let new_carry = value & 0x01 == 1;
        let new_value = (value >> 1) | (old_carry << 7);
        self.status.carry = new_carry;
        self.status.negative = new_value & 0x80 == 0x80;
        self.accumulator = new_value;
    }

    pub fn rti(&mut self, address: u16) {
        self.plp(address);
        self.program_counter = self.stack_pop_16();
    }

    pub fn rts(&mut self, _address: u16) {
        self.program_counter = self.stack_pop_16() + 1;
    }

    pub fn sbc(&mut self, address: u16) {
        let value = self.read_8(address) as u16;
        let carry = self.status.carry as u16;

        let result = (self.accumulator as u16).wrapping_sub(value).wrapping_sub(1 - carry);

        self.status.carry = result < 0x100;
        self.status.zero = result & 0xFF == 0;
        self.status.negative = result & 0x80 == 0x80;
        self.status.overflow =
            value & 0x80 != self.accumulator as u16 & 0x80 &&
            value & 0x80 == result & 0x80;
        self.accumulator = (result & 0xFF) as u8;
        if self.page_crossed { self.skip_cycles += 1; }
    }

    pub fn sec(&mut self, _address: u16) {
        self.status.carry = true;
    }

    pub fn sed(&mut self, _address: u16) {
        self.status.decimal = true;
    }

    pub fn sei(&mut self, _address: u16) {
        self.status.interrupt = true;
    }

    pub fn sta(&mut self, address: u16) {
        self.write_8(address, self.accumulator);
    }

    pub fn stx(&mut self, address: u16) {
        self.write_8(address, self.x_index);
    }

    pub fn sty(&mut self, address: u16) {
        self.write_8(address, self.y_index);
    }

    pub fn tax(&mut self, _address: u16) {
        self.x_index = self.accumulator;
        self.status.zero = self.x_index == 0;
        self.status.negative = self.x_index & 0x80 == 0x80;
    }

    pub fn tay(&mut self, _address: u16) {
        self.y_index = self.accumulator;
        self.status.zero = self.y_index == 0;
        self.status.negative = self.y_index & 0x80 == 0x80;
    }

    pub fn tsx(&mut self, _address: u16) {
        self.x_index = self.stack_pointer;
        self.status.zero = self.x_index == 0;
        self.status.negative = self.x_index & 0x80 == 0x80;
    }

    pub fn txa(&mut self, _address: u16) {
        self.accumulator = self.x_index;
        self.status.zero = self.accumulator == 0;
        self.status.negative = self.accumulator & 0x80 == 0x80;
    }

    pub fn txs(&mut self, _address: u16) {
        self.stack_pointer = self.x_index;
    }

    pub fn tya(&mut self, _address: u16) {
        self.accumulator = self.y_index;
        self.status.zero = self.accumulator == 0;
        self.status.negative = self.accumulator & 0x80 == 0x80;
    }
}