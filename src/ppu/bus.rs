use crate::cpu::ram::Ram;
use crate::cartridge::Cartridge;

use std::cell::RefCell;
use std::rc::Rc;


pub struct Bus {
    vram: Ram,
    cartridge: Rc<RefCell<Cartridge>>
}

impl Bus {
    pub fn new(vram: Ram, cartridge: Rc<RefCell<Cartridge>>) -> Bus {
        let size = vram.size;
        if size != 0x0800 {
            panic!("Creating a new Bus: PPU RAM does not have correct size (0x0800)");
        }
        Bus { vram, cartridge }
    }
    
    pub fn read(&self, address: u16) -> u8 {
        let cartridge = self.cartridge.borrow();
        match address { // TODO: move this (or at least 0x0000-0x2FFF) logic inside cartridge or mappers
            0x0000...0x0FFF => cartridge.read_from_pattern_table(address),         // Pattern table 0
            0x1000...0x1FFF => cartridge.read_from_pattern_table(address),// Pattern table 1
            0x2000...0x23FF => cartridge.read_from_nametable(address, &self.vram),// Nametable 0
            0x2400...0x27FF => cartridge.read_from_nametable(address, &self.vram),// Nametable 1
            0x2800...0x2BFF => cartridge.read_from_nametable(address, &self.vram),// Nametable 2
            0x2C00...0x2FFF => cartridge.read_from_nametable(address, &self.vram),// Nametable 3
            0x3000...0x3EFF => unimplemented!(),// Mirrors of $2000-$2EFF
            0x3F00...0x3F1F => unimplemented!(),// Palette RAM indexes
            0x3F20...0x3FFF => unimplemented!(),//
            _ => panic!("PPU bus: unknown address {}", address),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            _ => panic!(format!("PPU bus: unknown address {}", address)),
        }
    }
}