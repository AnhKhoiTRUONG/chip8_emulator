use std::{
    fs::File,
    io::{BufRead, BufReader, Bytes, Read, Result},
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

    // fn load_fonts
}

fn main() -> Result<()> {
    let mut prog = Chip8::new();

    prog.load_rom("dummy.ch8")?;

    Ok(())
}
