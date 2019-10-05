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
            _ => panic!("PPU bus: unknown address {:#x}", address),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        let mirrored_address = address % 0x4000;
        match mirrored_address {
            0x0000...0x1FFF => (), // Writing to rom does basicly nothing
            0x2000...0x2FFF => self.write_name_table(mirrored_address, value),
            // TODO: check these addresses and mirroring from documentations related to PPUDATA
            0x3000...0x3EFF => self.write_name_table(mirrored_address - 0x1000, value),
            0x3F00...0x3FFF => self.write_to_palette_ram(mirrored_address, value),
            _ => panic!("PPU bus: unknown address {:#x}", address),
        }
    }

    fn write_to_palette_ram(&mut self, address: u16, value: u8) {
        // TODO: implement this
    }

    fn write_name_table(&mut self, address: u16, value: u8) {
        let cartridge = self.cartridge.borrow();

        match address {
            0x2000...0x23FF => cartridge.write_to_nametable(address, &mut self.vram, value),// Nametable 0
            0x2400...0x27FF => cartridge.write_to_nametable(address, &mut self.vram, value),// Nametable 1
            0x2800...0x2BFF => cartridge.write_to_nametable(address, &mut self.vram, value),// Nametable 2
            0x2C00...0x2FFF => cartridge.write_to_nametable(address, &mut self.vram, value),// Nametable 3
            _ => panic!(format!("PPU bus: should be called with address of range 0x2000...0x2FFF. Was called with {:#x}", address)),
        }
    }

    pub fn write_to_vram(&mut self, address: u16, value: u8) {
        self.vram.write(address as usize, value);
    }

     pub fn read_from_vram(&mut self, address: u16) -> u8 {
        self.vram.read(address as usize)
    }

    
}