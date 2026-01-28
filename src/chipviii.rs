use raylib::prelude::*;
use std::time::Instant;

pub const SCALE: i32 = 10;
pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;

const PROGRAM_START: u16 = 0x200;
const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub enum Opcode {
    // 00**
    Cls, // 00E0
    Ret, // 00EE

    // 1nnn / 2nnn
    Jp { addr: u16 },   // 1nnn
    Call { addr: u16 }, // 2nnn

    // 3xkk / 4xkk / 5xy0
    SeByte { x: u8, byte: u8 },  // 3xkk
    SneByte { x: u8, byte: u8 }, // 4xkk
    SeReg { x: u8, y: u8 },      // 5xy0

    // 6xkk / 7xkk
    LdByte { x: u8, byte: u8 },  // 6xkk
    AddByte { x: u8, byte: u8 }, // 7xkk

    // 8xy*
    LdReg { x: u8, y: u8 },  // 8xy0
    Or { x: u8, y: u8 },     // 8xy1
    And { x: u8, y: u8 },    // 8xy2
    Xor { x: u8, y: u8 },    // 8xy3
    AddReg { x: u8, y: u8 }, // 8xy4
    Sub { x: u8, y: u8 },    // 8xy5
    Shr { x: u8 },           // 8xy6
    SubN { x: u8, y: u8 },   // 8xy7
    Shl { x: u8 },           // 8xyE

    // 9xy0
    SneReg { x: u8, y: u8 }, // 9xy0

    // Annn / Bnnn
    LdI { addr: u16 },  // Annn
    JpV0 { addr: u16 }, // Bnnn

    // Cxkk
    Rnd { x: u8, byte: u8 }, // Cxkk

    // Dxyn
    Draw { x: u8, y: u8, n: u8 }, // Dxyn

    // Ex**
    Skp { x: u8 },  // Ex9E
    Sknp { x: u8 }, // ExA1

    // Fx**
    LdXDt { x: u8 },  // Fx07
    LdXK { x: u8 },   // Fx0A
    LdDtX { x: u8 },  // Fx15
    LdStX { x: u8 },  // Fx18
    AddI { x: u8 },   // Fx1E
    LdF { x: u8 },    // Fx29
    LdB { x: u8 },    // Fx33
    LdIReg { x: u8 }, // Fx55
    LdRegI { x: u8 }, // Fx65

    Unknown(u16),
}

pub struct ChipVIIIState {
    memory: [u8; 4096],
    pc: u16,

    v: [u8; 16], // V0-VF
    i: u16,

    stack: [u16; 16],
    sp: u8,

    delay_timer: u8,
    sound_timer: u8,

    pub keys: [bool; 16],
    pub display: [bool; DISPLAY_WIDTH * DISPLAY_HEIGHT],

    pub draw_flag: bool,
    pub wait_for_key: Option<u8>,

    last_timer_tick: Instant,
    pub cycles_per_second: u32,
}

impl ChipVIIIState {
    pub fn new() -> Self {
        let mut state = Self {
            memory: [0; 4096],
            pc: PROGRAM_START,
            v: [0; 16],
            i: 0,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keys: [false; 16],
            display: [false; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            draw_flag: false,
            wait_for_key: None,
            last_timer_tick: Instant::now(),
            cycles_per_second: 500,
        };

        state.memory[..FONTSET.len()].copy_from_slice(&FONTSET);

        state
    }

    pub fn read_rom(&mut self, filename: &str) {
        let rom = std::fs::read(filename).expect("Failed to read ROM");
        self.memory[PROGRAM_START as usize..PROGRAM_START as usize + rom.len()]
            .copy_from_slice(&rom);
    }

    pub fn render(&mut self, d: &mut RaylibDrawHandle) {
        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                let idx = x + y * DISPLAY_WIDTH;
                if self.display[idx] {
                    let rect = Rectangle::new(
                        (x as i32 * SCALE) as f32,
                        (y as i32 * SCALE) as f32,
                        SCALE as f32,
                        SCALE as f32,
                    );
                    d.draw_rectangle_rec(rect, Color::WHITE);
                }
            }
        }
        self.draw_flag = false;
    }

    pub fn set_key(&mut self, key: usize, pressed: bool) {
        self.keys[key] = pressed;

        if pressed {
            if let Some(x) = self.wait_for_key {
                self.v[x as usize] = key as u8;
                self.wait_for_key = None;
            }
        }
    }

    // Based on real elapsed time
    fn update_timers_real_time(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_timer_tick);
        let ticks = (elapsed.as_secs_f64() * 60.0).floor() as u32; // 60 Hz timers

        if ticks > 0 {
            self.delay_timer = self.delay_timer.saturating_sub(ticks as u8);
            if self.sound_timer > 0 {
                self.sound_timer = self.sound_timer.saturating_sub(ticks as u8);
                /*
                 * TODO:
                 * let beep = rl.load_sound(beep.wav) somewhere
                 * and
                 * rl.play_sound(beep) here
                 */
            }
            self.last_timer_tick = now;
        }
    }

    pub fn run_cycle(&mut self) {
        if self.wait_for_key.is_none() {
            self.cycle();
        }
        self.update_timers_real_time();
    }

    pub fn fetch_opcode(&self) -> Opcode {
        let hi = self.memory[self.pc as usize] as u16;
        let lo = self.memory[self.pc as usize + 1] as u16;
        let raw = (hi << 8) | lo;

        let nnn = raw & 0x0FFF;
        let nn = (raw & 0x00FF) as u8;
        let n = (raw & 0x000F) as u8;
        let x = ((raw & 0x0F00) >> 8) as u8;
        let y = ((raw & 0x00F0) >> 4) as u8;

        match raw & 0xF000 {
            0x0000 => match raw {
                0x00E0 => Opcode::Cls,
                0x00EE => Opcode::Ret,
                _ => Opcode::Unknown(raw),
            },

            0x1000 => Opcode::Jp { addr: nnn },
            0x2000 => Opcode::Call { addr: nnn },
            0x3000 => Opcode::SeByte { x, byte: nn },
            0x4000 => Opcode::SneByte { x, byte: nn },
            0x5000 if n == 0 => Opcode::SeReg { x, y },

            0x6000 => Opcode::LdByte { x, byte: nn },
            0x7000 => Opcode::AddByte { x, byte: nn },

            0x8000 => match n {
                0x0 => Opcode::LdReg { x, y },
                0x1 => Opcode::Or { x, y },
                0x2 => Opcode::And { x, y },
                0x3 => Opcode::Xor { x, y },
                0x4 => Opcode::AddReg { x, y },
                0x5 => Opcode::Sub { x, y },
                0x6 => Opcode::Shr { x },
                0x7 => Opcode::SubN { x, y },
                0xE => Opcode::Shl { x },
                _ => Opcode::Unknown(raw),
            },

            0x9000 if n == 0 => Opcode::SneReg { x, y },

            0xA000 => Opcode::LdI { addr: nnn },
            0xB000 => Opcode::JpV0 { addr: nnn },
            0xC000 => Opcode::Rnd { x, byte: nn },
            0xD000 => Opcode::Draw { x, y, n },

            0xE000 => match nn {
                0x9E => Opcode::Skp { x },
                0xA1 => Opcode::Sknp { x },
                _ => Opcode::Unknown(raw),
            },

            0xF000 => match nn {
                0x07 => Opcode::LdXDt { x },
                0x0A => Opcode::LdXK { x },
                0x15 => Opcode::LdDtX { x },
                0x18 => Opcode::LdStX { x },
                0x1E => Opcode::AddI { x },
                0x29 => Opcode::LdF { x },
                0x33 => Opcode::LdB { x },
                0x55 => Opcode::LdIReg { x },
                0x65 => Opcode::LdRegI { x },
                _ => Opcode::Unknown(raw),
            },

            _ => Opcode::Unknown(raw),
        }
    }

    pub fn cycle(&mut self) {
        let opcode = self.fetch_opcode();
        self.pc += 2;

        match opcode {
            // 00**
            Opcode::Cls => {
                self.display.fill(false);
                self.draw_flag = true;
            }
            Opcode::Ret => {
                self.sp -= 1;
                self.pc = self.stack[self.sp as usize];
            }

            // 1nnn / 2nnn
            Opcode::Jp { addr } => self.pc = addr,
            Opcode::Call { addr } => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = addr;
            }

            // 3xkk / 4xkk / 5xy0
            Opcode::SeByte { x, byte } => {
                if self.v[x as usize] == byte {
                    self.pc += 2;
                }
            }
            Opcode::SneByte { x, byte } => {
                if self.v[x as usize] != byte {
                    self.pc += 2;
                }
            }
            Opcode::SeReg { x, y } => {
                if self.v[x as usize] == self.v[y as usize] {
                    self.pc += 2;
                }
            }

            // 6xkk / 7xkk
            Opcode::LdByte { x, byte } => {
                self.v[x as usize] = byte;
            }
            Opcode::AddByte { x, byte } => {
                self.v[x as usize] = self.v[x as usize].wrapping_add(byte);
            }

            // 8xy*
            Opcode::LdReg { x, y } => {
                self.v[x as usize] = self.v[y as usize];
            }
            Opcode::Or { x, y } => {
                self.v[x as usize] |= self.v[y as usize];
            }
            Opcode::And { x, y } => {
                self.v[x as usize] &= self.v[y as usize];
            }
            Opcode::Xor { x, y } => {
                self.v[x as usize] ^= self.v[y as usize];
            }
            Opcode::AddReg { x, y } => {
                let (res, carry) = self.v[x as usize].overflowing_add(self.v[y as usize]);
                self.v[x as usize] = res;
                self.v[0xF] = carry as u8;
            }
            Opcode::Sub { x, y } => {
                let (res, borrow) = self.v[x as usize].overflowing_sub(self.v[y as usize]);
                self.v[x as usize] = res;
                self.v[0xF] = (!borrow) as u8;
            }
            Opcode::Shr { x } => {
                self.v[0xF] = self.v[x as usize] & 1;
                self.v[x as usize] >>= 1;
            }
            Opcode::SubN { x, y } => {
                let (res, borrow) = self.v[y as usize].overflowing_sub(self.v[x as usize]);
                self.v[x as usize] = res;
                self.v[0xF] = (!borrow) as u8;
            }
            Opcode::Shl { x } => {
                self.v[0xF] = (self.v[x as usize] >> 7) & 1;
                self.v[x as usize] <<= 1;
            }

            // 9xy0
            Opcode::SneReg { x, y } => {
                if self.v[x as usize] != self.v[y as usize] {
                    self.pc += 2;
                }
            }

            // Annn / Bnnn
            Opcode::LdI { addr } => self.i = addr,
            Opcode::JpV0 { addr } => self.pc = addr + self.v[0] as u16,

            // Cxkk
            Opcode::Rnd { x, byte } => {
                let r = rand::random::<u8>();
                self.v[x as usize] = r & byte;
            }

            // Dxyn
            Opcode::Draw { x, y, n } => {
                let vx = self.v[x as usize] as usize;
                let vy = self.v[y as usize] as usize;

                self.v[0xF] = 0; // collision flag

                for row in 0..n {
                    let sprite_byte = self.memory[(self.i + row as u16) as usize];

                    for col in 0..8 {
                        if (sprite_byte & (0x80 >> col)) != 0 {
                            let px = (vx + col) % DISPLAY_WIDTH;
                            let py = (vy + row as usize) % DISPLAY_HEIGHT;
                            let idx = px + py * DISPLAY_WIDTH;

                            if self.display[idx] {
                                self.v[0xF] = 1;
                            }

                            self.display[idx] ^= true;
                        }
                    }
                }

                self.draw_flag = true;
            }

            // Ex**
            Opcode::Skp { x } => {
                if self.keys[self.v[x as usize] as usize] {
                    self.pc += 2;
                }
            }
            Opcode::Sknp { x } => {
                if !self.keys[self.v[x as usize] as usize] {
                    self.pc += 2;
                }
            }

            // Fx**
            Opcode::LdXDt { x } => {
                self.v[x as usize] = self.delay_timer;
            }
            Opcode::LdXK { x } => {
                self.wait_for_key = Some(x);
            }
            Opcode::LdDtX { x } => {
                self.delay_timer = self.v[x as usize];
            }
            Opcode::LdStX { x } => {
                self.sound_timer = self.v[x as usize];
            }
            Opcode::AddI { x } => {
                self.i = self.i.wrapping_add(self.v[x as usize] as u16);
            }
            Opcode::LdF { x } => {
                self.i = (self.v[x as usize] as u16) * 5;
            }
            Opcode::LdB { x } => {
                let vx = self.v[x as usize];
                self.memory[self.i as usize] = vx / 100;
                self.memory[self.i as usize + 1] = (vx / 10) % 10;
                self.memory[self.i as usize + 2] = vx % 10;
            }
            Opcode::LdIReg { x } => {
                for idx in 0..=x {
                    self.memory[self.i as usize + idx as usize] = self.v[idx as usize];
                }
            }
            Opcode::LdRegI { x } => {
                for idx in 0..=x {
                    self.v[idx as usize] = self.memory[self.i as usize + idx as usize];
                }
            }

            Opcode::Unknown(raw) => {
                panic!("Unknown opcode {:04X}", raw);
            }
        }
    }
}
