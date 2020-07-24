use crate::cartridge::Cartridge;
use crate::cpu::ram::Ram;

use std::cell::RefCell;
use std::rc::Rc;

pub struct Bus {
    vram: Ram,
    palette_ram: Ram,
    cartridge: Rc<RefCell<Cartridge>>,
}

impl Bus {
    pub fn new(cartridge: Rc<RefCell<Cartridge>>) -> Bus {
        Bus {
            vram: Ram::default(),
            palette_ram: Ram::new(0x20),
            cartridge,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        let cartridge = self.cartridge.borrow();
        match address {
            // TODO: move this (or at least 0x0000-0x2FFF) logic inside cartridge or mappers
            0x0000..=0x1FFF => cartridge.read_from_pattern_table(address), // Pattern table 0..1
            0x2000..=0x2FFF => cartridge.read_from_nametable(address, &self.vram), // Nametable 0..3
            0x3000..=0x3EFF => unimplemented!(), // Mirrors of $2000-$2EFF
            0x3F00..=0x3F1F => self.read_from_palette_ram(address - 0x3F00), // Palette RAM indexes
            0x3F20..=0x3FFF => unimplemented!(), // TODO ??
            _ => panic!("PPU bus: unknown address {:#x}", address),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => (), // Writing to rom does basicly nothing
            0x2000..=0x2FFF => self.write_name_table(address, value),
            // TODO: check these addresses and mirroring from documentations related to PPUDATA
            0x3000..=0x3EFF => self.write_name_table(address - 0x1000, value),
            0x3F00..=0x3FFF => self.write_to_palette_ram(address - 0x3F00, value),
            _ => panic!("PPU bus: unknown address {:#x}", address),
        }
    }

    fn write_to_palette_ram(&mut self, address: u16, value: u8) {
        println!("Write to palette address {}", address);
        self.palette_ram.write(address as usize % 0x20, value)
    }
    
    fn read_from_palette_ram(& self, address: u16) -> u8 {
        self.palette_ram.read(address as usize % 0x20)
    }

    fn write_name_table(&mut self, address: u16, value: u8) {
        let cartridge = self.cartridge.borrow();
        match address {
            0x2000..=0x2FFF => cartridge.write_to_nametable(address, &mut self.vram, value), // Nametable 0..3
            _ => panic!(format!("PPU bus: should be called with address of range 0x2000..=0x2FFF. Was called with {:#x}", address)),
        }
    }

    pub fn write_to_vram(&mut self, address: u16, value: u8) {
        self.vram.write(address as usize, value);
    }

    pub fn read_from_vram(&mut self, address: u16) -> u8 {
        self.vram.read(address as usize)
    }
}
