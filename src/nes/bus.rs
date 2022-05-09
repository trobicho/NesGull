use crate::Cartridge;
use crate::nes::{
  memory::{MemRead, MemWrite, Memory},
  mapper::{m000_nrom},
};


pub struct Bus {
  wram: Memory,
  vram: Memory,
  mapper: m000_nrom::Nrom,
  ppu_reg: Memory,
  ppu_oam: Memory,
  cpu_mapped_reg: Memory,
  //ppu: PPU,
  //apu: APU,
  //input: Input,
  //mapper: Mapper,
}

impl Bus {
  pub fn new(cartridge: &Cartridge) -> Self {
    Self {
      wram: Memory::ram(0x0800),
      vram: Memory::ram(0x0800),
      ppu_reg: Memory::ram(8),
      ppu_oam: Memory::ram(256),
      cpu_mapped_reg: Memory::ram(0x2F),
      mapper: m000_nrom::Nrom::load(cartridge),
    }
  }

  pub fn mapper_load(&mut self, mapper: m000_nrom::Nrom) {
    self.mapper = mapper;
  }
}


impl MemRead for Bus {
  fn read(&mut self, addr: usize) -> u8 {
    match addr {
      0x0000..=0x0800 => self.wram.read(addr),
      0x2000..=0x2007 => self.ppu_reg.read(addr),
      0x4000..=0x401F => self.cpu_mapped_reg.read(addr),
      0x4020..=0xFFFF => self.mapper.read(addr),
      _ => 0,
    }
  }
}

impl MemWrite for Bus {
  fn write(&mut self, addr: usize, value: u8) {
    match addr {
      0x0000..=0x0800 => self.wram.write(addr, value),
      0x2000..=0x2007 => self.ppu_reg.write(addr, value),
      0x4000..=0x401F => self.cpu_mapped_reg.write(addr, value),
      0x4020..=0xFFFF => self.mapper.write(addr, value),
      _ => (),
    }
  }
}

impl Bus {
  pub fn ppu_read(&mut self, addr: usize) -> u8 {
    match addr {
      0x0000..=0x1FFF => self.mapper.read(addr),
      0x2000..=0x2FFF => self.vram.read(addr),
      0x3000..=0x3EFF => self.vram.read(addr),
      _ => 0,
    }
  }

  pub fn ppu_write(&mut self, addr: usize, value: u8) {
    match addr {
      0x0000..=0x1FFF => self.mapper.write(addr, value),
      0x2000..=0x2FFF => self.vram.write(addr, value),
      0x3000..=0x3EFF => self.vram.write(addr, value),
      _ => (),
    }
  }
}
