pub mod pulse;

use std::fmt;
use enum_dispatch::enum_dispatch;

use crate::nes::{
  memory::{Memory},
  bus::Bus,
  clock::Clock,
};

use pulse::Pulse;

#[derive(Debug, Clone)]
pub struct NullChannel {}

#[allow(clippy::large_enum_variant)]
#[enum_dispatch]
#[derive(Clone)]
pub enum ChannelType{
  NullChannel,
  Pulse,
  //Triangle,
  //Noise,
  //DMC,
}

#[enum_dispatch(ChannelType)]
pub trait Channel: {
  fn tick(&mut self, _bus: &mut Bus) -> (bool, f32) {
    (false, 0f32)
  }
}

impl Channel for NullChannel {}

pub fn null() -> ChannelType {
  let null = NullChannel {};
  null.into()
}
