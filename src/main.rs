use std::collections::HashMap;
use std::{fs, time};
use std::ops::Shl;
use indicatif::{ProgressBar, ProgressStyle};
use log::LevelFilter;
use raqote::Color;
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use crate::chip8::Chip8;
use crate::chip8_instruction_set::Instruction;

mod chip8;
mod chip8_instruction_set;

fn main() {
    let mut chip = Chip8::new(4096, 16, 16, Color::new(255, 255, 25, 25),
    HashMap::from([
        (Key::Key1, 0x1),
        (Key::Key2, 0x2),
        (Key::Key3, 0x3),
        (Key::Q, 0x4),
        (Key::W, 0x5),
        (Key::E, 0x6),
        (Key::A, 0x7),
        (Key::S, 0x8),
        (Key::D, 0x9),
        (Key::Z, 0xA),
        (Key::X, 0x0),
        (Key::C, 0xB),
        (Key::Key4, 0xC),
        (Key::R, 0xD),
        (Key::F, 0xE),
        (Key::C, 0xF)
    ]));
    let (screen_width, screen_height) = chip.get_screen_size();
    let mut window = Window::new("Chip-8", screen_width, screen_height, WindowOptions::default()).unwrap();
    let program = fs::read("./roms/test_opcode.ch8").expect("File not found");
    log::set_max_level(LevelFilter::Info);
    chip.load(&program);
    let mut last_tick = time::Instant::now();
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(ProgressStyle::with_template("{spinner} Chip-8 | run time: {elapsed} clock speed: {per_sec}").unwrap());
    while window.is_open() {
        window.get_keys_pressed(KeyRepeat::No).iter().for_each(|k|chip.set_pressed(k, true));
        window.get_keys_released().iter().for_each(|k|chip.set_pressed(k, false));
        spinner.inc(1);
        chip.tick();
        if last_tick.elapsed().as_secs_f32() >= 1f32/60f32 {
            last_tick = time::Instant::now();
            chip.decrement_time();
            window.update_with_buffer(chip.get_screen_buffer(), screen_width, screen_height).unwrap();
        }
    }
    spinner.finish();
}
