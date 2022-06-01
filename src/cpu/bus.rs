use crate::cpu::ram::Ram;
use crate::cartridge::Cartridge;
use crate::ppu::Ppu;
use crate::controller::Controller;

use std::cell::RefCell;
use std::rc::Rc;

pub struct Bus {
    ram: Ram,
    cartridge: Rc<RefCell<Cartridge>>,
    pub ppu: Option<Ppu>,
    pub oamdma_occurred: bool,
    pub oamdma_high_byte: u16,
    pub controller: Option<Controller>,
}

impl Bus {
    pub fn new(ram: Ram, cartridge: Rc<RefCell<Cartridge>>) -> Bus {
        let size = ram.size;
        if size != 0x0800 {
            panic!("Creating a new Bus: CPU RAM does not have correct size (0x0800)");
        }
        Bus { ram, cartridge, ppu: None, controller: None, oamdma_occurred: false, oamdma_high_byte: 0 }
    }

    pub fn set_ppu(&mut self, ppu: Ppu) {
        self.ppu = Some(ppu);
    }

    pub fn set_controller(&mut self, controller: Controller) {
        self.controller = Some(controller);
    }

    pub fn read(&mut self, address: u16) -> u8 {
        match address {
            0x0000..=0x07FF => self.ram.read(address as usize), // CPU RAM
            0x0800..=0x1FFF => self.ram.read((address % 0x0800) as usize), // CPU RAM (mirror)
            0x2000..=0x2007 => self.read_ppu_register(address), // PPU registers
            0x2008..=0x3FFF => self.read_ppu_register((address - 0x2008u16) % 0x0008u16 + 0x2000u16), // PPU registers (mirror)
            0x4000..=0x401F => self.read_apu_io_registers(address), // NES APU and I/O registers
            0x6000..=0xFFFF => self.cartridge.borrow().read_using_cpu_bus_address(address as usize), // Cartridge (PRG ROM, PRG RAM, and mapper)
            _ => panic!("CPU bus: unknown address {}", address),
        }
    }



    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x07FF => self.ram.write(address as usize, value), // CPU RAM
            0x0800..=0x1FFF => self.ram.write((address % 0x0800) as usize, value), // CPU RAM (mirror)
            0x2000..=0x2007 => self.write_ppu_register(address, value), // PPU registers
            0x2008..=0x3FFF => self.write_ppu_register((address - 0x2008u16) % 0x0008u16 + 0x2000u16, value), // PPU registers (mirror)
            0x4000..=0x401F => self.write_apu_io_registers(address, value), // NES APU and I/O registers
            0x6000..=0xFFFF => unimplemented!(), // Cartridge (PRG ROM, PRG RAM, and mapper)
            _ => panic!("CPU bus: unknown address {}", address),
        }
    }

    fn read_ppu_register(&mut self, address: u16) -> u8 {
        let ppu: &mut Ppu = self.ppu.as_mut().unwrap();
        match address {
            0x2000 => ppu.ppuctrl,
            0x2001 => ppu.ppumask,
            0x2002 => ppu.read_ppustatus(),
            0x2003 => ppu.oamaddr,
            0x2004 => ppu.oamdata,
            0x2005 => ppu.ppuscroll,
            0x2006 => panic!("CPU bus: reading from write-only PPU register {} aka PPUADDR.", address),
            0x2007 => ppu.read_ppudata(),
            _ => panic!("CPU bus: unknown PPU register {}", address)
        }
    }

    fn write_ppu_register(&mut self, address: u16, value: u8) {
        let ppu = self.ppu.as_mut().unwrap();

        match address {
            0x2000 => ppu.write_ppuctrl(value),
            0x2001 => ppu.ppumask = value,
            0x2002 => ppu.ppustatus = value,
            0x2003 => ppu.oamaddr = value,
            0x2004 => ppu.oamdata = value,
            0x2005 => ppu.ppuscroll = value,
            0x2006 => ppu.write_ppuaddr(value),
            0x2007 => ppu.write_ppudata(value),
            _ => panic!("CPU bus: unknown PPU register {}", address)
        };
    }

    fn read_apu_io_registers(&mut self, address: u16) -> u8 {
        debug_assert!((0x4000..=0x401f).contains(&address));
        match address {
            0x4000..=0x4015 => 0,
            0x4016 => {
                self.controller.as_mut().map_or(1, |c| c.read()) },
            0x4017..=0x401f => 0,
            _ => unreachable!()
        }
    }

    fn write_apu_io_registers(&mut self, address: u16, value: u8) {
        debug_assert!((0x4000..=0x401f).contains(&address)); // TODO add this kind of assert to other similiar functions that uses match.
        match address {
            0x4000..=0x4013 => (), // TODO Implement
            0x4014 => self.oamdma(value),
            0x4015 => (), // TODO Implement
            0x4016 => { if let Some(c) = self.controller.as_mut() { c.write(value)} },
            0x4017..=0x401f => (), // TODO Implement
            _ => unreachable!()
        }
    }

    fn oamdma(&mut self, value: u8) {
        self.oamdma_occurred = true;
        self.oamdma_high_byte = (value as u16) << 8;
    }
}