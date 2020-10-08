mod contracts;
mod cpu;
mod display;
mod sound;
mod emulator;
mod util;

use emulator::Emulator;

fn main() {
    let mut path = "./roms/c8games/BRIX".to_string();
    if std::env::args().len() > 1 {
        let v: Vec<String> = std::env::args().collect();
        path = v[1].clone();
    }
    
    let mut emu = Emulator::new();
    emu.run(&std::fs::read(&path).unwrap());
}
