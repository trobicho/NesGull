use std::fmt;

pub trait MemRead {
  fn read(&mut self, _addr: usize) -> u8 {
    0
  }
}

pub trait MemWrite {
  fn write(&mut self, _addr: usize, _value: u8) {
    ()
  }
}

#[derive(Debug)]
pub struct Memory {
  data: Vec<u8>,
  writable: bool,
}

impl fmt::Display for Memory {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl Memory {
  pub fn new() -> Self {
    Self::with_capacity(0)
  }

  pub fn with_capacity(capacity: usize) -> Self {
    let data = vec![0; capacity];
    Self {
      data,
      writable: true,
    }
  }

  pub fn rom(capacity: usize) -> Self {
    let mut rom = Self::with_capacity(capacity);
    rom.writable = false;
    rom
  }

  pub fn rom_from_bytes(bytes: &[u8]) -> Self {
    let mut rom = Self::rom(bytes.len());
    rom.data = bytes.to_vec();
    rom
  }

  pub fn ram(capacity: usize) -> Self{
    Self::with_capacity(capacity)
  }

  pub fn ram_from_bytes(bytes: &[u8]) -> Self {
    let mut ram = Self::ram(bytes.len());
    ram.data = bytes.to_vec();
    ram
  }

  pub fn len(&self) -> usize {
    self.data.len()
  }

  pub fn is_empty(&self) -> bool {
    self.data.is_empty()
  }

  pub fn writable(&self) -> bool {
    self.writable
  }

  pub fn write_protect(&mut self, protect: bool) {
    self.writable = !protect;
  }
}

impl MemRead for Memory {
  fn read(&mut self, addr: usize) -> u8 {
    if self.len() > 0 {
      let addr = addr % self.len();
      self.data[addr]
    }
    else {
      0
    }
  }
}

impl MemWrite for Memory {
  fn write(&mut self, addr: usize, value: u8) {
    if self.len() > 0 {
      let addr = addr % self.len();
      if self.writable {
        self.data[addr] = value;
      }
    }
  }
}
