mod cartridge;
pub mod cpu;
pub mod ppu;

pub fn run(path: &String) {
    // Some test code
    let cartridge = cartridge::Cartridge::new_from_file(path);
    let ppu_vram = cpu::ram::Ram::new(0x0800);
    let ppu_bus = ppu::bus::Bus::new(ppu_vram, &cartridge);
    let ppu = ppu::Ppu::new(ppu_bus);

    let cpu_ram = cpu::ram::Ram::new(0x0800);
    let cpu_bus = cpu::bus::Bus::new(cpu_ram, &cartridge);

    let mut cpu = cpu::Cpu::new(cpu_bus);
    cpu.bus.set_ppu(ppu);

    loop {
        cpu.step();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;
    use std::io::prelude::*;
    use std::fs::File;

    #[test]
    fn test_official_opcodes_with_nestest() -> std::result::Result<(), std::io::Error> {
        let rom_path = String::from("tests/nestest.nes");
        let cartridge = cartridge::Cartridge::new_from_file(&rom_path);
        let cpu_ram = cpu::ram::Ram::new(0x0800);
        let cpu_bus = cpu::bus::Bus::new(cpu_ram, &cartridge);
        let mut cpu = cpu::Cpu::new(cpu_bus);

        cpu.set_program_counter(0xC000);

        let f = File::open("tests/nestest.log")?;
        let reader = BufReader::new(f);
        let lines_iter = reader.lines().map(|l| l.unwrap());

        struct Helper<'a> {
            slice: &'a str,
            name: &'a str,
            value: u8
        }

        let last_line_official_opcodes = 5004;
        let mut line_number: u32 = 1;

        for line in lines_iter {
            if line_number == last_line_official_opcodes {
                break;
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
                Helper {slice: &line[6..8],   name: "opcode",        value: cpu.get_next_opcode()},
                Helper {slice: &line[50..52], name: "accumulator",   value: cpu.accumulator},
                Helper {slice: &line[55..57], name: "x_index",       value: cpu.x_index},
                Helper {slice: &line[60..62], name: "y_index",       value: cpu.y_index},
                Helper {slice: &line[65..67], name: "status",        value: cpu.status.get_as_byte()},
                Helper {slice: &line[71..73], name: "stack pointer", value: cpu.stack_pointer},
            ];
            for help in h {
                let log_value = match u8::from_str_radix(help.slice, 16) {
                    Ok(t) => t,
                    Err(_) => panic!("Detected wrong format while parsing {} from nestest.log", help.name),
                };
                assert_eq!(help.value, log_value, "Comparing CPU's {} {:X} and log's {} {:X} on line {}", help.name, help.value, help.name, log_value, line_number);
            }

            // Prepare for next line
            cpu.execute_next_opcode();
            line_number += 1;
        }



        Ok(())
    }
}