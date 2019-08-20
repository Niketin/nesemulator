pub mod bus;

const scanlines: u16 = 262;
const scanlines_in_vblank: u16 = 20;

use crate::ppu::bus::Bus;


pub struct Ppu<'a> {
    pub ppuaddr: u8,
    pub ppuctrl: u8,
    pub ppudata: u8,
    pub ppumask: u8,
    pub ppustatus: u8,
    pub ppuscroll: u8,
    pub oamaddr: u8,
    pub oamdata: u8,
    oamdma: u8,
    pub bus: Bus<'a>
}


impl<'a> Ppu<'a> {
    pub fn new(bus: Bus<'a>) -> Ppu<'a> {
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
            bus
        }
    }

    pub fn cycle(&mut self) {
        unimplemented!();
    }
}