extern crate sdl2;
use std::thread;
use std::time::Duration;
use std::{
    fs::File,
    io::{BufReader, Read},
};

use rodio::OutputStreamBuilder;
use rodio::{Source, source::SineWave};

use rand::Rng;

use sdl2::rect::Point;
use sdl2::{event::Event, keyboard::Keycode, pixels::Color};

const START_ADDRESS: u16 = 0x200;
const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
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

struct Chip8 {
    registers: [u8; 16],
    mem: [u8; 4096],
    index_register: u16,
    pc: u16,
    stack: [u16; 16],
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,
    input_keys: [u8; 16],
    display: [u32; 64 * 32],
    opcode: u16,
}

impl Chip8 {
    fn new() -> Self {
        let mut memory: [u8; 4096] = [0; 4096];

        memory[0x50..160].clone_from_slice(&FONTSET);

        Chip8 {
            registers: [0; 16],
            mem: memory,
            index_register: 0,
            pc: START_ADDRESS,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            input_keys: [0; 16],
            display: [0; 64 * 32],
            opcode: 0,
        }
    }

    fn load_rom(&mut self, file_name: &str) -> std::io::Result<()> {
        let file = File::open(file_name)?;
        let lecteur = BufReader::new(file);
        let mut i = 512;
        for bytes in lecteur.bytes() {
            let byte = bytes?;
            self.mem[i] = byte;
            i += 1;
        }
        Ok(())
    }

    fn fetch(&mut self) {
        let cmp = (self.mem[self.pc as usize] as u16) << 8 | self.mem[self.pc as usize + 1] as u16;
        self.opcode = cmp;
        self.pc += 2;
    }

    fn clear(&mut self) {
        self.display.clone_from_slice(&[0; 64 * 32]);
    }

    fn jump(&mut self) {
        self.pc = self.opcode & 0x0FFF;
    }

    fn set(&mut self) {
        self.registers[((self.opcode >> 8) & 0xF) as usize] = (self.opcode & 0x00FF) as u8;
    }

    fn add(&mut self) {
        self.registers[((self.opcode >> 8) & 0xF) as usize] = self.registers
            [((self.opcode >> 8) & 0xF) as usize]
            .wrapping_add((self.opcode & 0xFF) as u8);
    }

    fn set_index(&mut self) {
        self.index_register = self.opcode & 0x0FFF;
    }

    fn call_sub(&mut self) {
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = self.opcode & 0xFFF;
    }

    fn return_sub(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }

    fn if_eq(&mut self) {
        let x = (self.opcode >> 8) & 0xF;
        let val = (self.opcode & 0xFF) as u8;

        if self.registers[x as usize] == val {
            self.pc += 2;
        }
    }

    fn if_ne(&mut self) {
        let x = (self.opcode >> 8) & 0xF;
        let val = (self.opcode & 0xFF) as u8;

        if self.registers[x as usize] != val {
            self.pc += 2;
        }
    }

    fn if_xy_eq(&mut self) {
        let x = (self.opcode >> 8) & 0xF;
        let y = (self.opcode >> 4) & 0xF;

        if self.registers[x as usize] == self.registers[y as usize] {
            self.pc += 2;
        }
    }

    fn if_xy_ne(&mut self) {
        let x = (self.opcode >> 8) & 0xF;
        let y = (self.opcode >> 4) & 0xF;

        if self.registers[x as usize] != self.registers[y as usize] {
            self.pc += 2;
        }
    }

    fn set_arith(&mut self) {
        let x = (self.opcode >> 8) & 0xF;
        let y = (self.opcode >> 4) & 0xF;

        self.registers[x as usize] = self.registers[y as usize];
    }

    fn or(&mut self) {
        let x = (self.opcode >> 8) & 0xF;
        let y = (self.opcode >> 4) & 0xF;

        self.registers[x as usize] |= self.registers[y as usize];
    }

    fn and(&mut self) {
        let x = (self.opcode >> 8) & 0xF;
        let y = (self.opcode >> 4) & 0xF;

        self.registers[x as usize] &= self.registers[y as usize];
    }

    fn xor(&mut self) {
        let x = (self.opcode >> 8) & 0xF;
        let y = (self.opcode >> 4) & 0xF;

        self.registers[x as usize] ^= self.registers[y as usize];
    }

    fn add_(&mut self) {
        let x = (self.opcode >> 8) & 0xF;
        let y = (self.opcode >> 4) & 0xF;

        let (res, overflow) =
            self.registers[x as usize].overflowing_add(self.registers[y as usize]);
        self.registers[x as usize] = res;
        self.registers[0xF] = if overflow { 0 } else { 1 };
    }

    fn sub_xy(&mut self) {
        let x = (self.opcode >> 8) & 0xF;
        let y = (self.opcode >> 4) & 0xF;

        let (res, overflow) =
            self.registers[x as usize].overflowing_sub(self.registers[y as usize]);
        self.registers[x as usize] = res;
        self.registers[0xF] = if overflow { 0 } else { 1 };
    }

    fn sub_yx(&mut self) {
        let x = (self.opcode >> 8) & 0xF;
        let y = (self.opcode >> 4) & 0xF;

        let (res, overflow) =
            self.registers[y as usize].overflowing_sub(self.registers[x as usize]);
        self.registers[x as usize] = res;
        self.registers[0xF] = if overflow { 0 } else { 1 };
    }

    fn shift_right(&mut self) {
        let x = (self.opcode >> 8) & 0xF;
        let y = (self.opcode >> 4) & 0xF;

        let vy = self.registers[y as usize];

        self.registers[0xF] = vy & 0x1;
        self.registers[x as usize] = vy >> 1;
    }

    fn shift_left(&mut self) {
        let x = (self.opcode >> 8) & 0xF;
        let y = (self.opcode >> 4) & 0xF;

        let vy = self.registers[y as usize];

        self.registers[0xF] = (vy >> 7) & 0x1;
        self.registers[x as usize] = vy << 1;
    }

    fn jump_offset(&mut self) {
        self.pc = (self.opcode & 0xFFF) + self.registers[0] as u16;
    }

    fn random(&mut self) {
        let x = (self.opcode >> 8) & 0xF;
        let mut rng = rand::rng();
        let n: u8 = rng.random();
        self.registers[x as usize] = n & (self.opcode & 0xFF) as u8;
    }

    fn if_key_pressed(&mut self) {
        if self.input_keys[self.registers[((self.opcode >> 8) & 0xF) as usize] as usize] == 1 {
            self.pc += 2;
        }
    }

    fn if_key_non_pressed(&mut self) {
        if self.input_keys[self.registers[((self.opcode >> 8) & 0xF) as usize] as usize] != 1 {
            self.pc += 2;
        }
    }

    fn vx_timer(&mut self) {
        let x = (self.opcode >> 8) & 0xF;
        self.registers[x as usize] = self.delay_timer;
    }

    fn timer_vx(&mut self) {
        let x = (self.opcode >> 8) & 0xF;
        self.delay_timer = self.registers[x as usize];
    }

    fn sound_vx(&mut self) {
        let x = (self.opcode >> 8) & 0xF;
        self.sound_timer = self.registers[x as usize];
    }

    fn add_index(&mut self) {
        self.index_register += self.registers[((self.opcode >> 8) & 0xF) as usize] as u16;
    }

    fn get_key(&mut self) {
        for i in 0..16 {
            if self.input_keys[i] == 1 {
                self.registers[((self.opcode >> 8) & 0xF) as usize] = i as u8;
                return;
            }
        }
        self.pc -= 2;
    }

    fn font_car(&mut self) {
        self.index_register = 0x50 + self.registers[((self.opcode >> 8) & 0xF) as usize] as u16 * 5;
    }

    fn decode(&mut self) {
        let num = self.registers[((self.opcode >> 8) & 0xF) as usize];
        self.mem[self.index_register as usize] = (num / 100) % 10;
        self.mem[(self.index_register + 1) as usize] = (num / 10) % 10;
        self.mem[(self.index_register + 2) as usize] = num % 10;
    }

    fn store_mem(&mut self) {
        let x = (self.opcode >> 8) & 0xF;
        for i in 0..x + 1 {
            self.mem[self.index_register as usize + i as usize] = self.registers[i as usize];
        }
    }

    fn load_mem(&mut self) {
        let x = (self.opcode >> 8) & 0xF;
        for i in 0..x + 1 {
            self.registers[i as usize] = self.mem[self.index_register as usize + i as usize];
        }
    }

    fn display(&mut self) {
        let mut sprite;
        let x_start = self.registers[((self.opcode >> 8) & 0xF) as usize] % 64;
        let y_start = self.registers[((self.opcode >> 4) & 0xF) as usize] % 32;
        let n = self.opcode & 0xF;

        self.registers[0xF] = 0;
        let mut bit;

        for i in 0..n {
            sprite = self.mem[(self.index_register + i) as usize];
            for j in 0..8 {
                let x = (x_start + j) % 64;
                let y = (y_start + i as u8) % 32;

                let idx = y as usize * 64 + x as usize;
                bit = (sprite >> (7 - j)) & 1;

                if bit == 1 {
                    if self.display[idx] == 1 {
                        self.registers[0xF] = 1;
                    }
                    self.display[idx] ^= 1;
                }
            }
        }
    }

    pub fn tick(&mut self) {
        self.fetch();

        let op = self.opcode;
        let nibble1 = (op & 0xF000) >> 12;
        let nibble2 = (op & 0x0F00) >> 8;
        let nibble3 = (op & 0x00F0) >> 4;
        let nibble4 = op & 0x000F;

        match (nibble1, nibble2, nibble3, nibble4) {
            (0x0, 0x0, 0xE, 0x0) => self.clear(), // 00E0 (Clear Screen)
            (0x1, _, _, _) => self.jump(),        // 1NNN (Jump)
            (0x6, _, _, _) => self.set(),         // 6XNN (Set Register)
            (0x7, _, _, _) => self.add(),         // 7XNN (Add to Register)
            (0xA, _, _, _) => self.set_index(),   // ANNN (Set Index Register)
            (0xD, _, _, _) => self.display(),     // DXYN (Your display func)
            (0x2, _, _, _) => self.call_sub(),
            (0, 0, 0xE, 0xE) => self.return_sub(),
            (0x3, _, _, _) => self.if_eq(),
            (0x4, _, _, _) => self.if_ne(),
            (0x5, _, _, 0) => self.if_xy_eq(),
            (0x9, _, _, 0) => self.if_xy_ne(),
            (0x8, _, _, 0) => self.set_arith(),
            (0x8, _, _, 1) => self.or(),
            (0x8, _, _, 2) => self.and(),
            (0x8, _, _, 3) => self.xor(),
            (0x8, _, _, 4) => self.add_(),
            (0x8, _, _, 5) => self.sub_xy(),
            (0x8, _, _, 7) => self.sub_yx(),
            (0x8, _, _, 6) => self.shift_right(),
            (0x8, _, _, 0xE) => self.shift_left(),
            (0xB, _, _, _) => self.jump_offset(),
            (0xC, _, _, _) => self.random(),
            (0xE, _, 0x9, 0xE) => self.if_key_pressed(),
            (0xE, _, 0xA, 0x1) => self.if_key_non_pressed(),
            (0xF, _, 0, 0x7) => self.vx_timer(),
            (0xF, _, 0x1, 0x5) => self.timer_vx(),
            (0xF, _, 0x1, 0x8) => self.sound_vx(),
            (0xF, _, 0x1, 0xE) => self.add_index(),
            (0xF, _, 0, 0xA) => self.get_key(),
            (0xF, _, 0x2, 0x9) => self.font_car(),
            (0xF, _, 0x3, 0x3) => self.decode(),
            (0xF, _, 0x5, 0x5) => self.store_mem(),
            (0xF, _, 0x6, 0x5) => self.load_mem(),

            _ => println!("Unknown opcode: {:X}", op),
        }
    }

    fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    fn draw_ascii(&self) {
        //Clear screen
        print!("\x1B[2J\x1B[1;1H");

        for y in 0..32 {
            let mut line = String::new();

            for x in 0..64 {
                let index = y * 64 + x;
                let pixel = self.display[index];

                if pixel == 1 {
                    line.push('█');
                } else {
                    line.push(' ');
                }
            }

            println!("{}", line);
        }
    }

    fn draw(&mut self) -> Result<(), String> {
        // Set up sdl2 for graphics
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window("rust-sdl2 example", 800, 600)
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        let mut event_pump = sdl_context.event_pump()?;

        let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

        canvas.set_logical_size(64, 32).map_err(|e| e.to_string())?;

        // Sound with rodio
        let stream_handle = OutputStreamBuilder::open_default_stream().expect("stream");

        let sink = rodio::Sink::connect_new(&stream_handle.mixer());

        let source = SineWave::new(440.0).amplify(0.10); // Volume at 10%
        // stream_handle.mixer().add(source);
        sink.append(source);
        sink.pause();
        // sink.sleep_until_end();

        'main: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'main,

                    // Key Pressed -> Set to TRUE
                    Event::KeyDown {
                        keycode: Some(key), ..
                    } => {
                        if let Some(idx) = key2btn(key) {
                            self.input_keys[idx] = 1;
                        }
                    }

                    // Key Released -> Set to FALSE
                    Event::KeyUp {
                        keycode: Some(key), ..
                    } => {
                        if let Some(idx) = key2btn(key) {
                            self.input_keys[idx] = 0;
                        }
                    }
                    _ => {}
                }
            }

            for _ in 0..10 {
                self.tick();
            }
            if self.sound_timer == 0 {
                sink.pause();
            } else if self.sound_timer > 0 && sink.is_paused() {
                sink.play();
            }
            self.update_timers();

            // Set the background
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();

            // Draw a red rectangle
            canvas.set_draw_color(Color::RGB(0, 0, 255));

            for y in 0..32 {
                for x in 0..64 {
                    let idx = y * 64 + x;

                    let pixel = self.display[idx];

                    if pixel == 1 {
                        canvas.draw_point(Point::new(x as i32, y as i32))?;
                    }
                }
            }
            // Show it on the screen
            canvas.present();
            thread::sleep(Duration::new(0, 1_000_000_000 / 60));
        }
        Ok(())
    }
}

fn key2btn(key: Keycode) -> Option<usize> {
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC), // 1 2 3 C

        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD), // 4 5 6 D

        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE), // 7 8 9 E

        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF), // A 0 B F

        _ => None,
    }
}

fn main() -> std::io::Result<()> {
    let mut prog = Chip8::new();

    prog.load_rom("pong.ch8")?;

    if prog.draw().is_err() {
        println!("Oops, something wrong");
    }
    Ok(())
}
