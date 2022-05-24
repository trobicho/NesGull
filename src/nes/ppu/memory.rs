use std::fmt;

use crate::nes::{
  memory::{Memory, MemRead, MemWrite},
  mapper::{MirroringType, MapperType},
};

const NAMETABLE_ADDR: u16 = 0x2000;

const PPUCTRL_CPU_ADDR: u16 = 0x2000;
const PPUMASK_CPU_ADDR: u16 = 0x2001;
const PPUSTATUS_CPU_ADDR: u16 = 0x2002;
const OAMADDR_CPU_ADDR: u16 = 0x2003;
const OAMDATA_CPU_ADDR: u16 = 0x2004;
const PPUSCROLL_CPU_ADDR: u16 = 0x2005;
const PPUADDR_CPU_ADDR: u16 = 0x2006;
const PPUDATA_CPU_ADDR: u16 = 0x2007;
pub const OAMDMA_CPU_ADDR: u16 = 0x4014;

#[derive(Debug, Copy, Clone)]
pub struct Oam {
  pub y: u8,
  pub tile: u8,
  pub attr: u8,
  pub x: u8,
  pub is_sprite0: bool,
}

impl Oam {
  pub fn new() -> Self {
    Self {
      y: 0xFF,
      tile: 0xFF,
      attr: 0xFF,
      x: 0xFF,
      is_sprite0: false,
    }
  }
}

#[derive(Debug)]
pub struct PPUMemory {
  ctrl: u8,
  mask: u8,
  status: u8,
  pub v: u16,
  pub t: u16,
  pub x: u8,
  w: bool, 
  open_bus: u8,

  pub oam_addr: u8,
  oam_dma: u8,

  vram: Memory,
  oam: Memory,
  palette: Memory,

  nmi_output: bool,
  mirroring_type: MirroringType,
}

impl PPUMemory {
  pub fn new() -> Self {
    Self {
      ctrl: 0x00,
      mask: 0x00,
      status: 0x00,
      v: 0,
      t: 0,
      x: 0x00,
      w: false,
      open_bus: 0,

      oam_addr: 0,
      oam_dma: 0,
      vram: Memory::ram(0x1000),
      oam: Memory::ram(256),
      palette: Memory::ram(0x20),
      nmi_output: true,
      mirroring_type: MirroringType::Horizontal,
    }
  }
}

impl fmt::Display for PPUMemory {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl PPUMemory {
  pub fn read_ctrl(&mut self) -> u8 {
    self.ctrl
  }

  pub fn read_mask(&mut self) -> u8 {
    self.mask
  }

  pub fn read_status(&mut self) -> u8 { //Debug
    self.status
  }

  pub fn nmi(&mut self) {
    self.nmi_output = false;
  }

  pub fn set_interupt(&mut self, bit: bool) {
    match bit {
      true => self.status |= 0b1000_0000,
      false => self.status &= 0b0111_1111,
    }
  }

  pub fn set_sprite_overflow(&mut self, bit: bool) {
    match bit {
      true => self.status |= 0b0010_0000,
      false => self.status &= 0b1101_1111,
    }
  }

  pub fn set_sprite_0hit(&mut self, bit: bool) {
    match bit {
      true => self.status |= 0b0100_0000,
      false => self.status &= 0b1011_1111,
    }
  }

  pub fn get_nmi_output(&mut self) -> bool{
    self.nmi_output
  }

  pub fn oam_dma_write(&mut self, value: u8) {
    self.oam.write(self.oam_addr.into(), value);
    self.oam_addr = self.oam_addr.wrapping_add(1);
  }

  pub fn set_mirroring(&mut self, mirroring_type: MirroringType) {
    self.mirroring_type = mirroring_type;
    println!("{}", self.mirroring_type);
  }

  fn mirroring(&self, addr: usize) -> usize {
    let mut addr = addr;
    match addr {
      0x2000..=0x2FFF => {addr -= 0x2000;},
      0x3000..=0x3EFF => {addr -= 0x3000;},
      _ => {return addr;},
    }
    match self.mirroring_type {
      MirroringType::Vertical => {addr %= 0x800;},
      MirroringType::Horizontal => if (addr >= 0x400 && addr < 0x800) || addr >= 0xC00 {addr -= 0x400},
      MirroringType::FourScreen => (),
      MirroringType::SingleScreenA => {addr %= 0x400;},
      MirroringType::SingleScreenB => {addr %= 0x400; addr |= 0x400},
    }
    addr
  }
}

impl MemRead  for PPUMemory {
  fn read(&mut self, addr: usize) -> u8 {
    let addr = addr as u16;
    match addr {
      PPUSTATUS_CPU_ADDR => {
        self.w = false;
        let v = self.status;
        self.set_interupt(false);
        //print!(" PPUSTATUS({:#010b} {:#010b})", v, self.status);
        v
      },
      OAMDATA_CPU_ADDR => {
        self.oam.read(self.oam_addr.into())
      }
      PPUDATA_CPU_ADDR => {
        let value = self.vram.read(self.mirroring(self.v.into()));
        if self.ctrl & 0b0000_0010 != 0 {
          self.v += 32;
        }
        else {
          self.v += 1;
        }
        value
      },
      _ => 0,
    }
  }
}

impl PPUMemory {
  pub fn write(&mut self, mapper: &mut MapperType, addr: usize, value: u8) {
    let addr = addr as u16;
    match addr {
      PPUCTRL_CPU_ADDR => {
        self.ctrl = value & 0b1111_1100;
        self.nmi_output = if value & 0b1000_0000 != 0 {true} else {false};
        self.t = (self.t & 0b1111_0011_1111_1111) | (((value & 0b0000_0011) as u16) << 10);
      }
      PPUMASK_CPU_ADDR => self.mask = value,
      OAMADDR_CPU_ADDR => self.oam_addr = value,
      OAMDATA_CPU_ADDR => {self.oam.write(self.oam_addr.into(), value); self.oam_addr.wrapping_add(1);},
      PPUSCROLL_CPU_ADDR => {
        if self.w {
          self.t = (self.t & 0b0000_1100_0001_1111)
            | (((value & 0b0000_0111) as u16) << 12)
            | (((value & 0b0011_1000) as u16) << 2)
            | (((value & 0b1100_0000) as u16) << 2);
        }
        else {
          self.x = value & 0b0000_0111;
          self.t = (self.t & 0b0011_1111_1110_0000) | ((value & 0b1111_1000) >> 3) as u16;
        }
        self.w = !self.w;
      },
      PPUADDR_CPU_ADDR => {
        if self.w == false{
          self.t = (self.t & 0b0000_0000_1111_1111) | ((value & 0b0111_1111) as u16).wrapping_shl(8);
        }
        else {
          self.t = (self.t & 0b0011_1111_0000_0000) | ((value & 0b1111_1111) as u16);
          self.v = self.t;
          //print!(" PPUADDR: {:#06x} {:#04x} ", self.v, value);
        }
        self.w = !self.w;
      },
      PPUDATA_CPU_ADDR => {
        self.ppu_write(mapper, self.v.into(), value);
        //self.vram.write((self.v % 0x2000).into(), value);
        //print!(" PPUDATA: {:#06x} {} ", self.v, value);
        if self.ctrl & 0b0000_0100 != 0 {
          self.v += 32;
        }
        else {
          self.v += 1;
        }
      }
      _ => (),
    }
    self.status |= (value & 0b0001_1111)
    //println!("{}", self);
  }
}

impl PPUMemory {
  pub fn oam_read(&mut self, n: usize, offset: usize) -> u8 {
    let addr = (n % 64) * 4 + offset;
    self.oam.read(addr)
  }

  pub fn ppu_read(&mut self, mapper: &mut MapperType, addr: usize) -> u8 {
    match addr {
      0x0000..=0x1FFF => mapper.read(addr),
      0x2000..=0x2FFF => self.vram.read(self.mirroring(addr)),
      0x3000..=0x3EFF => self.vram.read(self.mirroring(addr)),
      0x3F10 => self.palette.read(0x00),
      0x3F14 => self.palette.read(0x04),
      0x3F18 => self.palette.read(0x08),
      0x3F1C => self.palette.read(0x0C),
      0x3F00..=0x3FFF => self.palette.read(addr & 0xFF),
      _ => 0,
    }
 }

 pub fn ppu_write(&mut self, mapper: &mut MapperType, addr: usize, value: u8) {
    match addr {
      0x0000..=0x1FFF => mapper.write(addr, value),
      0x2000..=0x2FFF => self.vram.write(self.mirroring(addr), value),
      0x3000..=0x3EFF => self.vram.write(self.mirroring(addr), value),
      0x3F10 => self.palette.write(0x00, value),
      0x3F14 => self.palette.write(0x04, value),
      0x3F18 => self.palette.write(0x08, value),
      0x3F1C => self.palette.write(0x0C, value),
      0x3F00..=0x3FFF => self.palette.write(addr & 0xFF, value),
      _ => (),
    }
  }

  pub fn print_oam(&mut self) {
    print!("OAM: ");
    for v in 0..self.oam.len() {
      print!("{:#04x},", self.oam.read(v));
    }
    println!("");
  }

  pub fn print_palette(&mut self) {
    println!("Palette: ");
    for v in 0..self.palette.len() {
      println!("{:#04x},", self.palette.read(v));
    }
  }
}

