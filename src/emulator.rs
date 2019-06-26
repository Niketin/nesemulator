
pub mod bus;
pub mod cartridge;
pub mod cpu;

pub fn run(path: &String) {
    // Some test code
    let mut ram = cpu::ram::Ram::new(0x0800);
    let mut cartridge = cartridge::Cartridge::new_from_file(path);
    let mut cpu_bus = bus::Bus::new(ram, cartridge);
    let mut cpu = cpu::Cpu::new(cpu_bus);
}
