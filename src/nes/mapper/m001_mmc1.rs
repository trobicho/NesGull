use std::fmt;

use crate::Cartridge;
use crate::nes::{
  memory::{MemRead, MemWrite, BankableMemory},
  mapper::{Mapper, MapperType, MirroringType},
};

const PRG_RAM_WINDOW: usize = 8 * 1024;
const PRG_RAM_SIZE: usize = 8 * 1024;
const PRG_ROM_WINDOW: usize = 16 * 1024;
const CHR_WINDOW: usize = 4 * 1024;
const CHR_SIZE: usize = 8 * 1024;

#[derive(Debug, Clone)]
pub struct MMC1 {
  prg_ram: BankableMemory,
  prg_rom: BankableMemory,
  chr: BankableMemory,
  mirroring: MirroringType,
  shift_reg: u8,
  control: u8,
  chr_bank0: u8,
  chr_bank1: u8,
  prg_bank: u8,
}

impl fmt::Display for MMC1 {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl MMC1 {
  pub fn load(cartridge: &Cartridge) -> MapperType {
    let mut mmc1 = Self {
      prg_ram: {
        if cartridge.header.prg_ram_size == 0 {
          BankableMemory::ram(PRG_RAM_SIZE, PRG_RAM_WINDOW)
        }
        else {
          BankableMemory::ram(cartridge.header.prg_ram_size, PRG_RAM_WINDOW)
        }
      },
      prg_rom: BankableMemory::rom_from_bytes(&cartridge.prg_rom, PRG_ROM_WINDOW),
      chr: {match &cartridge.chr_rom {
        Some(chr_rom) => {
          BankableMemory::ram_from_bytes(&chr_rom, CHR_WINDOW)
        },
        None => BankableMemory::ram(CHR_SIZE, CHR_WINDOW)
      }},
      mirroring: cartridge.header.mirroring_type,
      shift_reg: 0b0001_0000,
      control: 0,
      chr_bank0: 0,
      chr_bank1: 0,
      prg_bank: 0,
    };
    mmc1.prg_ram.add_bank_range(0x6000, 0x7FFF);
    mmc1.prg_rom.add_bank_range(0x8000, 0xFFFF);
    mmc1.prg_rom.set_bank(0xC000, mmc1.prg_rom.last_bank());
    mmc1.chr.add_bank_range(0x0000, 0x1FFF);
    mmc1.control_write(0b11110);
    mmc1.into()
  }
}

impl Mapper for MMC1 {
  fn debug_print_vec(&mut self) {
    for v in 0xfff0..=0xffff {
      println!("{:#06x} = {:#04x}", v, self.prg_rom.read(v));
    }
  }

  fn mirroring(&self) -> MirroringType {
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

impl MemRead for MMC1 {
  fn read(&mut self, addr: usize) -> u8 {
    match addr {
      0x0000..=0x1FFF => self.chr.read(addr),
      0x6000..=0x7FFF => {
        if self.prg_bank & 0b10000 == 0 {
          self.prg_ram.read(addr)
        }
        else {
          0
        }
      },
      0x8000..=0xBFFF => self.prg_rom.read(addr),
      0xC000..=0xFFFF => self.prg_rom.read(addr),
      _ => 0,
    }
  }
}

impl MemWrite for MMC1 {
  fn write(&mut self, addr: usize, value: u8) {
    //println!("write {:#06x} = ({:#010b})", addr, value);
    match addr {
      0x0000..=0x1FFF => self.chr.write(addr, value),
      0x6000..=0x7FFF => {
        //println!("prg_ram");
        if self.prg_bank & 0b10000 == 0 {
          self.prg_ram.write(addr, value);
        }
      },
      0x8000..=0xFFFF => {
        if value & 0b1000_0000 != 0 {
            self.shift_reg = 0b0001_0000;
            self.control_write(self.control | 0xC);
        }
        else {
          if self.shift_reg & 1 == 1 {
            let v: u8 = self.shift_reg.wrapping_shr(1) | ((value & 1) << 4);
            match addr & 0b01110_0000_0000_0000 {
              0x8000 => {print!("addr "); self.control_write(v)},
              0xA000 => self.chr_bank_write(v, false),
              0xC000 => self.chr_bank_write(v, true),
              0xE000 => self.prg_bank_write(v),
              _ => (),
            }
            self.shift_reg = 0b0001_0000;
          }
          else {
            self.shift_reg = self.shift_reg.wrapping_shr(1) | ((value & 1) << 4);
          }
        }
      },
      _ => (),
    }
    //println!(" control: {:#07b}", self.control);
  }
}

impl MMC1 {
  fn control_write(&mut self, v: u8) {
    self.control = v & 0b0001_1111;
    match self.control & 3 {
      0 => self.mirroring = MirroringType::SingleScreenA,
      1 => self.mirroring = MirroringType::SingleScreenB,
      2 => self.mirroring = MirroringType::Vertical,
      3 => self.mirroring = MirroringType::Horizontal,
      _ => (),
    }
    println!("mirroring change to: {}", self.mirroring);
    match self.control & 0xC {
      0 | 0x4 => {
        let bank_n = self.prg_bank & 0b0001_1110;
        self.prg_rom.set_bank(0x8000, bank_n.into());
        self.prg_rom.set_bank(0x8000, (bank_n | 1).into());
      },
      0x8 => { 
        let bank_n = self.prg_bank & 0b0001_1111;
        self.prg_rom.set_bank(0x8000, 0);
        self.prg_rom.set_bank(0xC000, bank_n.into());
      }
      0xC => { 
        let bank_n = self.prg_bank & 0b0001_1111;
        self.prg_rom.set_bank(0x8000, bank_n.into());
        self.prg_rom.set_bank(0xC000, self.prg_rom.last_bank());
      }
      _ => (),
    }
    match self.control & 0x10 {
      0x0 => {
        let bank_n = self.chr_bank0 & 0b0001_1110;
        self.chr.set_bank(0x0000, bank_n.into());
        self.chr.set_bank(0x1000, (bank_n | 1).into());
      },
      0x10 => {
        let bank_n = self.chr_bank0 & 0b0001_1111;
        self.chr.set_bank(0x0000, bank_n.into());
        let bank_n = self.chr_bank1 & 0b0001_1111;
        self.chr.set_bank(0x1000, bank_n.into());
      }
      _ => (),
    }
  }

  fn chr_bank_write(&mut self, v: u8, bank1: bool) {
    if !bank1 {
      self.chr_bank0 = v & 0b0001_1111;
      let bank_n = self.chr_bank0 & (if self.control & 0x10 == 0x10 {0b11110} else {0b11111});
      self.chr.set_bank(0x0000, bank_n.into());
      if self.control & 0x10 == 0x10 {
        self.chr.set_bank(0x1000, (bank_n | 1).into());
      }
    }
    else {
      self.chr_bank1 = v & 0b0001_1111;
      if self.control & 0x10 == 0 {
        self.chr.set_bank(0x1000, self.chr_bank1.into());
      }
    }
  }

  fn prg_bank_write(&mut self, v: u8) {
    self.prg_bank = v & 0b0001_1111;
    match self.control & 0xC {
      0 | 0x4 => {
        let bank_n = self.prg_bank & 0b11110;
        self.prg_rom.set_bank(0x8000, bank_n.into());
        self.prg_rom.set_bank(0xC000, (bank_n | 1).into());
      },
      0x8 => {
        let bank_n = self.prg_bank;
        self.prg_rom.set_bank(0xC000, bank_n.into());
      },
      0xC => {
        let bank_n = self.prg_bank;
        self.prg_rom.set_bank(0x8000, bank_n.into());
      },
      _ => (),
    }
  }
}
