pub mod bus;
pub mod ram;

use bus::Bus;

const MASTER_CLOCK_FREQUENCY: u32 = 21477272; // Hz


pub struct Cpu {
    accumulator: u8,
    x_index: u8,
    y_index: u8,
    status: Status,
    program_counter: u16,
    stack_pointer: u8,
    bus: Bus,
}

struct Status {
    carry: bool,
    zero: bool,
    interrupt: bool,
    decimal: bool,
    something1: bool, // According to nesdev.com/6502.txt this is set when BRK instruction is executed.
    something2: bool,
    overflow: bool,
    sign: bool,
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
            sign: false,
        }
    }
}


impl Cpu {
    pub fn new(bus: Bus) -> Cpu {
        let status = Status::default();

        Cpu {
            accumulator: 0,
            x_index: 0,
            y_index: 0,
            status: { Status::default() },
            program_counter: 0,
            stack_pointer: 0,
            bus,
        }
    }

    fn read_8(&self) -> u8 {
        unimplemented!();
    }

    fn read_16(&self) -> u16 {
        unimplemented!();
    }

    pub fn step(&mut self) {
        unimplemented!();
    }

}
