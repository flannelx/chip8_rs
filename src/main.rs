use std::env;
use chip8_rs::chip8_sdl2::Chip8Sdl;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut chip8sdl = Chip8Sdl::new(
        args[1].parse::<u64>().unwrap(),
        args[2].parse::<u32>().unwrap(),
        args[3].clone(),
    );
    chip8sdl.run();
}
