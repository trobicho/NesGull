mod cpu;
mod ppu;

use std::error::Error;
use crate::arch::cpu::CPU;
use crate::arch::ppu::PPU;

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

pub struct Cartridge<'rom> {
  full_rom: &'rom[u8],
  header: &'rom[u8],
  prg_rom : &'rom[u8],
  prg_size : usize,
  chr_rom : Option<&'rom[u8]>,
  chr_size : usize,
}

impl<'rom> Cartridge<'rom> {
  pub fn create_from_rom(rom: &'rom Vec<u8>) -> Self {
    let rom_info = RomInfo::info_from_header(&rom[..16]);
    Cartridge {
      full_rom: rom,
      header: &rom[..16],
      prg_rom: &rom[rom_info.start_prg..(rom_info.start_prg + rom_info.size_prg)],
      prg_size: rom_info.size_prg,
      chr_rom: if rom_info.size_chr > 0 {
          Some(&rom[rom_info.start_chr..(rom_info.start_chr + rom_info.size_chr)])
        } else {None},
      chr_size: rom_info.size_chr,
    }
  }
}

pub struct Nes<'rom> {
  cpu : CPU,
  ppu : PPU,
  cartridge : &'rom Cartridge<'rom>,
}

impl<'rom> Nes<'rom> {
  pub fn load_rom(cartridge: &'rom Cartridge) -> Self {
    Self {
      cpu: CPU::new(),
      ppu: PPU::new(),
      cartridge,
    }
  }

  pub fn load_palette(&mut self, filename: &str) -> Result<(), Box<dyn Error>> {
    self.ppu.load_palette(filename)
  }

  pub fn reset(&mut self) {
    self.cpu.load(self.cartridge.prg_rom, self.cartridge.prg_size);
    self.cpu.reset();
    match self.cartridge.chr_rom {
      Some(chr_rom) => {self.ppu.load(chr_rom, self.cartridge.chr_size)},
      None => {}
    }
    self.ppu.reset();
  }

  pub fn show_mem(&self) {
    self.cpu.show_mem();
    println!("==============================");
    self.ppu.show_mem();
  }

  pub fn run(&mut self) {
    for _ in 0..10 {
      self.cpu.debug_read_instr();
    }
  }
}
