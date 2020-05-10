mod cartridge;
pub mod cpu;
pub mod ppu;

use crate::cartridge::Cartridge;
use crate::cpu::ram::Ram;
use crate::ppu::Ppu;
use crate::cpu::Cpu;

use std::cell::RefCell;
use std::rc::Rc;



pub struct Emulator {
    _cartridge: Rc<RefCell<Cartridge>>,
    pub cpu: Cpu,
}

impl Emulator {
    pub fn new(path: &String) -> Emulator {
        // Some test code
    
        let cartridge = Rc::new(RefCell::new(Cartridge::new_from_file(path.clone())));
        let ppu_vram = Ram::new(0x0800);
        let ppu_bus = ppu::bus::Bus::new(ppu_vram, cartridge.clone());
        
        let ppu = Ppu::new(ppu_bus);

        let cpu_ram = cpu::ram::Ram::new(0x0800);
        let cpu_bus = cpu::bus::Bus::new(cpu_ram, cartridge.clone());

        let cpu = cpu::Cpu::new(cpu_bus);

        let mut emulator = Emulator {
            _cartridge: cartridge,
            cpu
        };

        emulator.cpu.bus.set_ppu(ppu);
        emulator
    }

    pub fn run(&mut self) {
        loop {
            self.cpu.step();
            self.cpu.bus.ppu.as_mut().unwrap().step();
        }
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;
    use std::io::prelude::*;
    use std::fs::File;
    use std::result::Result;

    #[test]
    fn test_nmi_timing() -> Result<(), std::io::Error> {
        unimplemented!();
    }

    #[test]
    fn test_official_opcodes_with_nestest() -> Result<(), std::io::Error> {
        let rom_path = String::from("tests/nestest.nes");
        let cartridge = Rc::new(RefCell::new(Cartridge::new_from_file(rom_path.clone())));
        let cpu_ram = cpu::ram::Ram::new(0x0800);
        let cpu_bus = cpu::bus::Bus::new(cpu_ram, cartridge.clone());
        let mut cpu = cpu::Cpu::new(cpu_bus);

        cpu.set_program_counter(0xC000);

        let f = File::open("tests/nestest.log")?;
        let reader = BufReader::new(f);
        let lines_iter = reader.lines().map(|l| l.unwrap());

        struct Helper<'a> {
            slice: &'a str,
            name: &'a str,
            value: u64,
            base: u32
        }

        let last_line_official_opcodes = 5004;
        let mut line_number: u32 = 1;

        for line in lines_iter {
            if line_number == last_line_official_opcodes {
                break;
            }

            // Skip cycles
            while cpu.skip_cycles != 0 { cpu.step();}

            // Program counter check
            let log_program_counter = match u16::from_str_radix(&line[0..4], 16) {
                Ok(t) => t,
                Err(_) => panic!("Detected wrong format while parsing program counter from nestest.log"),
            };
            let cpu_program_counter = cpu.program_counter;
            assert_eq!(cpu_program_counter, log_program_counter, "Comparing CPU's program counter {:X} and log's program counter {:X} on line {}", cpu_program_counter, log_program_counter, line_number);

            // Opcode and status check
            let h = vec![
                Helper {slice: &line[6..8],   name: "opcode",        base:  16, value: cpu.get_next_opcode() as u64},
                Helper {slice: &line[50..52], name: "accumulator",   base:  16, value: cpu.accumulator as u64},
                Helper {slice: &line[55..57], name: "x_index",       base:  16, value: cpu.x_index as u64},
                Helper {slice: &line[60..62], name: "y_index",       base:  16, value: cpu.y_index as u64},
                Helper {slice: &line[65..67], name: "status",        base:  16, value: cpu.status.get_as_byte() as u64},
                Helper {slice: &line[71..73], name: "stack pointer", base:  16, value: cpu.stack_pointer as u64},
                Helper {slice: &line[90..],   name: "cycle",         base:  10, value: cpu.cycle},
            ];
            for help in h {
                let log_value = match u64::from_str_radix(help.slice, help.base) {
                    Ok(t) => t,
                    Err(_) => panic!("Detected wrong format while parsing {} from nestest.log", help.name),
                };
                match help.base {
                    10 => assert_eq!(help.value, log_value, "Comparing CPU's {} {} and log's {} {} on line {}", help.name, help.value, help.name, log_value, line_number),
                    _  => assert_eq!(help.value, log_value, "Comparing CPU's {} {:X} and log's {} {:X} on line {}", help.name, help.value, help.name, log_value, line_number)
                }
            }

            // Prepare for next line
            line_number += 1;
            cpu.step();
        }

        Ok(())
    }
}
