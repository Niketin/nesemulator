mod cartridge;
pub mod cpu;
pub mod ppu;
mod controller;

use crate::cartridge::Cartridge;
use crate::ppu::Ppu;
use crate::cpu::Cpu;
use crate::controller::Controller;

pub use crate::controller::Button;


use std::cell::RefCell;
use std::rc::Rc;



pub struct Emulator {
    _cartridge: Rc<RefCell<Cartridge>>,
    pub cpu: Cpu,
}

impl Emulator {
    pub fn new(path: &str) -> Emulator {
        // Some test code
    
        let cartridge = Rc::new(RefCell::new(Cartridge::new_from_file(path.to_owned())));
        let ppu_bus = ppu::bus::Bus::new(cartridge.clone());
        
        let ppu = Ppu::new(ppu_bus);

        let cpu_ram = cpu::ram::Ram::new(0x0800);
        let cpu_bus = cpu::bus::Bus::new(cpu_ram, cartridge.clone());

        let cpu = cpu::Cpu::new(cpu_bus);

        let mut emulator = Emulator {
            _cartridge: cartridge,
            cpu,
        };

        emulator.cpu.bus.set_ppu(ppu);
        emulator.cpu.bus.set_controller(Controller::new());
        emulator
    }

    pub fn step(&mut self) {
        self.cpu.step();
        let ppu = self.cpu.bus.ppu.as_mut().unwrap();
        ppu.step();
        ppu.step();
        ppu.step();
    }

    // Run emulator steps until a frame is ready.
    pub fn step_frame(&mut self) {
        loop {
            self.step();
            let ppu = self.cpu.bus.ppu.as_ref().unwrap();
            if ppu.y == 240 && (0..=2).contains(&ppu.x) {
                break;
            }
        }
    }

    pub fn set_controller_state(&mut self, button: Button, value: bool) {
        if let Some(c) = self.cpu.bus.controller.as_mut() { c.set_button_state(button, value) }
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
    #[ignore = "not yet implemented"]
    fn test_nmi_timing() -> Result<(), std::io::Error> {
        unimplemented!();
    }

    #[test]
    fn test_official_opcodes_with_nestest() -> Result<(), std::io::Error> {
        let rom_path = String::from("tests/nes-test-roms/other/nestest.nes");
        let cartridge = Rc::new(RefCell::new(Cartridge::new_from_file(rom_path.clone())));
        let cpu_ram = cpu::ram::Ram::new(0x0800);
        let cpu_bus = cpu::bus::Bus::new(cpu_ram, cartridge.clone());
        let mut cpu = cpu::Cpu::new(cpu_bus);

        let ppu_bus = ppu::bus::Bus::new(cartridge.clone());
        let ppu = Ppu::new(ppu_bus);
        cpu.bus.set_ppu(ppu);

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
            while cpu.skip_cycles != 0 {
                cpu.step();
                let ppu = cpu.bus.ppu.as_mut().unwrap();
                ppu.step();
                ppu.step();
                ppu.step();
            }

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
                Helper {slice: &line[78..81], name: "ppu x",         base:  10, value: cpu.bus.ppu.as_ref().unwrap().x as u64},
                Helper {slice: &line[82..85], name: "ppu y",         base:  10, value: cpu.bus.ppu.as_ref().unwrap().y as u64},
            ];
            for help in h {
                let log_value = match u64::from_str_radix(help.slice.trim(), help.base) {
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
            let ppu = cpu.bus.ppu.as_mut().unwrap();
            ppu.step();
            ppu.step();
            ppu.step();
        }

        Ok(())
    }
}
