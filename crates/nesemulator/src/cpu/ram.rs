pub struct Ram {
    pub size: usize,
    mem: Vec<u8>,
}

impl Ram {
    pub fn new(size: usize) -> Ram {
        Ram {
            size,
            mem: vec![0; size],
        }
    }

    pub fn read(&self, address: usize) -> u8 {
        self.mem[address]
    }

    pub fn write(&mut self, address: usize, value: u8) {
        self.mem[address] = value;
    }
}

impl Default for Ram {
    fn default() -> Ram {
        Ram::new(0x0800)
    }
}