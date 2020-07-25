pub mod bus;
pub mod display;
pub mod palette;
mod shift_register;

use crate::ppu::bus::Bus;
use crate::ppu::display::Color;
use crate::ppu::display::Display;
use crate::ppu::palette::Palette;
use crate::ppu::shift_register::ShiftRegister;

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
    pub x: u16,
    pub y: u16,
    pub bus: Bus,
    pub display: Display,
    palette: Palette,
    pub nmi_occurred: bool,
    pub nmi_output: bool,
    pub ppuaddr_upper_byte_next: bool,
    shift_nametable: ShiftRegister<u8>, // u16
    shift_att_table: ShiftRegister<u8>, // u16
    shift_palette_0: ShiftRegister<u8>, // u8
    shift_palette_1: ShiftRegister<u8>, // u8
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
            x: 0,
            y: 0,
            bus,
            display: Default::default(),
            palette: Palette::new(),
            nmi_occurred: false,
            nmi_output: true,
            ppuaddr_upper_byte_next: true,
            shift_nametable: Default::default(),
            shift_att_table: Default::default(),
            shift_palette_0: Default::default(),
            shift_palette_1: Default::default(),
        }
    }

    pub fn write_ppuaddr(&mut self, value: u8) {
        if self.ppuaddr_upper_byte_next {
            self.ppuaddr = (value as u16) << 8;
            self.ppuaddr_upper_byte_next = false;
        } else {
            self.ppuaddr |= value as u16;
            self.ppuaddr_upper_byte_next = true;
        }
    }

    pub fn read_ppustatus(&mut self) -> u8 {
        let status = self.ppustatus;
        self.clear_vblank();
        self.nmi_occurred = false; // and this variable are kinda the same.
        self.ppuaddr_upper_byte_next = true;
        self.ppuaddr = 0;
        status
    }

    fn clear_vblank(&mut self) {
        self.ppustatus &= 0x7F;
    }

    fn set_vblank(&mut self) {
        self.ppustatus |= 0x80;
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
        if self.ppuctrl & 0x04 == 0x04 {
            32
        } else {
            1
        }
    }

    pub fn nmi(&mut self) -> bool {
        // TODO: set some flag off? Is it needed?
        let nmi_occurred = self.nmi_occurred;
        self.nmi_occurred = false;
        nmi_occurred
    }

    pub fn step(&mut self) {
        self.cycle();
    }

    pub fn cycle(&mut self) {
        if self.y == 241 && self.x == 1 {
            self.set_vblank();
        }

        if self.y == 261 && self.x == 1 {
            self.clear_vblank();
            // TODO: clear sprite 0 and overlow according to the timing chart.
        }

        match self.y {
            0..=239 => self.visible_scanline(),          // Visible scanlines
            240 => (),                                   // Post-render scanline
            241..=260 => self.vertical_blanking_lines(), // TODO Vertical blanking lines
            261 => {self.fetch_stuff();
                    self.vertical_blanking_lines();},
            _ => (),
        }

        self.increase_x();
        if self.x == 0 {
            self.increase_y();
        }
    }

    fn increase_y(&mut self) { self.y = self.next_y(); }
    fn increase_x(&mut self) { self.x = self.next_x(); }

    fn next_x(&self) -> u16 { self.mod_x(self.x + 1) }
    fn next_y(&self) -> u16 { self.mod_y(self.y + 1) }

    fn mod_x(&self, value: u16) -> u16 { value % 341 }
    fn mod_y(&self, value: u16) -> u16 { value % 262 }

    fn vertical_blanking_lines(&mut self) {
        let vblank_start = self.y == 241 && self.x == 1;
        let vblank_end = self.y == 261 && self.x == 1;

        if vblank_start {
            self.nmi_occurred = true;
            self.ppustatus |= 0x80; // set 7th bit (vblank) to 1
        }
        if vblank_end {
            self.nmi_occurred = false;
            self.ppustatus &= 0x7F; // set 7th bit (vblank) to 0
        }
    }

    fn fetch_match(&mut self) {
        match ((self.x - 1) % 8) + 1 {
            1 => (),
            2 => self.fetch_nametable_byte(),
            3 => (),
            4 => self.fetch_attribute_table_byte(),
            5 => (),
            6 => self.fetch_low_bg_tile_byte(),
            7 => (),
            8 => {
                self.fetch_high_bg_tile_byte();
                self.shift_registers();
            }
            _ => unreachable!(),
        }
    }

    fn shift_registers(&mut self) {
        self.shift_att_table.shift();
        self.shift_nametable.shift();
        self.shift_palette_0.shift();
        self.shift_palette_1.shift();
    }

    fn fetch_stuff(&mut self) {
        match self.x {
            0 => (), // TODO: is this ok?
            1..=256 => self.fetch_match(),
            257 => (),
            258..=320 => (),
            321..=336 => self.fetch_match(),
            337..=340 => (),
            _ => unreachable!("PPU cycle count {} exceeds 340", self.y),
        }
    }

    pub fn visible_scanline(&mut self) {
        self.fetch_stuff();

        if 1 <= self.x && self.x <= 256 {
            let att_entry = self.shift_att_table.get();
            let nt_entry = self.shift_nametable.get();
            let pattern_table_address: u16 = ((self.ppuctrl as u16 >> 4) & 1) * 0x1000;
            let pattern_fine_y_offset = self.y % 8;
            let pattern_fine_x_offset = self.x % 8;
            let pattern_table_tile_row_address =
                pattern_table_address | ((nt_entry as u16) << 4) | pattern_fine_y_offset;

            let pt_low = self.bus.read(pattern_table_tile_row_address);
            let pt_high = self.bus.read(pattern_table_tile_row_address | 0x8);

            let low = (pt_low >> (7 - pattern_fine_x_offset)) & 1;
            let high = (pt_high >> (7 - pattern_fine_x_offset)) & 1;
            let color_number = (high << 1) | low;
            
            let palette_shift: u8 = match ((self.x % 32) / 16, (self.y / 32) / 16) {
                (0, 0) => 0, // top left
                (1, 0) => 2, // top right
                (0, 1) => 4, // bottom left
                (1, 1) => 6, // bottom right
                _ => unreachable!()
            };

            let palette_number = (att_entry >> palette_shift) & 0x03;

            let color_address: u16 = ((palette_number as u16) << 2) | color_number as u16;
            
            let color_number_in_big_palette = self.bus.read(0x3F00 + color_address as u16);
            let color = self.palette.get_color(color_number_in_big_palette as usize).unwrap();
            

            //self.display.set_pixel(x as usize, y as usize, (*color).clone());
            
            /*let mut palette: Vec<Color> = vec![];
            for i in 0..=3 {
                let c = 255 / 3 * i;
                palette.push(Color::new_rgb(c, c, c));
            }*/
            self.display.set_pixel(
                (self.x - 1) as usize,
                self.y as usize,
                color.clone(),
            );
        }
    }

    fn fetch_nametable_byte(&mut self) {
        let nametable_address = self.nametable_address();
        let (x, y) = self.get_next_tile_xy();
        let cell_x = x / 8;
        let cell_y = y / 8;
        let address = nametable_address + cell_x + 32 * cell_y;
        let nt_entry = self.bus.read(address);
        self.shift_nametable.set(nt_entry);
    }

    fn fetch_attribute_table_byte(&mut self) {
        let nametable_address = self.nametable_address();
        let (x, y) = self.get_next_tile_xy();
        let cell_x = x / 32;
        let cell_y = y / 32;
        let address = nametable_address + 0x03C0 + (cell_y<<3) + cell_x;
        let att_entry = self.bus.read(address);
        self.shift_att_table.set(att_entry);
    }

    fn nametable_address(&self) -> u16 {
        let nametable_number = self.ppuctrl & 0x03;
        0x2000 + nametable_number as u16 * 0x400
    }

    fn get_next_tile_xy(&self) -> (u16, u16) {
        // x has to be at the tile after the next tile
        let x = if 321 <= self.x && self.x <= 336 {
            self.x-321
        }
        else {
            self.mod_x(self.x + 16)
        };

        // y has to be at the tile after the next tile
        let y = if x < self.x {
            self.next_y()
        } else {
            self.y
        };
        (x, y)
    }

    fn fetch_low_bg_tile_byte(&mut self) {}

    fn fetch_high_bg_tile_byte(&mut self) {}
}
