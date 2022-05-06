use crate::Cartridge;
use crate::nes::{
  memory::{MemRead, MemWrite, Memory},
  mapper::{m000_nrom},
};

pub struct Bus {
  wram: Memory,
  mapper: m000_nrom::Nrom,
  //ppu: PPU,
  //apu: APU,
  //input: Input,
  //mapper: Mapper,
}

impl Bus {
  pub fn new(cartridge: &Cartridge) -> Self {
    Self {
      wram: Memory::ram(0x0800),
      mapper: m000_nrom::Nrom::load(cartridge),
    }
  }

  pub fn mapper_load(&mut self, mapper: m000_nrom::Nrom) {
    self.mapper = mapper;
  }
}


impl MemRead for Bus{
  fn read(&mut self, addr: usize) -> u8 {
    match addr {
      0x0000..=0x0FFF => self.wram.read(addr),
      0x4020..=0xFFFF => self.mapper.read(addr),
      _ => 0,
    }
  }
}

impl MemWrite for Bus{
  fn write(&mut self, addr: usize, value: u8) {
    match addr {
      0x0000..=0x1FFF => self.wram.write(addr, value),
      0x4020..=0xFFFF => self.wram.write(addr, value),
      _ => (),
    }
  }
}
