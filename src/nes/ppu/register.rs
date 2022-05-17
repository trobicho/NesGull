use crate::nes::{
  memory::{Memory},
};

use super::memory::{Oam};

#[allow(non_snake_case)]
pub struct Register {
  pub shift_back_16: [u16; 2],
  pub shift_back_8: [u8; 2],

  pub NT_byte: u8,
  pub AT_byte: u8,
  pub latch_PT: [u8; 2],

  pub oam_secondary: Vec<Oam>,
  pub shift_sprite_high: Vec<u8>,
  pub shift_sprite_low: Vec<u8>,
  pub latch_sprite: Vec<u8>,
  pub counter_sprite: Vec<u8>,
  oam_cur: usize,
}

impl Register {
  pub fn new() -> Self {
    Self {
      shift_back_16: [0; 2],
      shift_back_8: [0; 2],
      NT_byte: 0,
      AT_byte: 0,
      latch_PT: [0; 2],

      oam_secondary: vec![Oam::new(); 8],
      shift_sprite_high: vec![0; 8],
      shift_sprite_low: vec![0; 8],
      latch_sprite: vec![0; 8],
      counter_sprite: vec![0; 8],
      oam_cur: 0,
    }
  }

  pub fn load_shift_reg(&mut self) {
    self.shift_back_16[0] = (self.shift_back_16[0] & 0b0000_0000_1111_1111)
      | (self.latch_PT[0] as u16).wrapping_shl(8);
    self.shift_back_16[1] = (self.shift_back_16[1] & 0b0000_0000_1111_1111)
      | (self.latch_PT[1] as u16).wrapping_shl(8);
    self.shift_back_8[0] = self.AT_byte;
    self.shift_back_8[1] = self.AT_byte;
  }

  pub fn shift_back_reg(&mut self) {
    self.shift_back_16[0] = self.shift_back_16[0].wrapping_shr(1);
    self.shift_back_16[1] = self.shift_back_16[1].wrapping_shr(1);
  }

  pub fn oam_clear_secondary(&mut self) {
    self.oam_secondary = vec![Oam::new(); self.oam_secondary.len()];
    self.oam_cur = 0;
  }

  pub fn counter_dec(&mut self) {
    for c in &mut self.counter_sprite {
      if (*c > 0) {
        *c -= 1;
      }
    }
  }

  pub fn oam_add(&mut self, oam: Oam) -> bool{
    if (self.oam_cur < 8) {
      self.oam_secondary[self.oam_cur] = oam;
      self.oam_cur += 1;
      true
    }
    else {
      false
    }
  }
}
