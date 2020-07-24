use super::Cpu;
use std::fmt;
#[allow(non_camel_case_types)]
pub enum AddressMode {
    Invalid,
    Abs, AbsX, AbsY, // Absolute  (indexed)
    Ind, IndX, IndY, // Indirect  (indexed)
    Zpg, ZpgX, ZpgY, // Zero page (indexed)
    Imp, // Implied
    Rel, // Relative
    Acc, // Accumulator
    Imm  // Immediate
}

impl Cpu {
    pub fn abs  (&mut self) -> u16 {
        let address = self.read_16(self.program_counter);
        self.program_counter += 2;
        address
    }

    pub fn abs_x(&mut self) -> u16 {
        let address_low = self.read_8(self.program_counter) as u16;
        self.program_counter += 1;
        let address_high = self.read_8(self.program_counter) as u16;
        self.program_counter += 1;
        let address_low_x = address_low.wrapping_add(self.x_index as u16);
        self.page_crossed = address_low_x > 0xFF;
        address_low_x.wrapping_add(address_high << 8)
    }

    pub fn abs_y(&mut self) -> u16 {
        let address_low = self.read_8(self.program_counter) as u16;
        self.program_counter += 1;
        let address_high = self.read_8(self.program_counter) as u16;
        self.program_counter += 1;
        let address_low_y = address_low.wrapping_add(self.y_index as u16);
        self.page_crossed = address_low_y > 0xFF;
        address_low_y.wrapping_add(address_high << 8)
    }

    pub fn ind(&mut self) -> u16 {
        let address = self.read_16(self.program_counter);
        self.program_counter += 2;

        let byte_low = self.read_8(address) as u16;
        let byte_high = self.read_8((address & 0xFF00) | (address.wrapping_add(1) & 0x00FF)) as u16;
        return byte_low | (byte_high << 8);
    }

    pub fn ind_x(&mut self) -> u16 {
        let a = self.read_8(self.program_counter).wrapping_add(self.x_index);
        self.program_counter += 1;
        let b = a.wrapping_add(1);
        (self.read_8(a as u16) as u16).wrapping_add((self.read_8(b as u16) as u16) << 8)
    }

    pub fn ind_y(&mut self) -> u16 {
        let a = self.read_8(self.program_counter);
        self.program_counter += 1;
        let b = a.wrapping_add(1);
        let address = (self.read_8(a as u16) as u16).wrapping_add((self.read_8(b as u16) as u16) << 8);
        let address_y = address.wrapping_add(self.y_index as u16);
        self.page_crossed = self.crossing_page(address, address_y);
        address_y
    }

    pub fn zpg(&mut self) -> u16 {
        let value = self.read_8(self.program_counter);
        self.program_counter += 1;
        value as u16
    }

    pub fn zpg_x(&mut self) -> u16 {
        let value = self.read_8(self.program_counter).wrapping_add(self.x_index);
        self.program_counter += 1;
        value as u16
    }

    pub fn zpg_y(&mut self) -> u16 {
        let value = self.read_8(self.program_counter).wrapping_add(self.y_index);
        self.program_counter += 1;
        value as u16
    }

    pub fn imp(&self) -> u16 {
        0
    }

    pub fn rel(&mut self) -> u16 {
        let offset = self.read_8(self.program_counter) as u16;
        self.program_counter += 1;

        if offset < 0x80u16 {
            offset.wrapping_add(self.program_counter)
        }
        else {
            offset.wrapping_add(self.program_counter).wrapping_add(0xFF00u16)
        }
    }

    pub fn acc  (&self) -> u16 {
        0
    }

    pub fn imm  (&mut self) -> u16 {
        let pc = self.program_counter;
        self.program_counter += 1;
        pc
    }
}