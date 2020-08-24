pub mod bus;
pub mod display;
pub mod palette;
mod shift_register;

use crate::ppu::bus::Bus;
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
    shift_att_table: ShiftRegister,
    shift_pattern_l: ShiftRegister,
    shift_pattern_h: ShiftRegister,
    latch_nametable: u8,
    latch_attribute: u8,
    latch_pattern_h: u8,
    latch_pattern_l: u8,
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
            shift_att_table: ShiftRegister::new(2),
            shift_pattern_l: ShiftRegister::new(2),
            shift_pattern_h: ShiftRegister::new(2),
            latch_nametable: 0,
            latch_attribute: 0,
            latch_pattern_h: 0,
            latch_pattern_l: 0,
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

    pub fn write_oamdma(&mut self, value: u8) {
        self.oam_primary[self.oamaddr as usize] = value;
        self.oamaddr = self.oamaddr.wrapping_add(1);
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
            1 => self.update_shift_registers_from_latches(),
            2 => self.fetch_nametable_byte(),
            3 => (),
            4 => self.fetch_attribute_table_byte(),
            5 => (),
            6 => self.fetch_low_bg_tile_byte(),
            7 => (),
            8 => self.fetch_high_bg_tile_byte(),
            _ => unreachable!(),
        }
    }

    fn update_shift_registers_from_latches(&mut self) {
        self.shift_att_table.shift_bytes();
        self.shift_att_table.set(self.latch_attribute);
        self.shift_pattern_l.set(self.latch_pattern_l);
        self.shift_pattern_h.set(self.latch_pattern_h);
    }

    fn shift_patterns(&mut self) {
        self.shift_pattern_l.shift_bits();
        self.shift_pattern_h.shift_bits();
    }

    fn fetch_stuff(&mut self) {
        match self.x {
            0 => (), // TODO: is this ok?
            1..=256 => {
                self.shift_patterns();
                self.fetch_match();
            },
            257 => (),
            258..=320 => (),
            321..=336 => {
                self.shift_patterns();
                self.fetch_match();
            },
            337..=340 => (),
            _ => unreachable!("PPU cycle count {} exceeds 340", self.y),
        }
    }

    pub fn visible_scanline(&mut self) {
        self.fetch_stuff();
        if 1 <= self.x && self.x <= 256 {
            let background_render_enabled = (self.ppumask >> 3) & 1 == 1;
            let color = if background_render_enabled {
                self.get_background_color()
            } else {
                self.palette.get_color(0).clone()
            };
            self.display.set_pixel(
                (self.x - 1) as usize,
                self.y as usize,
                color.clone(),
            );
        }
    }

    fn get_background_color(&self) -> display::Color {
        debug_assert!(1<= self.x && self.x <= 256);
        let pattern_l = self.shift_pattern_l.get() & 1;
        let pattern_h = self.shift_pattern_h.get() & 1;
        let color_number = (pattern_h << 1) | pattern_l;
        let palette_shift: u8 = ((self.x & 0x10) >> 3) as u8 | (self.y >> 7) as u8;
        let att_entry = self.shift_att_table.get();
        let palette_number = (att_entry >> palette_shift) & 0x03;
        let color_address: u16 = ((palette_number as u16) << 2) | color_number as u16;
        let color_number_in_big_palette = self.bus.read(0x3F00 + color_address as u16);
        return self.palette.get_color(color_number_in_big_palette as usize).clone();
    }

    fn fetch_nametable_byte(&mut self) {
        let nametable_address = self.nametable_address();
        let (x, y) = self.get_next_tile_xy();
        let cell_x = x >> 3;
        let cell_y = y >> 3;
        let address = nametable_address + cell_x + (cell_y << 5);
        let nt_entry = self.bus.read(address);
        self.latch_nametable = nt_entry;
    }

    fn fetch_attribute_table_byte(&mut self) {
        let nametable_address = self.nametable_address();
        let (x, y) = self.get_next_tile_xy();
        let cell_x = x >> 5;
        let cell_y = y >> 5;
        let address = nametable_address + 0x03C0 + (cell_y << 3) + cell_x;
        let att_entry = self.bus.read(address);
        self.latch_attribute = att_entry;
    }

    fn nametable_address(&self) -> u16 {
        let nametable_number = self.ppuctrl & 0x03;
        0x2000 + ((nametable_number as u16) << 10)
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

    fn get_pattern_table_tile_address(&self) -> u16 {
        let pattern_table_address: u16 = (self.ppuctrl as u16 & 0x10) << 8;
        let (_, y) = self.get_next_tile_xy();
        let pattern_fine_y_offset = y & 0b0111;
        pattern_table_address | ((self.latch_nametable as u16) << 4) | pattern_fine_y_offset
    }

    fn fetch_low_bg_tile_byte(&mut self) {
        self.latch_pattern_l = self.bus.read(self.get_pattern_table_tile_address()).reverse_bits();
    }

    fn fetch_high_bg_tile_byte(&mut self) {
        self.latch_pattern_h = self.bus.read(self.get_pattern_table_tile_address() | 0x8).reverse_bits();
    }
}
