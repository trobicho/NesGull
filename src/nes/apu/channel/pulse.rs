use crate::nes::{
  memory::{MemRead, MemWrite, Memory},
  apu::channel::{Channel, ChannelType},
  bus::Bus,
  clock::Clock,
};

const sequence_lookup_table: [[u8; 8]; 4] = [
  [0, 0, 0, 0, 0, 0, 0, 1],
  [0, 0, 0, 0, 0, 0, 1, 1],
  [0, 0, 0, 0, 1, 1, 1, 1],
  [1, 1, 1, 1, 1, 1, 0, 0],
];

#[derive(Clone)]
pub struct Pulse {
  addr_first_reg: usize,
  timer: u16,
  shift_reg: u8,
  duty_index: usize,
}

impl Channel for Pulse {
  fn tick(&mut self, bus: &mut Bus) -> (bool, f32) {
    self.handle_channel(bus)
  }
}

impl Pulse {
  pub fn new(addr_first_reg: usize) -> ChannelType {
    Self {
      addr_first_reg,
      timer: 0,
      shift_reg: 0,
      duty_index: 0,
    }.into()
  }

  fn handle_channel(&mut self, bus: &mut Bus) -> (bool, f32) {
    let channel_reg = bus.apu_mem.get_channel_reg(self.addr_first_reg);
    let duty_cycle: usize = (channel_reg[0] >> 6).into();

    if self.timer == 0 {
      self.timer = (((channel_reg[3] & 0b0000_0111) as u16) << 8) | (channel_reg[2] as u16);
      if self.timer != 0 {
        self.duty_index = (self.duty_index + 1) % 8;
      }
    }
    else {
      self.timer -= 1;
    }
    if (((channel_reg[3] & 0b0000_0111) as u16) << 8) | (channel_reg[2] as u16) >= 8 {
      if sequence_lookup_table[self.duty_index % 4][duty_cycle] != 0 {
        (true, sequence_lookup_table[self.duty_index % 4][duty_cycle] as f32)
      }
      else {
        (false, 0f32)
      }
    }
    else {
      (false, 0f32)
    }
  }
}
