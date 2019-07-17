use emulator;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    emulator::run(&args[1]);
}
