use crate::cpu::ram::Ram;
use crate::cartridge::Cartridge;

pub struct Bus<'a> {
    vram: Ram,
    cartridge: &'a Cartridge
}

impl<'a> Bus<'a> {
    pub fn new(vram: Ram, cartridge: &'a Cartridge) -> Bus<'a> {
        let size = vram.size;
        if size != 0x0800 {
            panic!("Creating a new Bus: PPU RAM does not have correct size (0x0800)");
        }
        Bus { vram, cartridge }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            _ => panic!(format!("PPU bus: unknown address {}", address)),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            _ => panic!(format!("PPU bus: unknown address {}", address)),
        }
    }
}