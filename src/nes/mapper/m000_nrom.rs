use crate::Cartridge;
use crate::nes::{
  memory::{MemRead, MemWrite, Memory},
};

//const PRG_RAM_SIZE

pub struct Nrom {
  prg_ram: Memory,
  prg_rom: Memory,
  chr_rom: Memory,
  //prg_rom_size: usize,
}

impl Nrom {
  pub fn load(cartridge: &Cartridge) -> Self {
    Self {
      prg_ram: Memory::ram(8 * 1024),
      prg_rom: Memory::rom_from_bytes(&cartridge.prg_rom),
      chr_rom: {match &cartridge.chr_rom {
        Some(chr_rom) => Memory::rom_from_bytes(&chr_rom),
        None => (Memory::new())
      }}
    }
  }
}

impl MemRead for Nrom {
  fn read(&mut self, addr: usize) -> u8 {
    match addr {
      0x0000..=0x1FFF => self.chr_rom.read(addr),
      0x6000..=0x7FFF => self.prg_ram.read(addr),
      0x8000..=0xFFFF => self.prg_rom.read(addr),
      _ => 0,
    }
  }
}

impl MemWrite for Nrom {
  fn write(&mut self, addr: usize, value: u8) {
    match addr {
      0x0000..=0x1FFF => self.chr_rom.write(addr, value),
      0x8000..=0xFFFF => self.prg_rom.write(addr, value),
      _ => (),
    }
  }
}
