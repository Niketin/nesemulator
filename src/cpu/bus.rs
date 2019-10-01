use crate::cpu::ram::Ram;
use crate::cartridge::Cartridge;
use crate::ppu::Ppu;

use std::cell::RefCell;
use std::rc::Rc;

pub struct Bus {
    ram: Ram,
    cartridge: Rc<RefCell<Cartridge>>,
    pub ppu: Option<Ppu>
}

impl Bus {
    pub fn new(ram: Ram, cartridge: Rc<RefCell<Cartridge>>) -> Bus {
        let size = ram.size;
        if size != 0x0800 {
            panic!("Creating a new Bus: CPU RAM does not have correct size (0x0800)");
        }
        Bus { ram, cartridge, ppu: None }
    }

    pub fn set_ppu(&mut self, ppu: Ppu) {
        self.ppu = Some(ppu);
    }

    pub fn read(&self, address: u16) -> u8 {
        let cartridge = self.cartridge.borrow();
        match address {
            0x0000...0x07FF => self.ram.read(address as usize), // CPU RAM
            0x0800...0x1FFF => self.ram.read((address % 0x0800) as usize), // CPU RAM (mirror)
            0x2000...0x2007 => self.read_ppu_register(address), // PPU registers
            0x2008...0x3FFF => self.read_ppu_register((address - 0x2008u16) % 0x0008u16 + 0x2000u16), // PPU registers (mirror)
            0x4000...0x401F => unimplemented!(), // NES APU and I/O registers
            0x6000...0xFFFF => cartridge.read_using_cpu_bus_address(address as usize), // Cartridge (PRG ROM, PRG RAM, and mapper)
            _ => panic!(format!("CPU bus: unknown address {}", address)),
        }
    }



    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000...0x07FF => self.ram.write(address as usize, value),            // CPU RAM
            0x0800...0x1FFF => self.ram.write((address % 0x0800) as usize, value), // CPU RAM (mirror)
            0x2000...0x2007 => self.write_ppu_register(address, value),            // PPU registers
            0x2008...0x3FFF => self.write_ppu_register((address - 0x2008u16) % 0x0008u16 + 0x2000u16, value),                                   // PPU registers (mirror)
            0x4000...0x401F => unimplemented!(),                                   // NES APU and I/O registers
            0x6000...0xFFFF => unimplemented!(),                                   // Cartridge (PRG ROM, PRG RAM, and mapper)
            _ => panic!(format!("CPU bus: unknown address {}", address)),
        }
    }

    fn read_ppu_register(&self, address: u16) -> u8 {
        let register = match &self.ppu {
            None => panic!(""),
            Some(ppu) =>
                match address {
                    0x2000 => &ppu.ppuctrl,
                    0x2001 => &ppu.ppumask,
                    0x2002 => &ppu.ppustatus,
                    0x2003 => &ppu.oamaddr,
                    0x2004 => &ppu.oamdata,
                    0x2005 => &ppu.ppuscroll,
                    0x2006 => &ppu.ppuaddr,
                    0x2007 => &ppu.ppudata,
                    _ => panic!(format!("CPU bus: unknown PPU register {}", address))
                }
        };
        *register
    }

    fn write_ppu_register(&mut self, address: u16, value: u8) {
        let register = match &mut self.ppu {
            None => panic!(""),
            Some(ppu) =>
                match address {
                    0x2000 => &mut ppu.ppuctrl,
                    0x2001 => &mut ppu.ppumask,
                    0x2002 => &mut ppu.ppustatus,
                    0x2003 => &mut ppu.oamaddr,
                    0x2004 => &mut ppu.oamdata,
                    0x2005 => &mut ppu.ppuscroll,
                    0x2006 => &mut ppu.ppuaddr,
                    0x2007 => &mut ppu.ppudata,
                    _ => panic!(format!("CPU bus: unknown PPU register {}", address))
                }
        };
        *register = value;
    }
}