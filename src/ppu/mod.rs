pub mod bus;
pub mod display;
pub mod palette;

const scanlines: u16 = 262;
const scanlines_in_vblank: u16 = 20;
const ppu_cycles_per_scanline: u16 = 341;
const pixels_per_scanline: u16 = 256;

use crate::cartridge::Cartridge;
use std::cell::RefCell;
use crate::ppu::bus::Bus;
use crate::ppu::display::Display;
use crate::ppu::palette::Palette;


pub struct Ppu {
    pub ppuaddr: u8,
    pub ppuctrl: u8,
    pub ppudata: u8,
    pub ppumask: u8,
    pub ppustatus: u8,
    pub ppuscroll: u8,
    pub oamaddr: u8,
    pub oamdata: u8,
    oamdma: u8,
    scanline: i16,
    cycle: u16,
    pub bus: Bus,
    pub display: Display,
    common_latch: u8,
    palette: Palette
}


impl Ppu {
    pub fn new(bus: Bus) -> Ppu {
        Ppu {
            ppuaddr: 0,
            ppuctrl: 0,
            ppudata: 0,
            ppumask: 0,
            ppustatus: 0xA0,
            ppuscroll: 0,
            oamaddr: 0,
            oamdata: 0,
            oamdma: 0,
            scanline: 0,
            cycle: 0,
            common_latch: 0,
            bus,
            display: Default::default(),
            palette: Palette::new()
        }
    }

    pub fn step(&mut self) {
        self.visible_scanline();
    }

    pub fn cycle(&mut self) {
        match self.scanline {
            0...239 => self.visible_scanline(), // Visible scanlines 
            _ => ()
        }
    }

    pub fn visible_scanline(&mut self) {
        let nametable_number = self.ppuctrl & 0x03;
        let nametable_address_start = 0x2000 + nametable_number as u16 * 0x400; 

        for y in 0..255 {
            for x in 0..239 {
                let cell_row = y / 8;
                let cell_col = x / 8;

                let nt_entry = self.bus.read(nametable_address_start + cell_col + 32*cell_row);
                let att_entry = self.bus.read(nametable_address_start + 0x03C0 + cell_row / 2 + cell_row / 2 * 8);

                let pattern_table_0_address = ((nt_entry as u16) << 4) + y%8;

                let pt_low  = self.bus.read(pattern_table_0_address);
                let pt_high = self.bus.read(pattern_table_0_address + 8);

                let a = x % 8;

                let low  = (pt_low  >> (7-x)) & 1;
                let high = (pt_high >> (7-x)) & 1;
                let color_number = (high << 1) & low;
                
                let palette_shift = match ((x % 32) / 16, (y % 32) / 16) {
                    (0, 0) => 0,
                    (1, 0) => 2,
                    (0, 1) => 4,
                    (1, 1) => 6,
                    _ => panic!()
                };

                let palette_number = (att_entry >> palette_shift) & 0x03;
                let color = self.palette.get_color(palette_number as usize).unwrap();
                self.display.set_pixel(x as usize, y as usize, (*color).clone());                
            }
        }
        


        match self.cycle {
            0 => (),
            1...257 => {
                match ((self.cycle-1) % 8) + 1  {
                    1...2 => self.fetch_nametable_byte(),
                    3...4 => (),
                    5...6 => (),
                    7...8 => (),
                    _ => panic!("oh no")
                }
            },
            258...320 => (),
            321...340 => (),
            _ => panic!("PPU cycle count {} exceeds 340", self.cycle)

        }
    }

    fn fetch_nametable_byte(&mut self) {
        let nametable_number = self.ppuctrl & 0x03;
        let nametable_address_start = 0x2000 + nametable_number as u16 * 0x400; 
    }

    fn fetch_attribute_table_byte(&mut self) {
        
    }

    fn fetch_low_bg_tile_byte(&mut self) {
        
    }

    fn fetch_high_bg_tile_byte(&mut self) {
        
    }

}
