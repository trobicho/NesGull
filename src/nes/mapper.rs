pub mod m000_nrom;
pub mod m001_mmc1;
pub mod m002_uxrom;

use std::fmt;
use enum_dispatch::enum_dispatch;
use std::error::Error;

use m000_nrom::Nrom;
use m001_mmc1::MMC1;
use m002_uxrom::Uxrom;

use crate::nes::{
  memory::{MemRead, MemWrite},
};

use crate::Cartridge;

#[derive(Debug)]
struct ErrorMissingMapper {
  mapper_num: u16,
}

impl ErrorMissingMapper {
  pub fn new(mapper_num: u16) -> Self {
    Self {
      mapper_num,
    }
  }
}

impl fmt::Display for ErrorMissingMapper {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "unsupported mapper number: {}", self.mapper_num)
  }
}

impl Error for ErrorMissingMapper {}


#[derive(Debug, Copy, Clone)]
pub enum MirroringType {
  Horizontal,
  Vertical,
  FourScreen,
  SingleScreenA,
  SingleScreenB,
}

impl fmt::Display for MirroringType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl Default for MirroringType {
  fn default() -> Self {
    MirroringType::Horizontal
  }
}

#[derive(Debug, Clone)]
pub struct NullMapper {}

#[allow(clippy::large_enum_variant)]
#[enum_dispatch]
#[derive(Debug, Clone)]
pub enum MapperType {
  NullMapper,
  Nrom,
  MMC1,
  Uxrom,
}

#[enum_dispatch(MapperType)]
pub trait Mapper: MemRead + MemWrite {
  fn irq_pending(&mut self) -> bool {
    false
  }
  fn mirroring(&self) -> MirroringType {
    MirroringType::Horizontal
  }
  fn battery_backed(&self) -> bool {
    false
  }
  fn use_ciram(&self, _addr: usize) -> bool {
    true
  }
  fn nametable_page(&self, _addr: usize) -> usize {
    0
  }
  fn ppu_write(&mut self, _addr: usize, _val: u8) {}
  fn open_bus(&mut self, _addr: usize, _val: u8) {}
  fn debug_print_vec(&mut self) {}
}

/// Attempts to return a valid Mapper for the given rom.
pub fn load_rom(cart: &Cartridge) -> Result<MapperType, Box<dyn Error>> {
  match cart.header.mapper_num {
    0 => Ok(Nrom::load(cart)),
    1 => Ok(MMC1::load(cart)),
    2 => Ok(Uxrom::load(cart)),
    71 => Ok(Uxrom::load(cart)), // TODO: Mapper 71 has slight differences from Uxrom
    _ => Err(Box::new(ErrorMissingMapper::new(cart.header.mapper_num))),
  }
}

impl Mapper for NullMapper {}
impl MemRead for NullMapper {}
impl MemWrite for NullMapper {}

pub fn null() -> MapperType {
  let null = NullMapper {};
  null.into()
}
