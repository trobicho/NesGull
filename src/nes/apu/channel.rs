pub mod pulse;
pub mod triangle;

use enum_dispatch::enum_dispatch;

use crate::nes::bus::Bus;

use pulse::Pulse;
use triangle::Triangle;

#[derive(Debug, Clone)]
pub struct NullChannel {}

#[allow(clippy::large_enum_variant)]
#[enum_dispatch]
#[derive(Clone)]
pub enum ChannelType{
  NullChannel,
  Pulse,
  Triangle,
  //Noise,
  //DMC,
}

#[enum_dispatch(ChannelType)]
pub trait Channel: {
  fn tick(&mut self, _bus: &mut Bus) -> u8 {
    0
  }
}

impl Channel for NullChannel {}

pub fn null() -> ChannelType {
  let null = NullChannel {};
  null.into()
}
