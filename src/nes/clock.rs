use crate::nes::{
  bus::Bus,
};

pub trait Clock {
  fn tick(&mut self, _bus: &mut Bus) -> bool {
    false
  }

  fn tick_n(&mut self, bus: &mut Bus, t: u32) {
    for _ in 0..t {
      self.tick(bus);
    }
  }
}

pub struct SlaveClock {
  pub div: u32,
  pub dec: u32,
}

impl SlaveClock {
  pub fn new(div: u32) -> Self{
    SlaveClock {
      div,
      dec: div - 1,
    }
  }

  pub fn reset(&mut self) {
    self.dec = self.div;
  }

  pub fn tick(&mut self) -> bool {
    if self.dec == 0 {
      self.dec = self.div - 1;
      true
    }
    else {
      self.dec -= 1;
      false
    }
  }
}
