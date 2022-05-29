pub mod memory;
pub mod channel;
pub mod mixer;

use crate::nes::{
  memory::{Memory},
  bus::Bus,
  clock::Clock,
};
use crate::nes::apu::channel::{Channel, ChannelType};
use crate::nes::apu::channel::{
  pulse::Pulse,
};

pub struct APU {
  channels: Vec<ChannelType>,
  debug_count: usize,
}

impl APU {
  pub fn new() -> Self {
    Self {
      channels: vec![
        Pulse::new(0x4000),
        Pulse::new(0x4004),
      ],
      debug_count: 0,
    }
  }

  pub fn mixer(&mut self, bus: &mut Bus) -> (bool, f32) {
    bus.apu_mem.tick();
    let mut r: f32 = 0f32;
    let mut has_sound: bool = false;

    for channel in &mut self.channels {
      let (temp_has_sound, temp_r) = channel.tick(bus);
      if (temp_has_sound) {
        has_sound = true;
        r = r.max(temp_r);
      }
    }
    (has_sound, r)
  }
}

impl Clock<()> for APU {
  fn tick(&mut self, bus: &mut Bus) -> () {
    let (mix, r) = self.mixer(bus);
    if mix {
      bus.mixer.add_to_stream(r);
    }
  }
}
