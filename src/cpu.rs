struct CPU {
    accumulator: u8,
    x_index: u8,
    y_index: u8,
    status: Status,
    program_counter: u16,
    stack_pointer: u8
}

struct Status {
    carry: bool,
    zero: bool,
    interrupt: bool,
    decimal: bool,
    something1: bool, // According to nesdev.com/6502.txt this is set when BRK instruction is executed.
    something2: bool,
    overflow: bool,
    sign: bool
}

impl Default for Status {
    fn default() -> Status {
        Status {
            carry: false,
            zero: false,
            interrupt: false,
            decimal: false,
            something1: false,
            something2: false,
            overflow: false,
            sign: false
        }
    }
}







impl CPU {
    fn create_cpu() -> CPU {
        let mut status = { ..Default::default() };
        
        CPU {
            accumulator: 0,
            x_index: 0,
            y_index: 0,
            status,
            program_counter: 0,
            stack_pointer: 0
        }
    }

    fn get_status(&self, bit_index: u8): u8 { // This returns 1 if true, else 0. 
        (self.status >> bit_index) & 1u8
    }

    fn set_status(mut &self, bit_index: u8, status: bool) {
        self.stack_pointer = ( self.stack_pointer & !(1u8 << bit_index) ) | ((status as u8) << bit_index);
    }

    fn step() {
        unimplemented!();
    }


}

