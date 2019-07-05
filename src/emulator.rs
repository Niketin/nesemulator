
pub mod bus;
pub mod cartridge;
pub mod cpu;

pub fn run(path: &String) {
    // Some test code
    let ram = cpu::ram::Ram::new(0x0800);
    let cartridge = cartridge::Cartridge::new_from_file(path);
    let cpu_bus = bus::Bus::new(ram, cartridge);
    let _cpu = cpu::Cpu::new(cpu_bus);
}
