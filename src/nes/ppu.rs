mod palette;

use palette::Palette

use std::error::Error;

pub struct PPU {
  mapper: Mapper::Nrom,
  palette: Palette,
}

impl PPU {
  pub fn new() -> Self {
    Self {
      palette: Palette.new(),
    }
  }

  pub fn reset(&mut self) {
  }

  pub fn load_mapper(&mut self, mapper: Mapper::Nrom) {
    self.mapper = mapper;
  }

  pub fn load_palette(&mut self, filename: &str) -> Result<(), Box<dyn Error>> {
    self.palette.change_from_file(filename);
  }
}
