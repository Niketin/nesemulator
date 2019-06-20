pub mod cpu;

pub fn start() {
    // Some test code
    let mut ram = cpu::ram::Ram::new(0x0800);
    let mut cpu_bus = cpu::bus::Bus::new(ram);
    let mut cpu = cpu::Cpu::new(cpu_bus);
    cpu.step();
}
