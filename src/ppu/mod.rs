pub mod bus;
pub mod display;
pub mod palette;

use crate::ppu::bus::Bus;
use crate::ppu::display::Color;
use crate::ppu::display::Display;
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
    pub x: u16,
    pub y: u16,
    pub bus: Bus,
    pub display: Display,
    _palette: Palette,
    pub nmi_occurred: bool,
    pub nmi_output: bool,
    pub ppuaddr_upper_byte_next: bool,
    shift_nametable: u16,
    shift_att_table: u16,
    shift_palette_0: u8,
    shift_palette_1: u8,
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
            _palette: Palette::new(),
            nmi_occurred: false,
            nmi_output: true,
            ppuaddr_upper_byte_next: true,
            shift_nametable: 0,
            shift_att_table: 0,
            shift_palette_0: 0,
            shift_palette_1: 0,
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
            261 => self.vertical_blanking_lines(),       // TODO Pre-render line
            _ => (),
        }

        self.increase_x();
        if self.x == 0 {
            self.increase_y();
        }
    }

    fn increase_y(&mut self) {
        self.y = self.next_y();
    }

    fn increase_x(&mut self) {
        self.x = self.next_x();
    }

    fn next_x(&self) -> u16 {
        (self.x + 1) % 341
    }

    fn next_y(&self) -> u16 {
        (self.y + 1) % 262
    }

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
        self.shift_att_table = self.shift_att_table.rotate_left(8);
        self.shift_nametable = self.shift_nametable.rotate_left(8);
        self.shift_palette_0 = self.shift_palette_0.rotate_left(4);
        self.shift_palette_1 = self.shift_palette_1.rotate_left(4);
    }

    pub fn visible_scanline(&mut self) {
        match self.x {
            0 => (), // TODO: is this ok?
            1..=256 => self.fetch_match(),
            257 => (),
            258..=320 => (),
            321..=336 => self.fetch_match(),
            337..=340 => (),
            _ => unreachable!("PPU cycle count {} exceeds 340", self.y),
        }

        if 1 <= self.x && self.x <= 256 {
            let nt_entry = (self.shift_nametable & 0x00FF) as u8;
            let pattern_table_address: u16 = ((self.ppuctrl as u16 >> 4) & 1) * 0x1000;
            let pattern_fine_y_offset = self.y % 8;
            let pattern_fine_x_offset = self.x % 8;
            let pattern_table_tile_row_address =
                pattern_table_address | ((nt_entry as u16) << 4) | pattern_fine_y_offset;

            let pt_low = self.bus.read(pattern_table_tile_row_address & 0xfff7);
            let pt_high = self.bus.read(pattern_table_tile_row_address | 0x8);

            let low = (pt_low >> (7 - pattern_fine_x_offset)) & 1;
            let high = (pt_high >> (7 - pattern_fine_x_offset)) & 1;
            let color_number = (high << 1) | low;

            let mut palette: Vec<Color> = vec![];
            for i in 0..=3 {
                let c = 255 / 3 * i;
                palette.push(Color::new_rgb(c, c, c));
            }
            self.display.set_pixel(
                (self.x - 1) as usize,
                self.y as usize,
                palette[color_number as usize].clone(),
            );
        }
    }

    fn fetch_nametable_byte(&mut self) {
        let nametable_number = self.ppuctrl as u16 & 0x03;
        let nametable_address_start = 0x2000 + nametable_number * 0x400;
        let cell_x = self.x / 8;
        let cell_y = self.y / 8;
        let address = nametable_address_start + cell_x + 32 * cell_y;
        let nt_entry = self.bus.read(address);
        self.shift_nametable &= 0x00ff;
        self.shift_nametable |= (nt_entry as u16) << 8;
    }

    fn fetch_attribute_table_byte(&mut self) {
        let nametable_number = self.ppuctrl & 0x03;
        let nametable_address_start = 0x2000 + nametable_number as u16 * 0x400;
        let cell_col = self.x / 32;
        let cell_row = self.y / 32;
        let address = nametable_address_start + 0x03C0 + cell_row + cell_col * 8;
        let att_entry = self.bus.read(address);
        self.shift_att_table &= 0x00ff;
        self.shift_att_table |= (att_entry as u16) << 8;
    }

    fn fetch_low_bg_tile_byte(&mut self) {}

    fn fetch_high_bg_tile_byte(&mut self) {}
}
