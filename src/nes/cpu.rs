mod opcode;

use crate::nes::{
  memory::{MemRead, MemWrite, Memory},
  bus::Bus,
};


struct StatusReg {
  //7654 3210
  //NVss DIZC

  pub value: u8,
}

use opcode::*;

const STACK_ADDR: u16 = 0x0100;
const INDIRECT_BUG_JMP: bool = true;

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
  cycles_branch : u32,
  operand: [u8; 2],
  addr_abs: u16,
  addr_rel: i8,
  instr: InstructionInfo,
  op_len: u16,
  as_jump: bool,
}

impl CPU {
  pub fn new() -> Self{
    Self {
      reg: Reg::new(),
      cycles_since_startup: 0,
      cycles_instr: 0,
      cycles_branch: 0,
      operand: [0, 0],
      addr_abs: 0,
      addr_rel: 0,
      instr: InstructionInfo::new(),
      op_len: 0,
      as_jump: false,
    }
  }

  pub fn reset(&mut self, bus: &mut Bus) {
    self.reg.reset();
    self.cycles_since_startup = 0;
    self.cycles_instr = 0;

    self.reg.PC = ((bus.read(0xFFFD) as u16) << 8) + bus.read(0xFFFC) as u16;
    println!("PC : {:#04x}", self.reg.PC);
  }

  pub fn debug_exec_instr(&mut self, bus: &mut Bus) {
    self.read_instr(bus);
    print!("{} ", self.instr.instr);
    if self.op_len >= 1 {
      print!(": {:#02x}", self.operand[0]);
      if self.op_len == 2 {
        print!(", {:#02x}", self.operand[1]);
      }
    }
    self.exec_instr(bus);
    if self.as_jump {
      print!(" AS BRANCH");
    }
    println!("");
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
  fn exec_instr(&mut self, bus: &mut Bus) {
    self.handle_adressing_mode(bus);
    self.cycles_branch = 0;
    self.as_jump = false;
    match self.instr.instr {
      //Logical and arithmetic commands:
      Instruction::ORA => {self.ORA(bus)},
      Instruction::AND => {self.AND(bus)},
      Instruction::BIT => {self.BIT(bus)},
      Instruction::EOR => {self.EOR(bus)},
      Instruction::ADC => {self.ADC(bus)},
      Instruction::CMP => {self.CMP(bus)},
      Instruction::CPX => {self.CPX(bus)},
      Instruction::CPY => {self.CPY(bus)},
      Instruction::DEC => {self.DEC(bus)},
      Instruction::DEX => {self.DEX(bus)},
      Instruction::DEY => {self.DEY(bus)},
      Instruction::INC => {self.INC(bus)},
      Instruction::INX => {self.INX(bus)},
      Instruction::INY => {self.INY(bus)},
      Instruction::ASL => {self.ASL(bus)},
      Instruction::ROL => {self.ROL(bus)},
      Instruction::LSR => {self.LSR(bus)},
      Instruction::ROR => {self.ROR(bus)},
      //Move commands:
      Instruction::LDA => {self.LDA(bus)},
      Instruction::LDX => {self.LDX(bus)},
      Instruction::LDY => {self.LDY(bus)},
      Instruction::STA => {self.STA(bus)},
      Instruction::STX => {self.STX(bus)},
      Instruction::STY => {self.STY(bus)},
      Instruction::TAX => {self.TAX(bus)},
      Instruction::TXA => {self.TXA(bus)},
      Instruction::TAY => {self.TAY(bus)},
      Instruction::TYA => {self.TYA(bus)},
      Instruction::TSX => {self.TSX(bus)},
      Instruction::TXS => {self.TXS(bus)},
      Instruction::PLA => {self.PLA(bus)},
      Instruction::PHA => {self.PHA(bus)},
      Instruction::PLP => {self.PLP(bus)},
      Instruction::PHP => {self.PHP(bus)},
      //Jump commands:
      Instruction::BPL => {self.BPL(bus)},
      Instruction::BMI => {self.BMI(bus)},
      Instruction::BVC => {self.BVC(bus)},
      Instruction::BVS => {self.BVS(bus)},
      Instruction::BCC => {self.BCC(bus)},
      Instruction::BCS => {self.BCS(bus)},
      Instruction::BNE => {self.BNE(bus)},
      Instruction::BEQ => {self.BEQ(bus)},
      Instruction::BCS => {self.BCS(bus)},
      Instruction::JMP => {self.JMP(bus)},
      Instruction::JSR => {self.JSR(bus)},
      Instruction::RTS => {self.RTS(bus)},
      //Flags commands:
      Instruction::CLC => {self.CLC(bus)},
      Instruction::SEC => {self.SEC(bus)},
      Instruction::CLD => {self.CLD(bus)},
      Instruction::SED => {self.SED(bus)},
      Instruction::CLI => {self.CLI(bus)},
      Instruction::SEI => {self.SEI(bus)},
      Instruction::CLV => {self.SEI(bus)},
      Instruction::NOP => (),
      _ => {println!("not implemented yet: {}", self.instr.instr)}
    }
  }

  fn handle_adressing_mode(&mut self, bus: &mut Bus) {
    match self.instr.mode {
      OpMode::ACC => {self.operand[0] = self.reg.A},
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
        let mut addr2 = bus.read(ind + 1);
        if INDIRECT_BUG_JMP && self.operand[0] == 0xFF {
          addr2 = bus.read((self.operand[1] as usize) << 8);
        }
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

  //Logical and arithmetic commands:
  fn ADC(&mut self, _bus: &mut Bus) {
    if self.reg.P.get_D() == false {
      let carry: u8 = if self.reg.P.get_C() {1} else {0};
      let n_a: bool = if self.reg.A & 0b1000_0000 != 0 {true} else {false};
      let n_o: bool = if self.operand[0] & 0b1000_0000 != 0 {true} else {false};
      let (r, c) = self.reg.A.overflowing_add(self.operand[0]);
      let (r2, c2) = r.overflowing_add(carry);
      self.reg.A = r2;

      let n = if self.reg.A & 0b1000_0000 != 0 {true} else {false};
      self.reg.P.set_C(c || c2);
      self.reg.P.set_Z(if self.reg.A == 0 {true} else {false});
      self.reg.P.set_V(if n && !n_a && !n_o {true} else {false});
      self.reg.P.set_N(n);
    }
  }

  fn SBC(&mut self, _bus: &mut Bus) {
  }

  fn BIT(&mut self, _bus: &mut Bus) {
    let r = self.reg.A & self.operand[0];
    self.reg.P.set_Z(if r == 0 {true} else {false});
    self.reg.P.set_V(if self.operand[0] & 0b0100_0000 != 0 {true} else {false});
    self.reg.P.set_N(if self.operand[0] & 0b1000_0000 != 0 {true} else {false});
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

  fn CPX(&mut self, _bus: &mut Bus) {
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

  fn CPY(&mut self, _bus: &mut Bus) {
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
    let mut r: u8 = self.operand[0];
    if r == 0 {
      r = 0xFF;
      self.reg.P.set_N(true);
      self.reg.P.set_Z(false);
    } else if r == 1 {
      r = 0;
      self.reg.P.set_N(false);
      self.reg.P.set_Z(true);
    } else {
      r -= 1;
      self.reg.P.set_N(false);
      self.reg.P.set_Z(false);
    }
    bus.write(self.addr_abs.into(), r)
  }

  fn DEX(&mut self, _bus: &mut Bus) {
    let mut r: u8 = self.reg.X;
    if r == 0 {
      r = 0xFF;
      self.reg.P.set_N(true);
      self.reg.P.set_Z(false);
    } else if r == 1 {
      r = 0;
      self.reg.P.set_N(false);
      self.reg.P.set_Z(true);
    } else {
      r -= 1;
      self.reg.P.set_N(false);
      self.reg.P.set_Z(false);
    }
    self.reg.X = r;
  }

  fn DEY(&mut self, _bus: &mut Bus) {
    let mut r: u8 = self.reg.Y;
    if r == 0 {
      r = 0xFF;
      self.reg.P.set_N(true);
      self.reg.P.set_Z(false);
    } else if r == 1 {
      r = 0;
      self.reg.P.set_N(false);
      self.reg.P.set_Z(true);
    } else {
      r -= 1;
      self.reg.P.set_N(false);
      self.reg.P.set_Z(false);
    }
    self.reg.Y = r;
  }

  fn INC(&mut self, bus: &mut Bus) {
    let mut r: u8 = self.operand[0];
    if r == 0xFE {
      r = 0;
      self.reg.P.set_Z(true);
    } else {
      r += 1;
      self.reg.P.set_Z(false);
    }
    bus.write(self.addr_abs.into(), r);
    self.reg.P.set_N(if r & 0b1000_0000 != 0 {true} else {false});
  }

  fn INX(&mut self, _bus: &mut Bus) {
    let mut r : u8 = self.reg.X;
    if r == 0xFE {
      r = 0;
      self.reg.P.set_Z(true);
    } else {
      r += 1;
      self.reg.P.set_Z(false);
    }
    self.reg.X = r;
    self.reg.P.set_N(if r & 0b1000_0000 != 0 {true} else {false});
  }

  fn INY(&mut self, _bus: &mut Bus) {
    let mut r: u8 = self.reg.Y;
    if r == 0xFE {
      r = 0;
      self.reg.P.set_Z(true);
    } else {
      r += 1;
      self.reg.P.set_Z(false);
    }
    self.reg.Y = r;
    self.reg.P.set_N(if r & 0b1000_0000 != 0 {true} else {false});
  }

  fn ASL(&mut self, bus: &mut Bus) {
    let mut r = self.operand[0];
    self.reg.P.set_C(if r & 0b1000_0000 != 0 {true} else {false});
    r = r.wrapping_shl(1);
    match self.instr.mode {
      OpMode::ACC => {self.reg.A = r},
      _ => {bus.write(self.addr_abs.into(), r)},
    }
    self.reg.P.set_Z(if r == 0 {true} else {false});
    self.reg.P.set_N(if r & 0b1000_0000 != 0 {true} else {false});
  }

  fn LSR(&mut self, bus: &mut Bus) {
    let mut r = self.operand[0];
    self.reg.P.set_C(if r & 0b0000_0001 != 0 {true} else {false});
    r = r.wrapping_shr(1);
    match self.instr.mode {
      OpMode::ACC => {self.reg.A = r},
      _ => {bus.write(self.addr_abs.into(), r)},
    }
    self.reg.P.set_Z(if r == 0 {true} else {false});
    self.reg.P.set_N(false);
  }

  fn ROL(&mut self, bus: &mut Bus) {
    let mut r = self.operand[0];
    let carry = if self.reg.P.get_C() {true} else {false};
    self.reg.P.set_C(if r & 0b1000_0000 != 0 {true} else {false});
    r = r.wrapping_shl(1);
    if carry {
      r &= 0b0000_0001;
    }
    match self.instr.mode {
      OpMode::ACC => {self.reg.A = r},
      _ => {bus.write(self.addr_abs.into(), r)},
    }
    self.reg.P.set_Z(if r == 0 {true} else {false});
    self.reg.P.set_N(if r & 0b1000_0000 != 0 {true} else {false});
  }

  fn ROR(&mut self, bus: &mut Bus) {
    let mut r = self.operand[0];
    let carry = if self.reg.P.get_C() {true} else {false};
    self.reg.P.set_C(if r & 0b0000_0001 != 0 {true} else {false});
    r = r.wrapping_shr(1);
    if carry {
      r &= 0b1000_0000;
    }
    match self.instr.mode {
      OpMode::ACC => {self.reg.A = r},
      _ => {bus.write(self.addr_abs.into(), r)},
    }
    self.reg.P.set_Z(if r == 0 {true} else {false});
    self.reg.P.set_N(if r & 0b1000_0000 != 0 {true} else {false});
  }

  //Move commands:
  fn LDA(&mut self, _bus: &mut Bus) {
    self.reg.A = self.operand[0];
    self.reg.P.set_Z(if self.reg.A == 0 {true} else {false});
    self.reg.P.set_N(if self.reg.A & 0b1000_0000 != 0 {true} else {false});
  }

  fn LDX(&mut self, _bus: &mut Bus) {
    self.reg.X = self.operand[0];
    self.reg.P.set_Z(if self.reg.X == 0 {true} else {false});
    self.reg.P.set_N(if self.reg.X & 0b1000_0000 != 0 {true} else {false});
  }

  fn LDY (&mut self, _bus: &mut Bus) {
    self.reg.Y = self.operand[0];
    self.reg.P.set_Z(if self.reg.Y == 0 {true} else {false});
    self.reg.P.set_N(if self.reg.Y & 0b1000_0000 != 0 {true} else {false});
  }

  fn STA(&mut self, bus: &mut Bus) {
    bus.write(self.addr_abs.into(), self.reg.A);
  }

  fn STX(&mut self, bus: &mut Bus) {
    bus.write(self.addr_abs.into(), self.reg.X);
  }

  fn STY(&mut self, bus: &mut Bus) {
    bus.write(self.addr_abs.into(), self.reg.Y);
  }

  fn TAX(&mut self, _bus: &mut Bus) {
    self.reg.X = self.reg.A;
    self.reg.P.set_Z(if self.reg.X == 0 {true} else {false});
    self.reg.P.set_N(if self.reg.X & 0b1000_0000 != 0 {true} else {false});
  }

  fn TXA(&mut self, _bus: &mut Bus) {
    self.reg.A = self.reg.X;
    self.reg.P.set_Z(if self.reg.A == 0 {true} else {false});
    self.reg.P.set_N(if self.reg.A & 0b1000_0000 != 0 {true} else {false});
  }

  fn TAY(&mut self, _bus: &mut Bus) {
    self.reg.Y = self.reg.A;
    self.reg.P.set_Z(if self.reg.Y == 0 {true} else {false});
    self.reg.P.set_N(if self.reg.Y & 0b1000_0000 != 0 {true} else {false});
  }

  fn TYA(&mut self, _bus: &mut Bus) {
    self.reg.A = self.reg.Y;
    self.reg.P.set_Z(if self.reg.A == 0 {true} else {false});
    self.reg.P.set_N(if self.reg.A & 0b1000_0000 != 0 {true} else {false});
  }

  fn TSX(&mut self, _bus: &mut Bus) {
    self.reg.X = self.reg.S;
    self.reg.P.set_Z(if self.reg.X == 0 {true} else {false});
    self.reg.P.set_N(if self.reg.X & 0b1000_0000 != 0 {true} else {false});
  }

  fn TXS(&mut self, _bus: &mut Bus) {
    self.reg.S = self.reg.X;
    self.reg.P.set_Z(if self.reg.S == 0 {true} else {false});
    self.reg.P.set_N(if self.reg.S & 0b1000_0000 != 0 {true} else {false});
  }

  fn PHA(&mut self, bus: &mut Bus) {
    let addr = STACK_ADDR + self.reg.S as u16;
    bus.write(addr.into(), self.reg.A);
    self.reg.S = self.reg.S.wrapping_add(1);
  }

  fn PHP(&mut self, bus: &mut Bus) {
    let addr = STACK_ADDR + self.reg.S as u16;
    bus.write(addr.into(), self.reg.P.value);
    self.reg.S = self.reg.S.wrapping_add(1);
  }

  fn PLA(&mut self, bus: &mut Bus) {
    let addr = STACK_ADDR + self.reg.S as u16;
    self.reg.A = bus.read(addr.into());
    self.reg.S = self.reg.S.wrapping_sub(1);
  }

  fn PLP(&mut self, bus: &mut Bus) {
    let addr = STACK_ADDR + self.reg.S as u16;
    self.reg.P.value = bus.read(addr.into());
    self.reg.S = self.reg.S.wrapping_sub(1);
  }

  //Jump / Branch commands:
  fn branch(&mut self, _bus: &mut Bus) {
    let pc = self.addr_abs;
    self.cycles_branch = 
      if (pc.wrapping_shr(8)) == (self.reg.PC.wrapping_shr(8)) {3}
      else {1};
    self.as_jump = true;
  }

  fn BCS(&mut self, bus: &mut Bus) {
    if self.reg.P.get_C() {self.branch(bus);}
  }

  fn BCC(&mut self, bus: &mut Bus) {
    if !self.reg.P.get_C() {self.branch(bus);}
  }

  fn BEQ(&mut self, bus: &mut Bus) {
    if self.reg.P.get_Z() {self.branch(bus);}
  }

  fn BNE(&mut self, bus: &mut Bus) {
    if !self.reg.P.get_Z() {self.branch(bus);}
  }

  fn BMI(&mut self, bus: &mut Bus) {
    if self.reg.P.get_N() {self.branch(bus);}
  }

  fn BPL(&mut self, bus: &mut Bus) {
    if !self.reg.P.get_N() {self.branch(bus);}
  }

  fn BVS(&mut self, bus: &mut Bus) {
    if self.reg.P.get_V() {self.branch(bus);}
  }

  fn BVC(&mut self, bus: &mut Bus) {
    if !self.reg.P.get_V() {self.branch(bus);}
  }

  fn JMP(&mut self, _bus: &mut Bus) {
    self.reg.PC = self.addr_abs;
    self.as_jump = true;
  }

  fn JSR(&mut self, bus: &mut Bus) {
    let stack_addr = STACK_ADDR + self.reg.S as u16;
    let lsb: u8 = (self.reg.PC.wrapping_shr(8)).try_into().unwrap();
    let msb: u8 = (self.reg.PC.wrapping_shl(8).wrapping_shr(8)).try_into().unwrap();
    bus.write(stack_addr.into(), lsb.wrapping_sub(1));
    self.reg.S = self.reg.S.wrapping_add(1);
    bus.write(stack_addr.into(), msb);
    self.reg.S = self.reg.S.wrapping_add(1);
    self.reg.PC = self.addr_abs;
    self.as_jump = true;
  }

  fn RTS(&mut self, bus: &mut Bus) {
    let stack_addr = STACK_ADDR + self.reg.S as u16;
    let mut lsb = bus.read(stack_addr.into());
    self.reg.S = self.reg.S.wrapping_sub(1);
    let msb = bus.read(stack_addr.into());
    self.reg.S = self.reg.S.wrapping_sub(1);
    lsb = lsb.wrapping_sub(1);
    self.reg.PC = ((msb as u16) << 8) + lsb as u16;
    self.as_jump = true;
  }

  //Flags commands:
  fn CLC(&mut self, _bus: &mut Bus) {
    self.reg.P.set_C(false);
  }

  fn SEC(&mut self, _bus: &mut Bus) {
    self.reg.P.set_C(true);
  }

  fn CLD(&mut self, _bus: &mut Bus) {
    self.reg.P.set_D(false);
  }

  fn SED(&mut self, _bus: &mut Bus) {
    self.reg.P.set_D(true);
  }

  fn CLI(&mut self, _bus: &mut Bus) {
    self.reg.P.set_I(false);
  }

  fn SEI(&mut self, _bus: &mut Bus) {
    self.reg.P.set_I(true);
  }

  fn CLV(&mut self, _bus: &mut Bus) {
    self.reg.P.set_V(false);
  }

  //Interrupt commands:
  fn BRK(&mut self, _bus: &mut Bus) { // TODO
  }

  fn RTI(&mut self, _bus: &mut Bus) { // TODO
  }
}
