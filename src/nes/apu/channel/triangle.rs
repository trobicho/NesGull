use std::fmt;

use crate::nes::{
  apu::channel::{Channel, ChannelType},
  bus::Bus,
};

const SEQUENCE_LOOKUP_TABLE: [u8; 32] = [
  15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0,
  0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15
];

#[derive(Debug, Clone)]
#[allow(non_snake_case)]
struct Register {
  control_flag: bool,
  linear_counter_load: u8,
  timer: u16,
  lenght_counter_load: u8,
}

impl fmt::Display for Register {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl Register {
  pub fn new() -> Self {
    Self {
      control_flag: false,
      linear_counter_load: 0,
      timer: 0,
      lenght_counter_load: 0,
    }
  }

  fn load_from_channel_reg(&mut self, channel_reg: &[u8]) {
    self.control_flag = channel_reg[0] & 0b1000_0000 != 0;
    self.linear_counter_load = channel_reg[0] & 0b0111_1111;

    self.lenght_counter_load = channel_reg[2] >> 3;
  }
}

#[derive(Clone)]
pub struct Triangle {
  addr_first_reg: usize,
  reg: Register,
}

impl Channel for Triangle {
  fn tick(&mut self, bus: &mut Bus) -> u8 {
    self.handle_channel(bus)
  }
}

impl Triangle {
  pub fn new(addr_first_reg: usize) -> ChannelType {
    let new = Self {
      addr_first_reg,
      reg: Register::new(),
    };
    new.into()
  }

  fn handle_channel(&mut self, bus: &mut Bus) -> u8 {
    let channel_reg = bus.apu_mem.get_channel_reg(self.addr_first_reg);
    self.reg.load_from_channel_reg(&channel_reg);

    if self.reg.timer == 0 {
      self.reg.timer = (((channel_reg[2] & 0b0000_0111) as u16) << 8) | (channel_reg[1] as u16);
    }
    else {
      self.reg.timer -= 1;
    }
    let seq = SEQUENCE_LOOKUP_TABLE[self.reg.timer as usize % 32];
    seq
  }
}
