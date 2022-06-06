use std::fs;
use std::error::Error;

use crate::nes::{
  memory::{Memory},
  cpu,
  mapper::{self, Mapper, MapperType},
  ppu::memory::{PPUMemory},
};

pub struct SaveState {
  pub wram: Memory,
  pub mapper: Box<MapperType>,
  pub ppu_mem: PPUMemory,
  pub cpu_reg: cpu::Reg,
}

impl SaveState {
  pub fn new() -> Self {
    Self {
      wram: Memory::new(),
      mapper: Box::new(mapper::null()),
      ppu_mem: PPUMemory::new(),
      cpu_reg: cpu::Reg::new(),
    }
  }

  pub fn save_to_file(&self, filename: &str) -> Result<(), Box<dyn Error>> {
    //let file = fs::write(filename, )?;
    Ok(())
  }
}
