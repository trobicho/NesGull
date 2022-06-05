pub mod memory;
pub mod channel;
pub mod mixer;

const SAMPLE_STEP: f32 = 40.58 / 2.0;

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
  next_sample_output: u32,
  step_fract: f32,
  output: f32,
}

impl APU {
  pub fn new() -> Self {
    Self {
      channels: vec![
        Pulse::new(0x4000),
        Pulse::new(0x4004),
      ],
      debug_count: 0,
      next_sample_output: SAMPLE_STEP.trunc() as u32,
      step_fract: SAMPLE_STEP.fract(),
      output: 0f32,
    }
  }

  pub fn mixer(&mut self, bus: &mut Bus) -> (bool, f32) {
    bus.apu_mem.tick();
    let mut r: f32 = 0f32;
    let mut has_sound: bool = false;

    for channel in &mut self.channels {
        let temp_r = channel.tick(bus);
        r += temp_r as f32;
    }
    self.output += r / 128.0;

    self.next_sample_output -= 1;
    if self.next_sample_output == 0 {
      self.step_fract += SAMPLE_STEP.fract();
      self.next_sample_output = (SAMPLE_STEP.trunc() + self.step_fract.trunc()) as u32;
      self.step_fract -= self.step_fract.trunc();
      let ret: f32 = self.output / SAMPLE_STEP;
      self.output = 0.0;
      (true, ret)
    }
    else {
      (false, 0f32)
    }
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
