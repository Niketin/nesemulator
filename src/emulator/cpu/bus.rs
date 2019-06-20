use crate::emulator::cpu::ram::Ram;

pub struct Bus {
    ram: Ram,
}

impl Bus {
    pub fn new(mut ram: Ram) -> Bus {
        let size = ram.size;
        if size != 0x0800 {
            panic!("Creating a new Bus: CPU RAM does not have correct size (0x0800)");
        }
        Bus { ram }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x0000...0x07FF => self.ram.read(address as usize), // CPU RAM
            0x0800...0x1FFF => unimplemented!(),                // CPU RAM (mirror)
            0x2000...0x2007 => unimplemented!(),                // PPU registers
            0x2008...0x3FFF => unimplemented!(),                // PPU registers (mirror)
            0x4000...0x401F => unimplemented!(),                // Other IO devices?
            0x6000...0xFFFF => unimplemented!(),                // ROM?
            _ => panic!(format!("CPU bus: unknown address {}", address)),
        }
    }


    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000...0x07FF => self.ram.write(address as usize, value), // CPU RAM
            0x0800...0x1FFF => unimplemented!(), // CPU RAM (mirror)
            0x2000...0x2007 => unimplemented!(), // PPU registers
            0x2008...0x3FFF => unimplemented!(), // PPU registers (mirror)
            0x4000...0x401F => unimplemented!(), // Other IO devices?
            0x6000...0xFFFF => unimplemented!(), // ROM?
            _ => panic!(format!("CPU bus: unknown address {}", address)),
        }
    }
}