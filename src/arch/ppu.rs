use std::fs;
use std::error::Error;

#[allow(non_snake_case)]
#[derive(Copy, Clone)]
pub struct NesColor {
  R: u8,
  G: u8,
  B: u8,
}

pub struct PPU {
  memory: [u8; 0x4000],
  palette: [NesColor; 256],
}

impl PPU {
  pub fn new() -> Self {
    Self {
      memory: [0; 0x4000],
      palette: [NesColor{R: 0, G: 0, B: 0}; 256],
    }
  }

  pub fn reset(&mut self) {
  }

  pub fn load(&mut self, chr_ram : &[u8], chr_size : usize) {
    self.memory[0x0..chr_size].clone_from_slice(&chr_ram);
  }

  pub fn load_palette(&mut self, filename: &str) -> Result<(), Box<dyn Error>> {
    let file = fs::read(filename)?;
    for i in 0..64 {
      self.palette[i] = NesColor {
        R: file[i * 3],
        G: file[i * 3 + 1],
        B: file[i * 3 + 2],
      }
    }
    Ok(())
  }

  pub fn show_mem(&self) {
    println!("chr memory:");
    let mut i = 0;
    for addr in &self.memory[..] {
      if i % 16 == 0 {
        println!("");
      }
      print!("{} ", addr);
      i += 1;
    }
    println!("");
  }
}
