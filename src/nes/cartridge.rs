use std::error::Error;

struct RomInfo {
  trainer : bool,
  size_prg : usize,
  size_chr : usize,
  start_prg : usize,
  start_chr : usize,
}

impl RomInfo {
  fn info_from_header(header : &[u8]) -> Self {
    let size_prg = header[4] as usize * 16 * 1024;
    let size_chr = header[5] as usize * 8 * 1024;

    RomInfo {
      trainer : false,
      start_prg : 16,
      start_chr: 16 + size_prg,
      size_prg,
      size_chr,
    }
  }
}

pub struct Cartridge {
  header: [u8; 16],
  pub prg_rom : Vec<u8>,
  pub prg_size : usize,
  pub chr_rom : Option<Vec<u8>>,
  pub chr_size : usize,
}

impl Cartridge {
  pub fn create_from_rom(rom: &Vec<u8>) -> Self {
    let rom_info = RomInfo::info_from_header(&rom[..16]);
    Cartridge {
      header: rom[..16].try_into().unwrap(),
      prg_rom: rom[rom_info.start_prg..(rom_info.start_prg + rom_info.size_prg)].to_vec(),
      prg_size: rom_info.size_prg,
      chr_rom: if rom_info.size_chr > 0 {
          Some(rom[rom_info.start_chr..(rom_info.start_chr + rom_info.size_chr)].to_vec())
        } else {None},
      chr_size: rom_info.size_chr,
    }
  }
}

