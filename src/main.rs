mod chipviii;
use crate::chipviii::{ChipVIIIState, DISPLAY_HEIGHT, DISPLAY_WIDTH, SCALE};

use raylib::prelude::*;
use std::time::{Duration, Instant};

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(DISPLAY_WIDTH as i32 * SCALE, DISPLAY_HEIGHT as i32 * SCALE)
        .title("CHIP-8 Emulator")
        .build();

    let mut chip8 = ChipVIIIState::new();
    chip8.read_rom("breakout.ch8");

    let mut last_cycle = Instant::now();
    let cycle_delay = Duration::from_micros(1_000_000 / chip8.cycles_per_second as u64);

    while !rl.window_should_close() {
        for i in 0..16 {
            chip8.set_key(
                i,
                rl.is_key_down(match i {
                    0x0 => KeyboardKey::KEY_X,
                    0x1 => KeyboardKey::KEY_ONE,
                    0x2 => KeyboardKey::KEY_TWO,
                    0x3 => KeyboardKey::KEY_THREE,
                    0x4 => KeyboardKey::KEY_Q,
                    0x5 => KeyboardKey::KEY_W,
                    0x6 => KeyboardKey::KEY_E,
                    0x7 => KeyboardKey::KEY_A,
                    0x8 => KeyboardKey::KEY_S,
                    0x9 => KeyboardKey::KEY_D,
                    0xA => KeyboardKey::KEY_Z,
                    0xB => KeyboardKey::KEY_C,
                    0xC => KeyboardKey::KEY_FOUR,
                    0xD => KeyboardKey::KEY_R,
                    0xE => KeyboardKey::KEY_F,
                    0xF => KeyboardKey::KEY_V,
                    _ => KeyboardKey::KEY_NULL,
                }),
            );
        }

        if last_cycle.elapsed() >= cycle_delay {
            chip8.run_cycle();
            last_cycle = Instant::now();
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        chip8.render(&mut d);
    }
}
