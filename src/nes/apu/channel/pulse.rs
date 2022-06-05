use std::fmt;

use crate::nes::{
  memory::{MemRead, MemWrite, Memory},
  apu::channel::{Channel, ChannelType},
  bus::Bus,
  clock::Clock,
};

/*
const sequence_lookup_table: [[u8; 8]; 4] = [
  [0, 0, 0, 0, 0, 0, 0, 1],
  [0, 0, 0, 0, 0, 0, 1, 1],
  [0, 0, 0, 0, 1, 1, 1, 1],
  [1, 1, 1, 1, 1, 1, 0, 0],
];
*/

const sequence_lookup_table: [[u8; 8]; 4] = [
  [0, 1, 0, 0, 0, 0, 0, 0],
  [0, 1, 1, 0, 0, 0, 0, 0],
  [0, 1, 1, 1, 1, 0, 0, 0],
  [1, 0, 0, 1, 1, 1, 1, 1],
];

#[derive(Debug, Clone)]
struct SweepReg {
  pub enable: bool,
  pub div: u8,
  pub neg: bool,
  pub shift_count: u8,
}

impl fmt::Display for SweepReg {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl SweepReg {
  pub fn new(v: u8) -> Self {
    Self {
      enable: v & 0b1000_0000 != 0,
      div: (v >> 4) & 0b0111,
      neg: v & 0b0000_1000 != 0,
      shift_count: v & 0b0111,
    }
  }
}

#[derive(Debug, Clone)]
#[allow(non_snake_case)]
struct Register {
  pub duty_cycle: u8,
  pub LC_halt_flag: bool,
  pub envelope_flag: bool,
  pub envelope: u8,

  pub sweep: SweepReg,
  pub timer: u16,
  pub LC_load: u8,
}

impl fmt::Display for Register {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl Register {
  pub fn new() -> Self {
    Self {
      duty_cycle: 0,
      LC_halt_flag: false,
      envelope_flag: false,
      envelope: 0,
      sweep: SweepReg::new(0),
      timer: 0,
      LC_load: 0,
    }
  }

  pub fn load_from_channel_reg(&mut self, channel_reg: &[u8]) {
    self.duty_cycle = channel_reg[0] >> 6;
    self.LC_halt_flag = channel_reg[0] & 0b0010_0000 != 0;
    self.envelope_flag = channel_reg[0] & 0b0001_0000 != 0;
    self.envelope = channel_reg[0] & 0b0000_1111;

    self.sweep = SweepReg::new(channel_reg[1]);

    self.LC_load = channel_reg[3] >> 3;
  }
}

#[derive(Clone)]
pub struct Pulse {
  addr_first_reg: usize,
  reg: Register,
  duty_index: usize,
}

impl Channel for Pulse {
  fn tick(&mut self, bus: &mut Bus) -> u8 {
    self.handle_channel(bus)
  }
}

impl Pulse {
  pub fn new(addr_first_reg: usize) -> ChannelType {
    Self {
      addr_first_reg,
      reg: Register::new(),
      duty_index: 0,
    }.into()
  }

  fn handle_channel(&mut self, bus: &mut Bus) -> u8 {
    let channel_reg = bus.apu_mem.get_channel_reg(self.addr_first_reg);
    self.reg.load_from_channel_reg(&channel_reg);

    if self.reg.timer == 0 {
      self.reg.timer = (((channel_reg[3] & 0b0000_0111) as u16) << 8) | (channel_reg[2] as u16);
      if self.reg.timer != 0 {
        self.duty_index = (self.duty_index + 1) % 8;
      }
    }
    else {
      self.reg.timer -= 1;
    }
    let mut seq = sequence_lookup_table[self.duty_index % 4][self.reg.duty_cycle as usize];
    if self.reg.LC_load == 0 || self.reg.timer < 8 {
      seq = 0;
    }
    //if (((channel_reg[3] & 0b0000_0111) as u16) << 8) | (channel_reg[2] as u16) >= 8 {
    //}
    if seq == 1 {
      self.reg.envelope
    }
    else {
      0
    }
  }
}