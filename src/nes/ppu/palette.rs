use std::fs;
use std::error::Error;

#[allow(non_snake_case)]
#[derive(Debug, Copy, Clone)]
pub struct NesColor {
  pub R: u8,
  pub G: u8,
  pub B: u8,
}

pub struct Palette {
  pub color: [NesColor; 64],
}

impl Palette {
  pub fn new() -> Self {
    Self {
      color: [NesColor{R: 0, G: 0, B: 0}; 64],
    }
  }

  pub fn from_file(filename: &str) -> Result<Self, Box<dyn Error>> {
    let file = fs::read(filename)?;
    let mut vec = [NesColor{R: 0, G: 0, B: 0}; 64];
    for i in 0..64 {
      vec[i] = NesColor {
        R: file[i * 3],
        G: file[i * 3 + 1],
        B: file[i * 3 + 2],
      };
    }
    Ok(Self {
      color: vec
    })
  }

  pub fn change_from_file(&mut self, filename: &str) -> Result<(), Box<dyn Error>> {
    let file = fs::read(filename)?;
    for i in 0..64 {
      self.color[i] = NesColor {
        R: file[i * 3],
        G: file[i * 3 + 1],
        B: file[i * 3 + 2],
      };
    }
    Ok(())
  }
}
