const NAMETABLE_ADDR: u16 = 0x2000;

pub struct Register {
  pub vram_cur: u16, //15 bits
  pub vram_temp: u16, //15 bits
  pub fine_x_scroll: u8, //3 bits
  pub w: bool,
  pub shift_back_16: [u16; 2],
  pub shift_back_8: [u8; 2],
}

impl Register {
  pub fn new() -> Self {
    Self {
      vram_cur: NAMETABLE_ADDR,
      vram_temp: NAMETABLE_ADDR,
      fine_x_scroll: 0x00,
      w: true,
      shift_back_16: [0; 2],
      shift_back_8: [0; 2],
    }
  }

  pub fn load_back_upper(&mut self, value: u8, pane: usize) {
    self.shift_back_16[pane] = (self.shift_back_16[pane] & 0b0000_0000_1111_1111)
      | (value as u16).wrapping_shl(8);
  }

  pub fn shift_background(&mut self) {
    self.shift_back_16[0] = self.shift_back_16[0].wrapping_shr(1);
    self.shift_back_16[1] = self.shift_back_16[1].wrapping_shr(1);
    self.shift_back_8[0] = self.shift_back_8[0].wrapping_shr(1);
    self.shift_back_8[1] = self.shift_back_8[1].wrapping_shr(1);
  }
}
