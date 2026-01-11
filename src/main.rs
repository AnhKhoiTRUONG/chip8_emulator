use std::{
    fs::File,
    io::{BufRead, BufReader, Read, Result},
};

struct chip_8 {
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

impl chip_8 {
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
}

fn main() {
    println!("Hello, world!");
}
