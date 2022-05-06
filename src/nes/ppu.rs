mod palette;

use palette::Palette

use std::error::Error;

const FRAME_HEIGHT_NTSC = 224;
const FRAME_HEIGHT_PAL = 240;
const FRAME_WIDTH = 256;

pub struct PPU {
  palette: Palette,
  frame Vec<u8>,
}

impl PPU {
  pub fn new() -> Self {
    Self {
      palette: Palette.new(),
      frame: vec![0; FRAME_WIDTH * FRAME_HEIGHT_NTSC],
    }
  }

  pub fn reset(&mut self) {
  }

  pub fn load_palette(&mut self, filename: &str) -> Result<(), Box<dyn Error>> {
    self.palette.change_from_file(filename);
  }

  pub fn render_frame(&mut self, bus: &mut Bus) -> &Vec<u8>{
    frame,
  }
}
