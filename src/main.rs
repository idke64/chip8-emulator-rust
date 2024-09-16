mod cpu;
mod opcodes;

use std::env;
use std::path::PathBuf;


use std::collections::HashMap;

use cpu::CPU;
use minifb::{Key, Window, WindowOptions};

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;
pub const SCALE: usize = 20;
pub const SCALED_WIDTH: usize = WIDTH * SCALE; 
pub const SCALED_HEIGHT: usize = HEIGHT * SCALE;
pub const FPS: usize = 60;

fn main() {
    let mut cpu = CPU::new();

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let rom_path = PathBuf::from(manifest_dir).join("roms/tank.ch8");

    cpu.load_instructions(rom_path.to_str().unwrap());

    let key_map: HashMap<Key, u8> = [
        (Key::X, 0x0),    // 0
        (Key::Key1, 0x1), // 1
        (Key::Key2, 0x2), // 2
        (Key::Key3, 0x3), // 3
        (Key::Q, 0x4),    // 4
        (Key::W, 0x5),    // 5
        (Key::E, 0x6),    // 6
        (Key::A, 0x7),    // 7
        (Key::S, 0x8),    // 8
        (Key::D, 0x9),    // 9
        (Key::Z, 0xA),    // A
        (Key::C, 0xB),    // B
        (Key::Key4, 0xC), // C
        (Key::R, 0xD),    // D
        (Key::F, 0xE),    // E
        (Key::V, 0xF),    // F
    ].iter().cloned().collect();
    
    let mut window = Window::new("Chip 8 emulator", 
    SCALED_WIDTH, 
    SCALED_HEIGHT, 
    WindowOptions::default())
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.set_target_fps(FPS);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        cpu.cycle();

        let mut buffer: Vec<u32> = vec![0; SCALED_WIDTH * SCALED_HEIGHT];

        for (y, row) in cpu.display.iter().enumerate() {
            for (x, &pixel) in row.iter().enumerate() {
                let color = if pixel { 0xFFFFFF } else { 0x000000 };

                // scale each pixel
                for dy in 1..(SCALE - 1) {
                    for dx in 1..(SCALE - 1) {
                        let scaled_x = x * SCALE + dx;
                        let scaled_y = y * SCALE + dy;
                        buffer[scaled_y * SCALED_WIDTH + scaled_x] = color;
                    }
                }
            }
        }
        
        cpu.keyboard = [false; 16];
        for (key, index) in &key_map {
            if window.is_key_down(*key) {
                cpu.keyboard[*index as usize] = true;
            }
        }

        window
            .update_with_buffer(&buffer, SCALED_WIDTH, SCALED_HEIGHT)
            .unwrap();
    }
}
