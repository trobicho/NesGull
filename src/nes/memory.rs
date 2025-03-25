use std::fmt;
use enum_dispatch::enum_dispatch;

#[enum_dispatch(MapperType)]
pub trait MemRead {
  fn read(&mut self, _addr: usize) -> u8 {
    0
  }
}

#[enum_dispatch(MapperType)]
pub trait MemWrite {
  fn write(&mut self, _addr: usize, _value: u8) {
    ()
  }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Bank {
  addr: usize,
  bank_n: usize,
}

impl fmt::Display for Bank {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

#[derive(Debug, Clone)]
pub struct BankableMemory {
  data: Vec<u8>,
  writable: bool,
  window: usize,
  banks: Vec<Bank>,
  bank_count: usize,
}

impl fmt::Display for BankableMemory {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl BankableMemory {
  pub fn with_capacity(capacity: usize, window: usize) -> Self {
    let capacity = capacity + (capacity % window);
    let data = vec![0; capacity];
    Self {
      data,
      writable: true,
      window,
      banks: Vec::new(),
      bank_count: capacity / window,
    }
  }

  pub fn rom(capacity: usize, window: usize) -> Self {
    let mut rom = Self::with_capacity(capacity, window);
    rom.writable = false;
    rom
  }

  pub fn rom_from_bytes(bytes: &[u8], window: usize) -> Self {
    let mut rom = Self::rom(bytes.len(), window);
    rom.data = bytes.to_vec();
    rom
  }

  pub fn ram(capacity: usize, window: usize) -> Self{
    Self::with_capacity(capacity, window)
  }

  pub fn ram_from_bytes(bytes: &[u8], window: usize) -> Self {
    let mut ram = Self::ram(bytes.len(), window);
    ram.data = bytes.to_vec();
    ram
  }

  pub fn add_bank_range(&mut self, addr_first: usize, addr_last: usize) {
    if self.bank_count == 0 {
      return;
    }
    let mut addr = addr_first;
    let mut i = 0;
    while addr < addr_last {
      self.banks.push(Bank{addr, bank_n: i % self.bank_count});
      addr += self.window;
      i += 1;
    }
  }

  pub fn set_bank(&mut self, addr: usize, bank_n: usize) {
    for bank in &mut self.banks {
      if bank.addr == addr {
        bank.bank_n = bank_n % self.bank_count;
        break;
      }
    }
  }

  pub fn last_bank(&self) -> usize {
    self.bank_count - 1
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

impl MemRead for BankableMemory {
  fn read(&mut self, addr: usize) -> u8 {
    let mut r = 0;
    for bank in &self.banks {
      if addr >= bank.addr && addr < bank.addr + self.window {
        r = self.data[bank.bank_n * self.window + (addr - bank.addr)];
        break;
      }
    }
    r
  }
}

impl MemWrite for BankableMemory {
  fn write(&mut self, addr: usize, value: u8) {
    for bank in &self.banks {
      if addr >= bank.addr && addr < bank.addr + self.window {
        self.data[bank.bank_n * self.window + (addr - bank.addr)] = value;
        //println!("{:#06x} = {:#04x}", addr, value);
        break;
      }
    }
  }
}
