mod palette;
mod register;

use crate::nes::{
  memory::{MemRead, MemWrite, Memory},
  bus::Bus,
  clock::Clock,
};
use palette::{Palette, NesColor};
use register::*;

use std::error::Error;

const FRAME_HEIGHT_NTSC: usize = 224;
const FRAME_HEIGHT_PAL: usize = 240;
const FRAME_WIDTH: usize = 256;
const PATTERN_TABLE_ADDR: u16 = 0x0000;
const NAMETABLE_ADDR: u16 = 0x2000;

const PPUCTRL_CPU_ADDR: u16 = 0x2000;
const PPUMASK_CPU_ADDR: u16 = 0x2001;
const PPUSTATUS_CPU_ADDR: u16 = 0x2002;
const OAMDDR_CPU_ADDR: u16 = 0x2003;
const OAMDATA_CPU_ADDR: u16 = 0x2004;
const PPUSCROLL_CPU_ADDR: u16 = 0x2005;
const PPUADDR_CPU_ADDR: u16 = 0x2006;
const PPUDATA_CPU_ADDR: u16 = 0x2007;
const OAMDMA_CPU_ADDR: u16 = 0x4014;

pub struct Frame {
  pub width: usize,
  pub height: usize,
  pixels: Vec<u8>,
}

impl Frame {
  pub fn new(width: usize, height: usize) -> Self {
    Frame {
      width,
      height,
      pixels: vec![0; width * height * 4],
    }
  }

  pub(super) fn put_pixel(&mut self, x: usize, y: usize, color: NesColor) {
    self.pixels[((self.width * y + x) << 2) + 3] = 255;
    self.pixels[((self.width * y + x) << 2) + 2] = color.R;
    self.pixels[((self.width * y + x) << 2) + 1] = color.G;
    self.pixels[((self.width * y + x) << 2) + 0] = color.B;
  }

  pub fn get_texture_buffer(&self) -> &Vec<u8> {
    &self.pixels
  }
}

#[derive(Debug, Copy, Clone)]
pub struct PPUInfo {
  pub frame_w: usize,
  pub frame_h: usize,
}

pub struct Oam {
  y: u8,
  tile: u8,
  attr: u8,
  x: u8
}

pub struct PPU {
  palette: Palette,
  frame: Frame,
  oam: Memory,
  palette_mem: Memory,
  scanline_n : u32,
  cycle_n: u32,
  rendering_enable: bool,
  reg: Register,
}

impl PPU {
  pub fn new() -> Self {
    Self {
      palette: Palette::new(),
      frame: Frame::new(FRAME_WIDTH, FRAME_HEIGHT_NTSC),
      oam: Memory::ram(256),
      palette_mem: Memory::ram(32),
      scanline_n : 261,
      cycle_n: 0,
      rendering_enable: true,
      reg: Register::new(),
    }
  }

  pub fn reset(&mut self) {
  }

  pub fn load_palette(&mut self, filename: &str) -> Result<(), Box<dyn Error>> {
    self.palette.change_from_file(filename)
  }

  pub fn render_frame(&mut self, _bus: &mut Bus) -> &Frame {
    for y in 0..self.frame.height {
      for x in 0..self.frame.width {
        let palette_index: u32 = ((x as f32 / self.frame.width as f32) * 16.0) as u32
                            + ((y as f32 / self.frame.height as f32) * 4.0) as u32 * 16;
        let color = self.palette.color[palette_index as usize];
        self.frame.put_pixel(x, y, color);
      }
    }
    &self.frame
  }

  //DCBA98 76543210
  //---------------
  //0HRRRR CCCCPTTT
  //|||||| |||||+++- T: Fine Y offset, the row number within a tile
  //|||||| ||||+---- P: Bit plane (0: "lower"; 1: "upper")
  //|||||| ++++----- C: Tile column
  //||++++---------- R: Tile row
  //|+-------------- H: Half of sprite table (0: "left"; 1: "right")
  //+--------------- 0: Pattern table is at $0000-$1FFF
  pub fn feed_shift_back(&mut self, bus: &mut Bus){
    let fine_y: u16 = (self.scanline_n % 8) as u16;
    let pattern_n = bus.read(self.reg.vram_cur as usize);
    let mut pattern_addr: u16 = ((pattern_n & 0b1111_1111)as u16).wrapping_shl(4);

    pattern_addr += fine_y;
    let r = bus.ppu_read(pattern_addr.into());
    self.reg.load_back_upper(r, 0);
    pattern_addr += 8;

    let r = bus.ppu_read(pattern_addr.into());
    self.reg.load_back_upper(r, 1);
  }

  pub fn scanline_vblank(&mut self, bus: &mut Bus) {
      self.reg.vram_cur = PATTERN_TABLE_ADDR;
      self.reg.vram_temp = PATTERN_TABLE_ADDR;
      if self.cycle_n == 1 {
        bus.write(PPUSTATUS_CPU_ADDR.into(), 0b1000_0000);
      }
  }

  pub fn scanline_prerender(&mut self, bus: &mut Bus) {
    if self.cycle_n == 261 {
      self.reg.vram_cur = 32 * self.scanline_n.wrapping_shr(3) as u16;
      self.feed_shift_back(bus);
    }
    else if self.cycle_n % 8 == 0 {
      self.feed_shift_back(bus);
      self.reg.vram_cur += 1;
    }
  }

  pub fn scanline_render(&mut self, bus: &mut Bus) {
    bus.write(PPUSTATUS_CPU_ADDR.into(), 0b0000_0000);
    let mut color = self.palette.color[0];
    if self.reg.shift_back_16[0] & 0b1 == 1 {
      color = self.palette.color[27];
    }
    self.frame.put_pixel(self.cycle_n as usize, self.scanline_n as usize, color);
    self.reg.shift_background();
  }

  pub fn handle_scanline(&mut self, bus: &mut Bus) -> bool{
    match self.scanline_n {
      261 => {self.scanline_prerender(bus);},
      0..=222 => {
        self.scanline_prerender(bus);
        if self.cycle_n < 256 {
          self.scanline_render(bus);
        }
      }
      _ => {self.scanline_vblank(bus);}
    }
    if self.cycle_n >= 340 {
      self.cycle_n = 0;
      self.scanline_n = self.scanline_n.wrapping_add(1);
      if self.scanline_n >= 261 {
        self.scanline_n = 0;
      }
      true
    }
    else {
      self.cycle_n += 1;
      false
    }
  }

  pub fn get_frame(&self) -> &Frame {
    &self.frame
  }

  pub fn get_cycles_info(&self) -> (u32, u32) {
    (self.scanline_n, self.cycle_n)
  }

  pub fn render_info(&self) -> PPUInfo {
    PPUInfo{frame_w: self.frame.width
      , frame_h: self.frame.height}
  }
}

impl Clock for PPU {
  fn tick(&mut self, bus: &mut Bus) -> bool {
    self.handle_scanline(bus)
  }
}
