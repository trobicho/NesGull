use crate::Cartridge;
use crate::nes::{
  memory::{MemRead, MemWrite, Memory},
  mapper::{self, Mapper, MapperType},
  ppu::memory::{PPUMemory, OAMDMA_CPU_ADDR},
  controller::{Controller},
};

pub struct Bus {
  pub(super) wram: Memory,
  pub(super) mapper: Box<MapperType>,
  pub ppu_mem: PPUMemory,
  pub(super) cpu_mapped_reg: Memory,
  oam_dma: (bool, u8, u8),
  pub(super) input: Box<dyn Controller>,
  //ppu: PPU,
  //apu: APU,
  //input: Input,
  //mapper: Mapper,
}

impl Bus {
  pub fn new(input: Box<dyn Controller>) -> Self {
    Self {
      wram: Memory::ram(0x0800),
      ppu_mem: PPUMemory::new(),
      cpu_mapped_reg: Memory::ram(0x2F),
      mapper: Box::new(mapper::null()),
      oam_dma: (false, 0, 0),
      input,
    }
  }

  pub fn load_mapper(&mut self, mapper: MapperType) {
    self.mapper = Box::new(mapper);
    self.ppu_mem.set_mirroring(self.mapper.mirroring());
  }

  pub fn print_wram(&self) {
    println!("{}", self.wram);
  }

  pub fn print_ppu_mem(&self) {
    println!("{}", self.ppu_mem);
  }

  pub fn get_oam_dma_state(&self) -> bool {
    self.oam_dma.0
  }

  pub fn oam_dma_tick(&mut self) -> bool {
    let addr: u16 = (((self.oam_dma.1) as u16) << 8) | (self.oam_dma.2 as u16);
    let value = self.read(addr.into());
    self.ppu_mem.oam_dma_write(value);
    if (self.oam_dma.2 == 0xFF) {
      self.oam_dma.0 = false;
    }
    else {
      self.oam_dma.2 += 1;
    }
    self.oam_dma.0
  }
}


impl MemRead for Bus {
  fn read(&mut self, addr: usize) -> u8 {
    match addr {
      0x0000..=0x0800 => self.wram.read(addr),
      (0x2000..=0x2007) | 0x4014 => self.ppu_mem.read(addr),
      0x4016 | 0x4017 => self.input.read(addr),
      0x4000..=0x401F => self.cpu_mapped_reg.read(addr),
      0x4020..=0xFFFF => self.mapper.read(addr),
      _ => 0,
    }
  }
}

impl MemWrite for Bus {
  fn write(&mut self, addr: usize, value: u8) {
    let addr16 = addr as u16;
    match addr16 {
      0x0000..=0x0800 => self.wram.write(addr, value),
      OAMDMA_CPU_ADDR => {self.oam_dma = (true, value, 0x00)},
      0x2000..=0x2007 => self.ppu_mem.write(&mut self.mapper, addr, value),
      0x4016 | 0x4017 => self.input.write(addr, value),
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
      0x2000..=0x3FFF => self.ppu_mem.ppu_read(&mut self.mapper, addr),
      _ => 0,
    }
  }

  pub fn ppu_write(&mut self, addr: usize, value: u8) {
    match addr {
      0x0000..=0x1FFF => self.mapper.write(addr, value),
      0x2000..=0x3FFF => self.ppu_mem.ppu_write(&mut self.mapper, addr, value),
      _ => (),
    }
  }
}
