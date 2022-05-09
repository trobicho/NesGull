mod palette;

use crate::nes::{
  memory::{MemRead, MemWrite, Memory},
  bus::Bus,
  clock::Clock,
};
use palette::{Palette, NesColor};

use std::error::Error;

const FRAME_HEIGHT_NTSC: usize = 224;
const FRAME_HEIGHT_PAL: usize = 240;
const FRAME_WIDTH: usize = 256;

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
  scanline_index: u32,
  cycle_n: u32,
}

impl PPU {
  pub fn new() -> Self {
    Self {
      palette: Palette::new(),
      frame: Frame::new(FRAME_WIDTH, FRAME_HEIGHT_NTSC),
      oam: Memory::ram(256),
      palette_mem: Memory::ram(32),
      scanline_index: 0,
      cycle_n: 0,
    }
  }

  pub fn reset(&mut self) {
  }

  pub fn load_palette(&mut self, filename: &str) -> Result<(), Box<dyn Error>> {
    self.palette.change_from_file(filename)
  }

  pub fn render_frame(&mut self, bus: &mut Bus) -> &Frame {
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

  pub fn get_frame(&self) -> &Frame {
    &self.frame
  }

  pub fn get_cycles_info(&self) -> (u32, u32) {
    (self.scanline_index, self.cycle_n)
  }

  pub fn render_info(&self) -> PPUInfo {
    PPUInfo{frame_w: self.frame.width
      , frame_h: self.frame.height}
  }
}

impl Clock for PPU {
  fn tick(&mut self, bus: &mut Bus) -> bool {
    match self.cycle_n {
      1..=256 => {self.frame.put_pixel((self.cycle_n - 1) as usize
        , self.scanline_index as usize, self.palette.color[21])},
      _ => {},
    }
    if self.cycle_n == 340 {
      self.cycle_n = 0;
      self.scanline_index += 1;
    }
    else {
      self.cycle_n += 1;
    }
    true
  }
}
