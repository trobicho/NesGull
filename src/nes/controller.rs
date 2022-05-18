pub mod basic;

pub trait Controller {
  fn update(&mut self) {
  }
  fn debug_print(&self) {
  }
  fn read(&mut self, _addr: usize) -> u8 {
    0
  }
  fn write(&mut self, _addr: usize, _value: u8) {
  }
}
