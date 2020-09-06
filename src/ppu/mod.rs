pub mod bus;
pub mod display;
pub mod palette;
mod shift_register;

use crate::ppu::bus::Bus;
use crate::ppu::display::Display;
use crate::ppu::palette::Palette;
use crate::ppu::shift_register::ShiftRegister;

const MASK_STATUS_OVERFLOW: u8 = 0b0010_0000;
const MASK_CONTROLLER_BACKGROUND_PATTERN_TABLE_ADDRESS: u8 = 0b0001_0000;
const MASK_CONTROLLER_SPRITE_PATTERN_TABLE_ADDRESS: u8 = 0b0000_1000;
const MASK_FLIP_SPRITE_HORIZONTALLY: u8 = 0b0100_0000;
const MASK_FLIP_SPRITE_VERTICALLY: u8 = 0b1000_0000;


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
    latch_background_pattern_high: u8,
    latch_background_pattern_low: u8,
    oam_primary: [u8; 256],
    oam_primary_n: u8,
    oam_primary_m: u8,
    oam_secondary_write_lock: bool,
    oam_temp_value: u8,
    oam_secondary: [u8; 32],
    oam_secondary_n: u8,
    oam_copying_sprite: bool,
    oam_overflow_reads_left: u8,
    oam_pattern_low: [u8; 8],
    oam_pattern_high: [u8; 8],
    oam_latches: [u8; 8],
    oam_counters: [u8; 8],
    oam_sprite_fetched_y: u8,
    oam_sprite_fetched_tile_index: u8,
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
            latch_background_pattern_high: 0,
            latch_background_pattern_low: 0,
            oam_primary: [0; 256],
            oam_primary_n: 0,
            oam_primary_m: 0,
            oam_temp_value: 0,
            oam_secondary_write_lock: false,
            oam_secondary: [0; 32],
            oam_secondary_n: 0,
            oam_copying_sprite: false,
            oam_overflow_reads_left: 0,
            oam_pattern_low: [0; 8],
            oam_pattern_high: [0; 8],
            oam_latches: [0; 8],
            oam_counters: [0; 8],
            oam_sprite_fetched_y: 0,
            oam_sprite_fetched_tile_index: 0,
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
            self.ppustatus &= 0b1110_1111;
            // TODO: clear sprite 0 according to the timing chart.
        }

        match self.y {
            0..=239 => self.visible_scanline(),          // Visible scanlines
            240 => (),                                   // Post-render scanline
            241..=260 => self.vertical_blanking_lines(),
            261 => {self.fetch_stuff();                  // Pre-render scanline
                    self.vertical_blanking_lines();},
            _ => unreachable!(),
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
            6 => self.load_low_background_tile_byte_latch(),
            7 => (),
            8 => self.load_high_background_tile_byte_latch(),
            _ => unreachable!(),
        }
    }

    fn update_shift_registers_from_latches(&mut self) {
        self.shift_att_table.shift_bytes();
        self.shift_att_table.set(self.latch_attribute);
        self.shift_pattern_l.set(self.latch_background_pattern_low);
        self.shift_pattern_h.set(self.latch_background_pattern_high);
    }

    fn shift_patterns(&mut self) {
        self.shift_pattern_l.shift_bits();
        self.shift_pattern_h.shift_bits();
    }

    fn fetch_stuff(&mut self) {
        // Prepare sprites
        match self.x {
            0 => (), // Idle
            1..=64 => self.clear_oam_secondary(), // Secondary OAM clear
            65..=256 => self.evaluate_sprites(), // Sprite evaluation for next scanline
            257..=320 => self.fetch_sprites(), // VRAM fetches
            _ => (),
        }

        // Fetch background stuff
        match self.x {
            0 => (), // TODO: is this ok? x: 0 y: 0 should be skipped every odd cycle?
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

    fn clear_oam_secondary(&mut self) {
        debug_assert!((1..=64).contains(&self.x));
        debug_assert!((0..=239).contains(&self.y) || self.y == 261);

        // Clear only on even cycles
        if self.x % 2 == 0 {
            self.oam_secondary[((self.x - 1) >> 1) as usize] = 0xff;
        }
    }

    /// Evaluate sprites for next scanline
    fn evaluate_sprites(&mut self) {
        debug_assert!((65..=256).contains(&self.x));
        debug_assert!((0..=239).contains(&self.y) || self.y == 261);

        let odd_cycle = self.x % 2 == 1;

        if self.x == 65 {
            // Init all oam values at first call in a scanline
            self.oam_primary_m = 0;
            self.oam_primary_n = 0;
            self.oam_secondary_n = 0;
            self.oam_secondary_write_lock = false;
            self.oam_copying_sprite = false;
            self.oam_overflow_reads_left = 0;
        }

        if self.oam_primary_n == 64 {
            // Pretend to fail copying by doing nothing
            return;
        }

        if odd_cycle {
            // Read from primary OAM
            self.oam_temp_value = self.oam_primary[(self.oam_primary_n as usize * 4 + self.oam_primary_m as usize)];
        }
        else {
            if self.oam_overflow_reads_left > 0 {
                self.oam_overflow_reads_left -= 1;
                return;
            }

            if !self.oam_secondary_write_lock && self.oam_secondary_n < 8 {
                // Write to secondary OAM
                self.oam_secondary[(self.oam_secondary_n * 4 + self.oam_primary_m) as usize] = self.oam_temp_value;
            }

            if self.oam_copying_sprite {
                // Copy remaining elements of current sprite
                self.oam_primary_m += 1;
                if self.oam_primary_m == 4 {
                    self.oam_primary_m = 0;
                    self.oam_secondary_n += 1;
                    self.oam_primary_n += 1;
                    self.oam_copying_sprite = false;
                }
                return;
            }

            if self.oam_secondary_n == 8 {
                // Secondary OAM full
                self.oam_secondary_write_lock = true;
                let sprite_y = self.oam_temp_value as u16;
                let y_in_range = (sprite_y..sprite_y+8).contains(&self.y);
                if y_in_range {
                    self.ppustatus |= MASK_STATUS_OVERFLOW;
                }
                else {
                    self.oam_primary_n += 1;
                }
                self.oam_primary_m += 1;
                if self.oam_primary_m == 4 {
                    self.oam_primary_m = 0;
                    self.oam_primary_n += 1;
                    self.oam_primary_n %= 4;
                    self.oam_overflow_reads_left = 3;
                }
                return;
            }

            let y = self.oam_temp_value as u16;
            let y_in_range = (y + 1..y+9).contains(&self.next_y());
            if y_in_range {
                self.oam_copying_sprite = true;
                self.oam_primary_m += 1;
                return;
            }
            else {
                self.oam_primary_n += 1;
            }
        }
    }

    fn fetch_sprites(&mut self) {
        debug_assert!((257..=320).contains(&self.x));
        debug_assert!((0..=239).contains(&self.y) || self.y == 261);
        let step = ((self.x - 257) % 8) + 1;
        let sprite_i = ((self.x - 257)/8) as usize;
        match step {
            1 => { // Read y-coordinate
                self.oam_sprite_fetched_y = self.oam_secondary[sprite_i * 4]
            },
            2 => { // Read tile index
                self.oam_sprite_fetched_tile_index = self.oam_secondary[sprite_i * 4 + 1];
            },
            3 => { // Read attributes
                self.oam_latches[sprite_i] = self.oam_secondary[sprite_i * 4 + 2];
            },
            4 => { // Read x-coordinate
                self.oam_counters[sprite_i] = self.oam_secondary[sprite_i * 4 + 3];
            },
            5 => {
                self.oam_pattern_low[sprite_i] = self.fetch_sprite_tile_byte(
                    sprite_i, false);
            },
            6 => (),
            7 => {
                self.oam_pattern_high[sprite_i] = self.fetch_sprite_tile_byte(
                    sprite_i, true);
            },
            8 => (),
            _ => unreachable!(),
        }
    }

    fn fetch_sprite_tile_byte(&mut self, sprite_i: usize, high_plane: bool) -> u8 {
        debug_assert!((257..=320).contains(&self.x) && (((self.x - 257) % 8) + 1 == 5 || ((self.x - 257) % 8) + 1 == 7));
        debug_assert!((0..=239).contains(&self.y) || self.y == 261);
        let sprite_y = self.oam_sprite_fetched_y;
        if (0xef..=0xff).contains(&sprite_y) {
            return 0; // Hide the sprite
        }
        let next_y = self.next_y();
        if !(sprite_y as u16 + 1..sprite_y as u16 + 9).contains(&next_y) {
            return 0;
        }
        let mut scanline_y = next_y;

        let flip_h = self.oam_latches[sprite_i] & MASK_FLIP_SPRITE_HORIZONTALLY > 0;
        let flip_v = self.oam_latches[sprite_i] & MASK_FLIP_SPRITE_VERTICALLY > 0;
        let sprite_y_fixed = sprite_y as u16 + 1;
        if flip_v {
            scanline_y = 7 - (scanline_y - sprite_y_fixed) + sprite_y_fixed; // Flip vertically
        }

        let mut tile_byte = self.get_sprite_tile_byte(
            self.oam_sprite_fetched_tile_index,
            sprite_y_fixed,
            scanline_y,
            high_plane
        );

        if flip_h {
            tile_byte = tile_byte.reverse_bits(); // Flip horizontally
        }
        tile_byte
    }

    pub fn visible_scanline(&mut self) {
        self.fetch_stuff();

        let mut sprite_color: Option<display::Color> = None;
        let show_sprites = (self.ppumask >> 4) & 1 == 1;
        if show_sprites && 1 <= self.x && self.x <= 256 {
            // "render"
            for (i, counter) in self.oam_counters.iter().enumerate() {
                if *counter != 0 {
                    continue;
                }
                let pattern_high = self.oam_pattern_high[i] & 1;
                let pattern_low = self.oam_pattern_low[i] & 1;
                let pattern = (pattern_high << 1) | pattern_low;
                if pattern == 0 {
                    continue;
                }
                
                let attribute = self.oam_latches[i];
                let palette_number = (attribute & 0x3) + 4;
                let color_address = (palette_number << 2) + pattern;

                let color_number_in_big_palette = self.bus.read(0x3F00 + color_address as u16);
                let color =  self.palette.get_color(color_number_in_big_palette as usize).clone();

                sprite_color = Some(color);
                break;
            }
        }

        
        if 1 <= self.x && self.x <= 256 {
            let show_background = (self.ppumask >> 3) & 1 == 1;

            let background_color = if show_background {
                self.get_background_color()
            } else {
                self.palette.get_color(0).clone()
            };

            let color = if sprite_color.is_some() {
                sprite_color.unwrap()
            }
            else {
                background_color
            };

            self.display.set_pixel(
                (self.x - 1) as usize,
                self.y as usize,
                color,
            );

            // Shift sprite pattern bytes
            for i in 0..self.oam_pattern_high.len() {
                if self.oam_counters[i] == 0 {
                    self.oam_pattern_high[i] >>= 1;
                    self.oam_pattern_low[i] >>= 1;
                }
            }
    
            // Decrease oam counters
            self.oam_counters
                .iter_mut()
                .filter(|c| **c > 0)
                .for_each(|c| *c -= 1);
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

    /// Returns the address of the given tile in the given pattern table.
    ///
    /// Pattern table address should be either 0x0000 or 0x1000.
    /// Pattern index is anything from 0..=255.
    /// Resulting address should be of the form 0b000X_YYYY_YYYY_0000
    /// where X is the pattern table index and Y is the index of the pattern tile.
    ///
    /// Useful links:
    /// [Nesdev wiki - PPU pattern tables]
    ///
    /// [Nesdev wiki - PPU pattern tables]: https://wiki.nesdev.com/w/index.php/PPU_pattern_tables
    fn get_pattern_table_tile_address(&self, pattern_table_address: u16, pattern_index: u8) -> u16 {
        pattern_table_address | ((pattern_index as u16) << 4)
    }

    /// Returns the address of the current [pattern table] for background.
    ///
    /// Current [pattern table] addresses are defined in the register [PPUCTRL].
    ///
    /// [PPUCTRL]: https://wiki.nesdev.com/w/index.php/PPU_registers#PPUCTRL
    /// [pattern table]: https://wiki.nesdev.com/w/index.php/PPU_pattern_tables
    fn get_current_background_pattern_table_address(&self) -> u16 {
        ((self.ppuctrl & MASK_CONTROLLER_BACKGROUND_PATTERN_TABLE_ADDRESS) as u16) << 8
    }

    /// Returns the address of the current [pattern table] for sprites.
    ///
    /// Current [pattern table] addresses are defined in the register [PPUCTRL].
    ///
    /// [PPUCTRL]: https://wiki.nesdev.com/w/index.php/PPU_registers#PPUCTRL
    /// [pattern table]: https://wiki.nesdev.com/w/index.php/PPU_pattern_tables
    fn get_current_sprite_pattern_table_address(&self) -> u16 {
        ((self.ppuctrl & MASK_CONTROLLER_SPRITE_PATTERN_TABLE_ADDRESS) as u16) << 8
    }

    /// Returns the next background tile's pattern byte's address.
    fn get_next_background_pattern_tile_byte_address(&self) -> u16 {
        let bg_pattern_table_address = self.get_current_background_pattern_table_address();
        let bg_pattern_tile_address = self.get_pattern_table_tile_address(
            bg_pattern_table_address,
            self.latch_nametable);
        let (_, y) = self.get_next_tile_xy();
        let pattern_fine_y_offset = y & 0b0111;
        bg_pattern_tile_address | pattern_fine_y_offset
    }

    /// Loads the lower background pattern latch with next value.
    fn load_low_background_tile_byte_latch(&mut self) {
        let address = self.get_next_background_pattern_tile_byte_address();
        self.latch_background_pattern_low = self.bus.read(address).reverse_bits();
    }

    /// Loads the higher background pattern latch with next value.
    fn load_high_background_tile_byte_latch(&mut self) {
        let address = self.get_next_background_pattern_tile_byte_address() | 0x8;
        self.latch_background_pattern_high = self.bus.read(address).reverse_bits();
    }

    /// Returns sprite tile byte from current pattern_table when given the pattern index, sprite y, scanline y
    fn get_sprite_tile_byte(&self, pattern_index: u8, sprite_y: u16, scanline_y: u16, high_plane: bool) -> u8 {
        let sprite_pattern_table_address = self.get_current_sprite_pattern_table_address();
        let sprite_address = self.get_pattern_table_tile_address(
            sprite_pattern_table_address, pattern_index);
        if (sprite_y..sprite_y+8).contains(&scanline_y) {
            let pattern_fine_y_offset = scanline_y - sprite_y;
            let mut sprite_tile_byte_address = sprite_address | pattern_fine_y_offset as u16;
            if high_plane {
                sprite_tile_byte_address |= 0x8;
            }
            return self.bus.read(sprite_tile_byte_address).reverse_bits()
        }
        0
    }

    /// Loads each tile from given pattern table into given display.
    ///
    /// A tile is 8x8 matrix where each cell has value 0..=3.
    /// Each value represents an index of a color in a palette.
    /// There are no palettes because this is not a normal render pipeline,
    /// therefore a greyscale palette is used.
    pub fn load_pattern_table_tiles_to_display(&mut self, pattern_table_address: u16, display: &mut Display) {
        debug_assert!(display.height == 128 && display.width == 128);
        for tile_row in 0x0..=0xf {
            for fine_y_offset in 0x0..=0x7 {
                for tile_col in 0x0..=0xf {
                    let pattern_address = pattern_table_address | (tile_row << 8) | (tile_col << 4) | fine_y_offset;
                    let pattern_low = self.bus.read(pattern_address);
                    let pattern_high = self.bus.read(pattern_address | 0b0000_1000);
                    for x in 0..=7 {
                        let shift = 7 - x;
                        let low_bit = (pattern_low >> shift) & 1;
                        let high_bit = (pattern_high >> shift) & 1;
                        let pattern = low_bit | (high_bit << 1);
                        let i: usize = ((tile_row as usize) << (10)) | ((fine_y_offset as usize) << (7)) | ((tile_col as usize) << 3) | x as usize;
                        display.set_pixel(i % 128, i / 128, palette::PALETTE_GREYSCALE[pattern as usize]);
                    }
                }
            }
        }
    }
}
