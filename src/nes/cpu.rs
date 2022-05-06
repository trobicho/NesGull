mod opcode;

use crate::nes::{
  memory::{MemRead, MemWrite, Memory},
  bus::Bus,
};


struct StatusReg {
  //7654 3210
  //NVss DIZC

  value: u8,
}

use opcode::*;

#[allow(non_snake_case)]
impl StatusReg {
  fn new() -> Self {
    Self {
      value: 0x34,
    }
  }
  fn reset(&mut self) {
    self.value = 0x34;
  }
  fn set_N(&mut self, v : bool) {
    match v {
      true => {self.value |= 0b1000_0000},
      false => {self.value &= 0b0111_1111},
    }
  }
  fn set_V(&mut self, v : bool) {
    match v {
      true => {self.value |= 0b0100_0000},
      false => {self.value &= 0b1011_1111},
    }
  }
  fn set_D(&mut self, v : bool) {
    match v {
      true => {self.value |= 0b0000_1000},
      false => {self.value &= 0b1111_0111},
    }
  }
  fn set_I(&mut self, v : bool) {
    match v {
      true => {self.value |= 0b0000_0100},
      false => {self.value &= 0b1111_1011},
    }
  }
  fn set_Z(&mut self, v : bool) {
    match v {
      true => {self.value |= 0b0000_0010},
      false => {self.value &= 0b1111_1101},
    }
  }
  fn set_C(&mut self, v : bool) {
    match v {
      true => {self.value |= 0b0000_0001},
      false => {self.value &= 0b1111_1110},
    }
  }
  fn get_N(&self) -> bool {
    (self.value & 0b1000_0000) != 0x0
  }
  fn get_V(&self) -> bool {
    (self.value & 0b0100_0000) != 0x0
  }
  fn get_D(&self) -> bool {
    (self.value & 0b0000_1000) != 0x0
  }
  fn get_I(&self) -> bool {
    (self.value & 0b0000_0100) != 0x0
  }
  fn get_Z(&self) -> bool {
    (self.value & 0b0000_0010) != 0x0
  }
  fn get_C(&self) -> bool {
    (self.value & 0b0000_0001) != 0x0
  }
}

#[allow(non_snake_case)]
struct Reg {
  A: u8,
  X: u8,
  Y: u8,
  PC: u16,
  S: u8,
  P: StatusReg,
}

impl Reg {
  fn new() -> Self {
    Self {
      A: 0x0,
      X: 0x0,
      Y: 0x0,
      PC: 0x0,
      S: 0x0,
      P: StatusReg::new(),
    }
  }
  fn reset(&mut self) {
    self.A = 0x0;
    self.X = 0x0;
    self.Y = 0x0;
    self.PC = 0x0;
    self.S = 0xFD;
    self.P.reset();
  }
}

fn operand_lenght(instr: &InstructionInfo) -> u16 {
  match instr.mode {
    OpMode::IMP => 0,
    OpMode::ACC => 0,
    OpMode::IMM => 1,
    OpMode::ZP0 => 1,
    OpMode::ZPX => 1,
    OpMode::ZPY => 1,
    OpMode::IZX => 1,
    OpMode::IZY => 1,
    OpMode::ABS => 2,
    OpMode::ABX => 2,
    OpMode::ABY => 2,
    OpMode::IND => 2,
    OpMode::REL => 2,
  }
}

pub struct CPU {
  reg: Reg,
  cycles_since_startup: u32,
  cycles_instr: u32,
  operand: [u8; 2],
  addr_abs: u16,
  addr_rel: i8,
  instr: InstructionInfo,
  op_len: u16,
}

impl CPU {
  pub fn new() -> Self{
    Self {
      reg: Reg::new(),
      cycles_since_startup: 0,
      cycles_instr: 0,
      operand: [0, 0],
      addr_abs: 0,
      addr_rel: 0,
      instr: InstructionInfo::new(),
      op_len: 0,
    }
  }

  pub fn reset(&mut self, bus: &mut Bus) {
    self.reg.reset();
    self.cycles_since_startup = 0;
    self.cycles_instr = 0;

    self.reg.PC = ((bus.read(0xFFFD) as u16) << 8) + bus.read(0xFFFC) as u16;
    println!("PC : {:#04x}", self.reg.PC);
  }

  pub fn debug_read_instr(&mut self, bus: &mut Bus) {
    self.read_instr(bus);
    print!("{}", self.instr.instr);
    if self.op_len >= 1 {
      print!(": {:#02x}", self.operand[0]);
      if self.op_len == 2 {
        print!(", {:#02x}", self.operand[1]);
      }
    }
    println!("");
  }

  fn read_instr(&mut self, bus: &mut Bus) {
    let opcode = bus.read(self.reg.PC as usize);
    self.instr = opcode::opcode_to_enum(opcode);
    self.op_len = operand_lenght(&self.instr);
    if self.op_len >= 1 {
      self.operand[0] = bus.read((self.reg.PC + 1).into());
    }
    if self.op_len == 2 {
      self.operand[1] = bus.read((self.reg.PC + 2).into());
    }
    self.reg.PC += self.op_len + 1;
  }
}

#[allow(non_snake_case)]
impl CPU {
  fn handle_adressing_mode(&mut self, bus: &mut Bus) {
    match self.instr.mode {
      OpMode::ACC => {},
      OpMode::IMP => {},
      OpMode::IMM => {},
      OpMode::ZP0 => {
        self.addr_abs = self.operand[0].into();
        self.operand[0] = bus.read(self.addr_abs.into());
      },
      OpMode::ZPX => {
        self.addr_abs = self.operand[0].wrapping_add(self.reg.X).into();
        self.operand[0] = bus.read(self.addr_abs.into());
      },
      OpMode::ZPY => {
        self.addr_abs = self.operand[0].wrapping_add(self.reg.Y).into();
        self.operand[0] = bus.read(self.addr_abs.into());
      },
      OpMode::IZX => {
        let ind = self.operand[0].wrapping_add(self.reg.X).into();
        let addr1 = bus.read(ind);
        let addr2 = bus.read(ind + 1);
        self.addr_abs = ((addr1 as u16) << 8) + addr2 as u16;
        self.operand[0] = bus.read(self.addr_abs.into());
      },
      OpMode::IZY => {
        let ind: usize = self.operand[0].into();
        let addr1 = bus.read(ind);
        let addr2 = bus.read(ind + 1);
        self.addr_abs = (((addr1 as u16) << 8) + addr2 as u16) + self.reg.Y as u16;
        self.operand[0] = bus.read(self.addr_abs.into());
      },
      OpMode::ABS => {
        self.addr_abs = ((self.operand[1] as u16) << 8)  + self.operand[0] as u16;
        self.operand[0] = bus.read(self.addr_abs.into());
      },
      OpMode::ABX => {
        self.addr_abs = ((self.operand[1] as u16) << 8)  + self.operand[0] as u16 + self.reg.X as u16;
        self.operand[0] = bus.read(self.addr_abs.into());
      },
      OpMode::ABY => {
        self.addr_abs = ((self.operand[1] as u16) << 8)  + self.operand[0] as u16 + self.reg.Y as u16;
        self.operand[0] = bus.read(self.addr_abs.into());
      },
      OpMode::IND => {
        let ind: usize = ((self.operand[1] as usize) << 8) + self.operand[0] as usize;
        let addr1 = bus.read(ind);
        let addr2 = bus.read(ind + 1);
        self.addr_abs = ((addr1 as u16) << 8) + addr2 as u16;
      },
      OpMode::REL => {
        let offset: i8 = match self.operand[0] {
          0..=127 => {self.operand[0].try_into().unwrap()}
          128..=255 => {-1 as i8 - ((255 - self.operand[0]) as i8) }
        };
        self.addr_abs = self.reg.PC.wrapping_add(offset as u16);
        self.addr_rel = offset;
      },
    }
  }

  fn ADC(&mut self, bus: &mut Bus) {
    if self.reg.P.get_D() == false {
      let carry: u8 = if self.reg.P.get_C() {1} else {0};
      let n_a: bool = if self.reg.A & 0b1000_0000 != 0 {true} else {false};
      let n_o: bool = if self.operand[0] & 0b1000_0000 != 0 {true} else {false};
      let (r, c) = self.reg.A.overflowing_add(self.operand[0]);
      let (r2, c2) = r.overflowing_add(carry);
      self.reg.A = r;

      let n = if self.reg.A & 0b1000_0000 != 0 {true} else {false};
      self.reg.P.set_C(c || c2);
      self.reg.P.set_Z(if self.reg.A == 0 {true} else {false});
      self.reg.P.set_V(if n && !n_a && !n_o {true} else {false});
      self.reg.P.set_N(n);
    }
  }

  fn SBC(&mut self, _bus: &mut Bus) {
  }

  fn AND(&mut self, _bus: &mut Bus) {
    self.reg.A &= self.operand[0];
    self.reg.P.set_Z(if self.reg.A == 0 {true} else {false});
    self.reg.P.set_N(if self.reg.A & 0b1000_0000 != 0 {true} else {false});
  }

  fn ORA(&mut self, _bus: &mut Bus) {
    self.reg.A |= self.operand[0];
    self.reg.P.set_Z(if self.reg.A == 0 {true} else {false});
    self.reg.P.set_N(if self.reg.A & 0b1000_0000 != 0 {true} else {false});
  }

  fn EOR(&mut self, _bus: &mut Bus) {
    self.reg.A ^= self.operand[0];
    self.reg.P.set_Z(if self.reg.A == 0 {true} else {false});
    self.reg.P.set_N(if self.reg.A & 0b1000_0000 != 0 {true} else {false});
  }

  fn CMP(&mut self, _bus: &mut Bus) {
    if self.reg.A < self.operand[0] {
      self.reg.P.set_C(false);
      self.reg.P.set_Z(false);
      self.reg.P.set_N(true);
    } else if self.reg.A == self.operand[0] {
      self.reg.P.set_C(true);
      self.reg.P.set_Z(true);
      self.reg.P.set_N(false);
    } else {
      self.reg.P.set_C(true);
      self.reg.P.set_Z(false);
      self.reg.P.set_N(false);
    }
  }

  fn CMX(&mut self, _bus: &mut Bus) {
    if self.reg.X < self.operand[0] {
      self.reg.P.set_C(false);
      self.reg.P.set_Z(false);
      self.reg.P.set_N(true);
    } else if self.reg.X == self.operand[0] {
      self.reg.P.set_C(true);
      self.reg.P.set_Z(true);
      self.reg.P.set_N(false);
    } else {
      self.reg.P.set_C(true);
      self.reg.P.set_Z(false);
      self.reg.P.set_N(false);
    }
  }

  fn CMY(&mut self, _bus: &mut Bus) {
    if self.reg.Y < self.operand[0] {
      self.reg.P.set_C(false);
      self.reg.P.set_Z(false);
      self.reg.P.set_N(true);
    } else if self.reg.Y == self.operand[0] {
      self.reg.P.set_C(true);
      self.reg.P.set_Z(true);
      self.reg.P.set_N(false);
    } else {
      self.reg.P.set_C(true);
      self.reg.P.set_Z(false);
      self.reg.P.set_N(false);
    }
  }

  fn DEC(&mut self, bus: &mut Bus) {
    let mut result: u8 = self.operand[0];
    if result == 0 {
      result = 0xFF;
      self.reg.P.set_N(true);
      self.reg.P.set_Z(false);
    } else if result == 1 {
      result = 0;
      self.reg.P.set_N(false);
      self.reg.P.set_Z(true);
    } else {
      result -= 1;
      self.reg.P.set_N(false);
      self.reg.P.set_Z(false);
    }
    bus.write(self.addr_abs.into(), result)
  }

  fn DEX(&mut self, _bus: &mut Bus) {
    let mut result: u8 = self.reg.X;
    if result == 0 {
      result = 0xFF;
      self.reg.P.set_N(true);
      self.reg.P.set_Z(false);
    } else if result == 1 {
      result = 0;
      self.reg.P.set_N(false);
      self.reg.P.set_Z(true);
    } else {
      result -= 1;
      self.reg.P.set_N(false);
      self.reg.P.set_Z(false);
    }
    self.reg.X = result;
  }

  fn DEY(&mut self, _bus: &mut Bus) {
    let mut result: u8 = self.reg.Y;
    if result == 0 {
      result = 0xFF;
      self.reg.P.set_N(true);
      self.reg.P.set_Z(false);
    } else if result == 1 {
      result = 0;
      self.reg.P.set_N(false);
      self.reg.P.set_Z(true);
    } else {
      result -= 1;
      self.reg.P.set_N(false);
      self.reg.P.set_Z(false);
    }
    self.reg.Y = result;
  }
}
