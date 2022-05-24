use std::fmt;

use crate::Cartridge;
use crate::nes::{
  memory::{MemRead, MemWrite, Memory, BankableMemory},
  mapper::{Mapper, MapperType, MirroringType},
};

const PRG_ROM_WINDOW: usize = 16 * 1024;
const CHR_WINDOW: usize = 8 * 1024;
const CHR_SIZE: usize = 8 * 1024;

//const PRG_RAM_SIZE

#[derive(Debug, Clone)]
pub struct Uxrom {
  prg_rom: BankableMemory,
  chr: BankableMemory,
  mirroring: MirroringType,
}

impl fmt::Display for Uxrom {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl Uxrom {
  pub fn load(cartridge: &Cartridge) -> MapperType {
    let mut uxrom = Self {
      prg_rom: BankableMemory::rom_from_bytes(&cartridge.prg_rom, PRG_ROM_WINDOW),
      chr: {match &cartridge.chr_rom {
        Some(chr_rom) => {
          BankableMemory::ram_from_bytes(&chr_rom, CHR_WINDOW)
        },
        None => (BankableMemory::ram(CHR_SIZE, CHR_WINDOW))
      }},
      mirroring: cartridge.header.mirroring_type,
    };
    uxrom.prg_rom.add_bank_range(0x8000, 0xFFFF);
    uxrom.prg_rom.set_bank(0xC000, uxrom.prg_rom.last_bank());
    uxrom.chr.add_bank_range(0x0000, 0x1FFF);
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
      0x0000..=0x1FFF => self.chr.read(addr),
      0x8000..=0xBFFF => self.prg_rom.read(addr),
      0xC000..=0xFFFF => self.prg_rom.read(addr),
      _ => 0,
    }
  }
}

impl MemWrite for Uxrom {
  fn write(&mut self, addr: usize, value: u8) {
    match addr {
      0x0000..=0x1FFF => self.chr.write(addr, value),
      0x8000..=0xFFFF => {
        let v = value & 0b0000_1111;
        self.prg_rom.set_bank(0x8000, v.into());
      },
      _ => (),
    }
  }
}
