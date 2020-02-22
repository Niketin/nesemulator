pub mod bus;
pub mod display;
pub mod palette;


use crate::ppu::bus::Bus;
use crate::ppu::display::Display;
use crate::ppu::display::Color;
use crate::ppu::palette::Palette;


pub struct Ppu {
    pub ppuaddr: u16,
    pub ppuctrl: u8,
    pub ppudata: u8,
    pub ppumask: u8,
    pub ppustatus: u8,
    pub ppuscroll: u8,
    pub oamaddr: u8,
    pub oamdata: u8,
    _oamdma: u8,
    pub scanline: i16,
    pub cycle: u16,
    pub bus: Bus,
    pub display: Display,
    _palette: Palette,
    pub nmi_occurred: bool,
    pub nmi_output: bool,
    pub ppuaddr_upper_byte_next: bool
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
            _oamdma: 0,
            scanline: 0,
            cycle: 0,
            bus,
            display: Default::default(),
            _palette: Palette::new(),
            nmi_occurred: false,
            nmi_output: true,
            ppuaddr_upper_byte_next: true
        }
    }

    pub fn write_ppuaddr(&mut self, value: u8) {
        if self.ppuaddr_upper_byte_next {
            self.ppuaddr = (value as u16) << 8;
            self.ppuaddr_upper_byte_next = false;
        }
        else {
            self.ppuaddr |= value as u16;
            self.ppuaddr_upper_byte_next = true;
        }
    }

    pub fn read_ppustatus(&mut self) -> u8 {
        let status = self.ppustatus;
        self.ppustatus &= 0x7F;   // TODO: 7th bit
        self.nmi_occurred = false;// and this variable are kinda the same. 
        self.ppuaddr_upper_byte_next = true;
        self.ppuaddr = 0;
        status
    }

    pub fn read_ppudata(&mut self) -> u8 {
        let result = self.bus.read(self.ppuaddr);
        self.ppuaddr += self.get_vram_address_increment();
        result
    }

    pub fn write_ppudata(&mut self, value: u8) {
        self.bus.write(self.ppuaddr, value);
        self.ppuaddr += self.get_vram_address_increment();
    }

    pub fn write_ppuctrl(&mut self, value: u8) {
        self.nmi_output = (value & 0x80) == 0x80;
        self.ppuctrl = value;
    }

    fn get_vram_address_increment(&self) -> u16 {
        if self.ppuctrl & 0x04 == 0x04 { 32 } else { 1 }
    }

    pub fn nmi(&mut self) -> bool {
        // TODO: set some flag off? Is it needed?
        self.nmi_occurred
    }

    pub fn step(&mut self) {
        self.cycle();
    }

    pub fn cycle(&mut self) {
        match self.scanline {
            0 if self.cycle == 0 => self.visible_scanline(), // Visible scanlines
            1..=239 => (),
            240 => (), // Post-render scanline
            241..=260 => self.vertical_blanking_lines(), // Vertical blanking lines
            261 => self.vertical_blanking_lines(),
            _ => ()
        }

        self.cycle = (self.cycle + 1) % 341; 
        if self.cycle == 0 {
            self.scanline = (self.scanline + 1) % 262;
        }
    }

    fn vertical_blanking_lines(&mut self) {
        let vblank_start = self.scanline == 241  && self.cycle == 1;
        let vblank_end = self.scanline == 261  && self.cycle == 1;

        if vblank_start {
            self.nmi_occurred = true;
            self.ppustatus |= 0x80; // set 7th bit (vblank) to 1
        }
        if vblank_end {
            self.nmi_occurred = false;
            self.ppustatus &= 0x7F; // set 7th bit (vblank) to 0
        }
    }

    pub fn visible_scanline(&mut self) {
        let nametable_number = self.ppuctrl & 0x03;
        let nametable_address_start = 0x2000 + nametable_number as u16 * 0x400; 

        for x in 0..255 {
            for y in 0..239 {
                let cell_row = y / 8;
                let cell_col = x / 8;

                let nt_entry = self.bus.read(nametable_address_start + cell_col + 32*cell_row);
                let att_entry = self.bus.read(nametable_address_start + 0x03C0 + cell_row / 2 + cell_row / 2 * 8);
                if nt_entry != 0 || att_entry != 0 {
                    println!("jee");
                }
                let pattern_table_0_address = nametable_address_start + ((nt_entry as u16) << 4) + y%8;

                let pt_low  = self.bus.read(pattern_table_0_address);
                let pt_high = self.bus.read(pattern_table_0_address + 8);

                let a = x % 8;

                let low  = (pt_low  >> (7-a)) & 1;
                let high = (pt_high >> (7-a)) & 1;

                let color_number = (high << 1) & low;
                /*
                
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
                */
                let mut palette: Vec<Color> = vec![];
                for i in 0..3 {
                    palette.push(Color::new_rgb(255/4*i, 255/4*i, 255/4*i));
                }
                self.display.set_pixel(x as usize, y as usize, palette[color_number as usize].clone());                
            }
        }
        


        match self.cycle {
            0 => (),
            1..=257 => {
                match ((self.cycle-1) % 8) + 1  {
                    1..=2 => self.fetch_nametable_byte(),
                    3..=4 => (),
                    5..=6 => (),
                    7..=8 => (),
                    _ => panic!("oh no")
                }
            },
            258..=320 => (),
            321..=340 => (),
            _ => panic!("PPU cycle count {} exceeds 340", self.cycle)

        }
    }

    fn fetch_nametable_byte(&mut self) {
        let nametable_number = self.ppuctrl & 0x03;
        let _nametable_address_start = 0x2000 + nametable_number as u16 * 0x400; 
    }

    fn _fetch_attribute_table_byte(&mut self) {
        
    }

    fn _fetch_low_bg_tile_byte(&mut self) {
        
    }

    fn _fetch_high_bg_tile_byte(&mut self) {
        
    }

}
