use std::fs;
use std::iter::FromIterator;


const HEADER_SIZE: usize = 16;
const PRG_ROM_PAGE_SIZE: usize = 0x4000; // 16384
const CHR_ROM_PAGE_SIZE: usize = 0x2000; //  8192


pub struct Cartridge {
    mem: Vec<u8>,
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    ines_format: bool,
    nes20_format: bool,
    prg_rom_pages: usize,
    chr_rom_pages: usize,
    mapper_number: u8,
}

impl Cartridge {
    pub fn new() -> Cartridge {
        Cartridge {
            mem: Vec::default(),
            prg_rom: Vec::default(),
            chr_rom: Vec::default(),
            prg_rom_pages: 0,
            chr_rom_pages: 0,
            ines_format: false,
            nes20_format: false,
            mapper_number: 0,
        }
    }

    pub fn new_from_file(path: &String) -> Cartridge {
        let mut rom = Cartridge::new();

        rom.mem = fs::read(path).expect("Failed to open file");

        if !rom.is_ines_format() {
            panic!("Invalid ROM: Missing identification string \"NES<EOF>\".");
        }
        rom.ines_format = true;
        rom.nes20_format = rom.is_nes20_format();

        rom.update_chr_rom_size();
        rom.update_prg_rom_size();
        rom.update_mapper_number();

        if rom.mapper_number != 0 {
            panic!(
                "Unsupported ROM: Mapper number {} not supported.",
                rom.mapper_number
            );
        }


        let prg_start = HEADER_SIZE;
        let prg_end = prg_start + PRG_ROM_PAGE_SIZE * rom.prg_rom_pages;
        let chr_start = prg_end;
        let chr_end = chr_start + CHR_ROM_PAGE_SIZE * rom.chr_rom_pages;
        rom.prg_rom = Vec::from_iter(rom.mem[prg_start..prg_end].iter().cloned());
        rom.chr_rom = Vec::from_iter(rom.mem[chr_start..chr_end].iter().cloned());

        rom
    }

    fn update_prg_rom_size(&mut self) {
        self.prg_rom_pages = self.mem[4] as usize;
        if self.prg_rom_pages == 0 {
            panic!("Invalid ROM: non-positive prg rom size.");
        }
    }

    fn update_chr_rom_size(&mut self) {
        self.chr_rom_pages = self.mem[5] as usize;
        if self.chr_rom_pages == 0 {
            panic!("Invalid ROM: non-positive chr rom size.");
        }
    }

    fn update_mapper_number(&self) -> u8 {
        (self.mem[6] & 0x0F) | (self.mem[7] & 0xF0)
    }

    fn is_ines_format(&self) -> bool {
        self.mem[0..4].eq(b"NES\x1a")
    }

    fn is_nes20_format(&self) -> bool {
        let nes20_bit_check = (self.mem[7] & 0x0C) == 0x08;
        let nes20_size_check = true; // TODO: do size checks with self.mem[9].
        self.ines_format && nes20_bit_check && nes20_size_check
    }


    pub fn read(&self, address: usize) -> u8 {
        match address {
            0x6000...0x7FFF => unimplemented!(),
            0x8000...0xBFFF => self.prg_rom[address - 0x8000usize],
            0xC000...0xFFFF => self.prg_rom[(address - 0x8000usize) % (self.prg_rom_pages * PRG_ROM_PAGE_SIZE)],
            _ => panic!("Trying to read from invalid ROM address."),
        }
    }

    pub fn write(&self, address: usize, value: u8) {
        unimplemented!(); // ROM is read-only memory.
    }

}