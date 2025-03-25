//use std::error::Error;
use std::fmt;

use crate::nes::{
  mapper::MirroringType,
};

#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum ConsoleType {
  NES,
  VsSystem{ppu_type: u8, hardware_type: u8},
  PC10,
  Extended(u8),
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum TimingType {
  NTSC_NES,
  PAL_NES,
  MUL_REG,
  Dendy,
}

#[derive(Debug)]
pub struct NesHeader {
  pub prg_rom_size: usize,
  pub chr_rom_size: usize,

  pub mirroring_type: MirroringType,
  pub battery: bool,
  pub trainer: bool,
  pub mapper_num: u16,

  pub console_type: ConsoleType,
  pub nes2: bool,

  pub submapper_num: u8,

  pub eeprom_size: usize,
  pub prg_ram_size: usize,
  pub chr_ram_size: usize,
  pub chr_nvram_size: usize,

  pub timing_type: TimingType,
  pub misc_roms: u8,
  pub default_exp_device: u8,
}

impl fmt::Display for NesHeader {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    writeln!(f, "PRG-ROM size: {:#06x}", self.prg_rom_size).unwrap();
    writeln!(f, "CHR-ROM size: {:#06x}", self.chr_rom_size).unwrap();

    writeln!(f, "Mirroring_type: {}", self.mirroring_type).unwrap();
    writeln!(f, "Battery: {}", self.battery).unwrap();
    writeln!(f, "Trainer: {}", self.trainer).unwrap();
    writeln!(f, "Mapper num: {}", self.mapper_num).unwrap();

    writeln!(f, "Console type: {:?}", self.console_type).unwrap();
    writeln!(f, "Nes2: {}", self.nes2).unwrap();

    writeln!(f, "Submapper num: {}", self.submapper_num).unwrap();

    writeln!(f, "EEPROM size: {:#06x}", self.eeprom_size).unwrap();
    writeln!(f, "PRG-RAM size: {:#06x}", self.prg_ram_size).unwrap();
    writeln!(f, "CHR-RAM size: {:#06x}", self.chr_ram_size).unwrap();
    writeln!(f, "CHR-NVRRAM size: {:#06x}", self.chr_nvram_size).unwrap();

    writeln!(f, "Timing type: {:?}", self.timing_type).unwrap();
    writeln!(f, "Misc. roms: {}", self.misc_roms).unwrap();
    writeln!(f, "Expension device: {}", self.default_exp_device).unwrap();
    Ok(())
  }
}

impl NesHeader {
  pub fn new(header: &[u8]) -> Self {
    Self {
      prg_rom_size: {
        if (header[9] & 0b00001111) == 0xF {
          (2 << (((header[4] & 0b1111_1100) as usize) >> 2)) * ((header[4] & 0b0000_0011) * 2 + 1) as usize
        }
        else {
          ((((header[9] & 0b00001111) as usize) << 8) | (header[4] as usize)) << (4 + 10)
        }
      },
      chr_rom_size: {
        if ((header[9] & 0b11110000) >> 4) == 0xF {
          (2 << (((header[5] & 0b1111_1100) as usize) >> 2)) * ((header[5] & 0b0000_0011) * 2 + 1) as usize
        }
        else {
          ((((header[9] & 0b11110000) as usize) << 4) | (header[5] as usize)) << (3 + 10)
        }
      },
          

      //prg_rom_size: (header[4] as usize) * 16 * 1024,
      //chr_rom_size: (header[5] as usize) * 8 * 1024,
      mirroring_type: {
        let mut m_type = MirroringType::Horizontal;
        if header[6] & 0b0000_0001 != 0 {
          m_type = MirroringType::Vertical;
        } 
        if header[6] & 0b0000_1000 != 0 {
          m_type = MirroringType::FourScreen;
        }
        m_type
      },
      battery: if header[6] & 0b0000_0010 != 0 {true} else {false},
      trainer: if header[6] & 0b0000_0100 != 0 {true} else {false},
      mapper_num: {
        ((header[6] >> 4) as u16)
          | ((header[7] & 0b1111_0000) as u16)
          | (((header[8] & 0b0000_1111) as u16) << 8)
      },
      console_type: {
        match header[7] & 0b0000_0011 {
          0 => ConsoleType::NES,
          1 => ConsoleType::VsSystem{ppu_type: header[13] & 0b0000_1111, hardware_type: (header[13] & 0b1111_0000) >> 4},
          2 => ConsoleType::PC10,
          3 => ConsoleType::Extended(header[13] & 0b0000_1111),
          _ => ConsoleType::NES,
        }
      },
      nes2: if header[7] & 0b0000_1100 == 8 {true} else {false},
      submapper_num: (header[8] & 0b1111_0000) >> 4,
      eeprom_size: if header[10] & 0b0000_1111 == 0 {0} else {64 << ((header[10] & 0b0000_1111) as usize)},
      prg_ram_size: if header[10] & 0b1111_0000 == 0 {0} else {64 << (((header[10] & 0b1111_0000) >> 4) as usize)},
      chr_ram_size: if header[11] & 0b0000_1111 == 0 {0} else {64 << ((header[11] & 0b0000_1111) as usize)},
      chr_nvram_size: if header[11] & 0b1111_0000 == 0 {0} else {64 << (((header[11] & 0b1111_0000) >> 4) as usize)},
      timing_type: {
        match header[12] & 0b0000_0011 {
          0 => TimingType::NTSC_NES,
          1 => TimingType::PAL_NES,
          2 => TimingType::MUL_REG,
          3 => TimingType::Dendy,
          _ => TimingType::NTSC_NES,
        }
      },
      misc_roms: header[14] & 3,
      default_exp_device: header[15] & 0b0011_1111,
    }
  }
}

pub struct Cartridge {
  pub header: NesHeader,
  pub prg_rom : Vec<u8>,
  pub prg_size : usize,
  pub chr_rom : Option<Vec<u8>>,
  pub chr_size : usize,
}

impl Cartridge {
  pub fn create_from_rom(rom: &Vec<u8>) -> Self {
    let header = NesHeader::new(&rom[..16]);
    let cart = Cartridge {
      //prg_rom: rom[rom_info.start_prg..(rom_info.start_prg + rom_info.size_prg)].to_vec(),
      prg_rom: Self::prg_rom_vec(rom, &header),
      prg_size: header.prg_rom_size,
      chr_rom: if header.chr_rom_size == 0 {None} else {Some(Self::chr_rom_vec(rom, &header))},
      chr_size: header.prg_rom_size,
      header,
    };
    println!("");
    println!("{}", cart.header);
    cart
  }

  fn prg_rom_vec(rom: &Vec<u8>, header: &NesHeader) -> Vec<u8> {
    let start: usize = 16;
    rom[start..(start + (header.prg_rom_size as usize))].to_vec()
  }

  fn chr_rom_vec(rom: &Vec<u8>, header: &NesHeader) -> Vec<u8> {
    let start: usize = 16 + (header.prg_rom_size as usize);
    rom[start..(start + (header.chr_rom_size as usize))].to_vec()
  }
}
