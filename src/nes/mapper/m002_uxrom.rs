use std::fmt;

use crate::Cartridge;
use crate::nes::{
  memory::{MemRead, MemWrite, Memory},
  mapper::{Mapper, MapperType, MirroringType},
};

//const PRG_RAM_SIZE

#[derive(Debug, Clone)]
pub struct Uxrom {
  prg_ram: Memory,
  prg_rom: Memory,
  chr_rom: Memory,
  //prg_rom_size: usize,
  mirroring: MirroringType,
}

impl fmt::Display for Uxrom {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl Uxrom {
  pub fn load(cartridge: &Cartridge) -> MapperType {
    let uxrom = Self {
      prg_ram: Memory::ram(8 * 1024),
      prg_rom: Memory::rom_from_bytes(&cartridge.prg_rom),
      chr_rom: {match &cartridge.chr_rom {
        Some(chr_rom) => {
          println!();
          Memory::rom_from_bytes(&chr_rom)
        },
        None => (Memory::new())
      }},
      mirroring: cartridge.header.mirroring_type,
    };
    uxrom.into()
  }
}

impl Mapper for Uxrom {
  fn debug_print_vec(&mut self) {
    for v in 0xfff0..=0xffff {
      println!("{:#06x} = {:#04x}", v, self.prg_rom.read(v));
    }
  }

  fn mirroring(&self) -> MirroringType {
    println!("{}", self.mirroring);
    self.mirroring
  }
  fn irq_pending(&mut self) -> bool {
    false
  }
  fn battery_backed(&self) -> bool {
    false
  }
  fn use_ciram(&self, _addr: usize) -> bool {
    true
  }
  fn nametable_page(&self, _addr: usize) -> usize {
    0
  }
  fn ppu_write(&mut self, _addr: usize, _val: u8) {}
  fn open_bus(&mut self, _addr: usize, _val: u8) {}
}

impl MemRead for Uxrom {
  fn read(&mut self, addr: usize) -> u8 {
    match addr {
      0x0000..=0x1FFF => self.chr_rom.read(addr),
      0x6000..=0x7FFF => self.prg_ram.read(addr),
      0x8000..=0xFFFF => self.prg_rom.read(addr),
      _ => 0,
    }
  }
}

impl MemWrite for Uxrom {
  fn write(&mut self, addr: usize, value: u8) {
    match addr {
      0x0000..=0x1FFF => self.chr_rom.write(addr, value),
      0x8000..=0xFFFF => self.prg_rom.write(addr, value),
      _ => (),
    }
  }
}
