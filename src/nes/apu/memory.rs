use std::fmt;

use crate::nes::{
  memory::{Memory, MemRead, MemWrite},
};

#[derive(Debug)]
pub struct APUMemory {
  pub pulse1_channel: [u8; 4],
  pub pulse2_channel: [u8; 4],
  pub triangle_channel: [u8; 3],
  pub noise_channel: [u8; 3],
  pub dmc_channel: [u8; 4],

  pub status: u8,
  pub frame_counter: u8,

  write_fc_counter: usize,
}

impl fmt::Display for APUMemory {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl APUMemory {
  pub fn new() -> Self {
    Self {
      pulse1_channel: [0; 4],
      pulse2_channel: [0; 4],
      triangle_channel: [0; 3],
      noise_channel: [0; 3],
      dmc_channel: [0; 4],
      status: 0,
      frame_counter: 0,
      write_fc_counter: 0,
    }
  }

  pub(super) fn tick(&mut self) -> bool {
    self.frame_counter.wrapping_add(1);
    if self.write_fc_counter > 0 {
      self.write_fc_counter -= 1;
      if self.write_fc_counter == 0 {
        self.frame_counter = 0;
      }
    }
    false
  }

  pub fn get_channel_reg(&self, addr: usize) -> &[u8]{
    match addr {
      0x4000 => &self.pulse1_channel,
      0x4004 => &self.pulse2_channel,
      0x4008 => &self.triangle_channel,
      0x400C => &self.noise_channel,
      0x4010 => &self.dmc_channel,
      _ => &[0 as u8]
    }
  }

  pub fn set_channel_reg(&mut self, addr: usize, reg_n: usize, value: u8) {
    match addr {
      0x4000 => self.pulse1_channel[reg_n] = value,
      0x4004 => self.pulse2_channel[reg_n] = value,
      0x4008 => self.triangle_channel[reg_n] = value,
      0x400C => self.noise_channel[reg_n] = value,
      0x4010 => self.dmc_channel[reg_n] = value,
      _ => (),
    }
  }
}

impl MemRead for APUMemory {
  fn read(&mut self, addr: usize) -> u8 {
    if addr == 0x4015 {
      self.status
    }
    else {
      0
    }
  }
}

impl MemWrite for APUMemory {
  fn write(&mut self, addr: usize, value: u8) {
    let addr = addr as u16;
    match addr {
      0x4000 => self.pulse1_channel[0] = value,
      0x4001 => self.pulse1_channel[1] = value,
      0x4002 => self.pulse1_channel[2] = value,
      0x4003 => self.pulse1_channel[3] = value,

      0x4004 => self.pulse2_channel[0] = value,
      0x4005 => self.pulse2_channel[1] = value,
      0x4006 => self.pulse2_channel[2] = value,
      0x4007 => self.pulse2_channel[3] = value,

      0x4008 => self.triangle_channel[0] = value,
      0x400A => self.triangle_channel[1] = value,
      0x400B => self.triangle_channel[2] = value,

      0x400C => self.noise_channel[0] = value & 0b0011_1111,
      0x400E => self.noise_channel[1] = value & 0b1000_1111,
      0x400F => self.noise_channel[2] = value & 0b1111_1000,

      0x4010 => self.dmc_channel[0] = value & 0b1100_1111,
      0x4011 => self.dmc_channel[1] = value & 0b0111_1111,
      0x4012 => self.dmc_channel[2] = value,
      0x4013 => self.dmc_channel[3] = value,

      0x4015 => self.status = value & 0b0001_1111,
      0x4017 => {
        self.frame_counter = value & 0b1100_0000;
        self.write_fc_counter = 2;
      },
      _ => (),
    }
  }
}
