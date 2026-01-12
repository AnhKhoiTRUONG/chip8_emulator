use std::thread;
use std::time::Duration;
use std::{
    fs::File,
    io::{BufRead, BufReader, Bytes, Read, Result},
    mem,
};

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

        memory[50..130].clone_from_slice(&FONTSET);

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

    fn load_rom(&mut self, file_name: &str) -> Result<()> {
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
        self.registers[((self.opcode >> 8) & 0xF) as usize] += (self.opcode & 0x00FF) as u8;
    }

    fn set_index(&mut self) {
        self.index_register = (self.opcode & 0x0FFF);
    }

    fn display(&mut self) {
        let mut sprite;
        let x_start = self.registers[((self.opcode >> 8) & 0xF) as usize] % 64;
        let y_start = self.registers[((self.opcode >> 4) & 0xF) as usize] % 32;
        let n = self.opcode & 0xF;

        self.registers[15] = 0;
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
            _ => println!("Unknown opcode: {:X}", op),
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
}

fn main() -> Result<()> {
    let mut prog = Chip8::new();

    prog.load_rom("IBM_logo.ch8")?;

    loop {
        prog.tick();

        prog.draw_ascii();

        thread::sleep(Duration::from_millis(2));
    }
}
