mod opcode;
use std::fmt;

use crate::nes::{
  memory::{MemRead, MemWrite},
  bus::Bus,
  clock::Clock,
};


#[derive(Debug)]
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
      value: 0x24,
    }
  }
  fn reset(&mut self) {
    self.value = 0x24;
  }

  fn set_value(&mut self, value: u8) {
    self.value = (value & 0b1100_1111)
                | (self.value & 0b0011_0000);
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
  fn set_B(&mut self, v : u8) -> u8 {
    let mut value = self.value;
    match v {
      1 => {value &= 0b0010_0000; value |= 0b0001_0000},
      2 => {value |= 0b0010_0000; value &= 0b0001_0000},
      3 => {value |= 0b0010_0000; value |= 0b0001_0000},
      0 | _ => {value &= 0b0010_0000; value &= 0b0001_0000},
    }
    value
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
#[derive(Debug)]
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

impl fmt::Display for StatusReg{
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:#8b}", self.value)
  }
}

impl fmt::Display for Reg{
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
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
    OpMode::REL => 1,
  }
}

pub struct CPU {
  reg: Reg,
  cycles_frame: u32,
  cycles_instr: u32,
  cycles_since_last_exec: u32,
  operand: [u8; 2],
  addr_abs: u16,
  addr_rel: i8,
  instr: InstructionInfo,
  op_len: u16,
  as_jump: bool,
  have_bcd: bool,
  instr_op_load: bool,
  pub debug: bool,
}

impl Clock for CPU {
  fn tick(&mut self, bus: &mut Bus) -> bool{
    self.cycles_since_last_exec += 1;
    self.cycles_frame += 1;
    //if bus.get_oam_dma_state() {
      //print!("-");
    //}
    if self.cycles_since_last_exec >= self.cycles_instr {
      if (bus.ppu_mem.get_nmi_output() && bus.ppu_mem.read_status() & 0b1000_0000 != 0) {
        bus.ppu_mem.nmi();
        self.cycles_instr += 2;
        self.NMI(bus);
      }
      else if bus.get_oam_dma_state() {
        if self.cycles_frame % 1 == 0 {
          bus.oam_dma_tick();
          self.cycles_since_last_exec = 0;
          self.cycles_instr = 2;
        }
      }
      else {
        if self.debug {
          self.debug_next_instr(bus);
        }
        else {
          self.next_instr(bus);
        }
        self.cycles_since_last_exec = 0;
      }
      true
    } else {
      false
    }
  }
}

impl CPU {
  pub fn new() -> Self{
    Self {
      reg: Reg::new(),
      cycles_frame: 0,
      cycles_instr: 0,
      cycles_since_last_exec: 0,
      operand: [0, 0],
      addr_abs: 0,
      addr_rel: 0,
      instr: InstructionInfo::new(),
      op_len: 0,
      as_jump: false,
      have_bcd: false,
      instr_op_load: false,
      debug: false,
    }
  }

  pub fn reset_cycles_frame(&mut self) {
    self.cycles_frame = 0;
  }

  pub fn get_cycles_frame(&self) -> u32 {
    self.cycles_frame
  }

  pub fn reset(&mut self, bus: &mut Bus) {
    self.reg.reset();
    self.cycles_frame = 0;
    self.cycles_instr = 2;
    self.cycles_since_last_exec = 0;

    self.reg.PC = ((bus.read(0xFFFD) as u16) << 8) + bus.read(0xFFFC) as u16;
    bus.write(0xFE, 0xFF);
    bus.write(0xFF, 0xFF);
    println!("PC : {:#04x}", self.reg.PC);
  }

  pub fn next_instr(&mut self, bus: &mut Bus) {
    self.cycles_instr = 0;
    self.read_instr(bus);
    self.exec_instr(bus);
  }

  pub fn set_debug(&mut self, debug: bool) {
    self.debug = debug;
  }

  fn read_instr(&mut self, bus: &mut Bus) {
    let opcode = bus.read(self.reg.PC as usize);
    self.instr = opcode::opcode_to_enum(opcode);
    self.op_len = operand_lenght(&self.instr);
    if self.op_len >= 1 {
      self.operand[0] = bus.read((self.reg.PC.wrapping_add(1)).into());
    }
    if self.op_len == 2 {
      self.operand[1] = bus.read((self.reg.PC.wrapping_add(2)).into());
    }
    self.reg.PC = self.reg.PC.wrapping_add(self.op_len + 1);
  }
}

impl CPU {
  pub fn debug_reset(&mut self, bus: &mut Bus) {
    self.reg.reset();
    self.reset(bus);
    self.cycles_instr = 7;

    self.reg.PC = 0xC000;
  }

  pub fn debug_next_instr(&mut self, bus: &mut Bus) {
    self.cycles_instr = 0;
    print!("{:#06x}", self.reg.PC);
    self.read_instr(bus);
    print!(" {}[{:#04x}]", self.instr.instr, self.instr.opcode);
    if self.op_len >= 1 {
      print!(" {:#04x}", self.operand[0]);
      if self.op_len == 2 {
        print!(" {:#04x}", self.operand[1]);
      } else {
        print!("\t");
      }
    } else {
      print!("\t");
    }
    print!("\t{:#04x} {:#04x} {:#04x} {:#04x} {:#04x}", self.reg.A, self.reg.X, self.reg.Y, self.reg.P.value, self.reg.S);
    self.exec_instr(bus);
    if self.as_jump {
      //print!(" AS BRANCH to {:#04x}", self.reg.PC);
    }
    //println!("");
  }

  pub fn debug_print_stack(&mut self, bus: &mut Bus) {
    for s in self.reg.S..=0xFF {
      let addr: usize = (STACK_ADDR + s as u16).into();
      println!("Stack{:04x}: {:#04x}", addr, bus.read(addr));
    }
  }
}

#[allow(non_snake_case)]
impl CPU {
  fn exec_instr(&mut self, bus: &mut Bus) {
    self.cycles_instr = self.instr.cycles;
    self.handle_adressing_mode(bus);
    self.as_jump = false;
    match self.instr.instr {
      //Logical and arithmetic commands:
      Instruction::ORA => {self.ORA(bus)},
      Instruction::AND => {self.AND(bus)},
      Instruction::BIT => {self.BIT(bus)},
      Instruction::EOR => {self.EOR(bus)},
      Instruction::ADC => {self.ADC(bus)},
      Instruction::SBC => {self.SBC(bus)},
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
      Instruction::JMP => {self.JMP(bus)},
      Instruction::JSR => {self.JSR(bus)},
      Instruction::RTS => {self.RTS(bus)},
      //Interrupt commands:
      Instruction::RTI => {self.RTI(bus)},
      //Instruction::BRK => {self.BRK(bus)},
      //Flags commands:
      Instruction::CLC => {self.CLC(bus)},
      Instruction::SEC => {self.SEC(bus)},
      Instruction::CLD => {self.CLD(bus)},
      Instruction::SED => {self.SED(bus)},
      Instruction::CLI => {self.CLI(bus)},
      Instruction::SEI => {self.SEI(bus)},
      Instruction::CLV => {self.CLV(bus)},
      Instruction::NOP => (),
      //Illegal commands:
      Instruction::SLO => (self.SLO(bus)),
      Instruction::RLA => (self.RLA(bus)),
      Instruction::SRE => (self.SRE(bus)),
      Instruction::RRA => (self.RRA(bus)),
      Instruction::SAX => (self.SAX(bus)),
      Instruction::LAX => (self.LAX(bus)),
      Instruction::DCP => (self.DCP(bus)),
      Instruction::ISC => (self.ISC(bus)),
      Instruction::ANC => (self.ANC(bus)),
      Instruction::ALR => (self.ALR(bus)),
      Instruction::ARR => (self.ARR(bus)),
      _ => {println!("not implemented yet: {}", self.instr.instr)}
    }
  }

  fn load_operand(&mut self, bus :&mut Bus) {
    if !self.instr_op_load {
      match self.instr.mode {
        OpMode::ACC => {self.operand[0] = self.reg.A},
        OpMode::ZP0 => {self.operand[0] = bus.read(self.addr_abs.into());},
        OpMode::ZPX => {self.operand[0] = bus.read(self.addr_abs.into());},
        OpMode::ZPY => {self.operand[0] = bus.read(self.addr_abs.into());},
        OpMode::IZX => {self.operand[0] = bus.read(self.addr_abs.into());},
        OpMode::IZY => {self.operand[0] = bus.read(self.addr_abs.into());},
        OpMode::ABS => {self.operand[0] = bus.read(self.addr_abs.into());},
        OpMode::ABX => {self.operand[0] = bus.read(self.addr_abs.into());},
        OpMode::ABY => {self.operand[0] = bus.read(self.addr_abs.into());},
        _ => (),
      }
    }
    self.instr_op_load = true;
  }

  fn handle_adressing_mode(&mut self, bus: &mut Bus) {
    self.instr_op_load = false;
    match self.instr.mode {
      OpMode::ACC => {self.operand[0] = self.reg.A},
      OpMode::IMP => {},
      OpMode::IMM => {},
      OpMode::ZP0 => {
        self.addr_abs = self.operand[0].into();
      },
      OpMode::ZPX => {
        self.addr_abs = self.operand[0].wrapping_add(self.reg.X).into();
      },
      OpMode::ZPY => {
        self.addr_abs = self.operand[0].wrapping_add(self.reg.Y).into();
      },
      OpMode::IZX => {
        let ind: u8 = self.operand[0].wrapping_add(self.reg.X);
        let addr1 = bus.read(ind.into());
        let addr2 = bus.read(ind.wrapping_add(1).into());
        self.addr_abs = ((addr2 as u16) << 8) + addr1 as u16;
        //print!("IZX:{:#04x}={:#04x},{:#04x}={:#04x},{:#04x}", ind, addr1
          //, ind.wrapping_add(1), addr2, self.operand[0]);
      },
      OpMode::IZY => {
        let ind: u8 = self.operand[0].into();
        let addr1 = bus.read(ind.into());
        let addr2 = bus.read(ind.wrapping_add(1).into());
        self.addr_abs = (((addr2 as u16) << 8) + addr1 as u16).wrapping_add(self.reg.Y as u16);
        if self.instr.cycle_inc_pbc && self.addr_abs.wrapping_shr(8) != addr2 as u16 {
          self.cycles_instr += 1;
        }
      },
      OpMode::ABS => {
        self.addr_abs = ((self.operand[1] as u16) << 8) + self.operand[0] as u16;
      },
      OpMode::ABX => {
        self.addr_abs = (((self.operand[1] as u16) << 8) + self.operand[0] as u16)
          .wrapping_add(self.reg.X as u16);
        if self.instr.cycle_inc_pbc
            && self.addr_abs.wrapping_shr(8) != self.operand[1] as u16 {
          self.cycles_instr += 1;
        }
      },
      OpMode::ABY => {
        self.addr_abs = (((self.operand[1] as u16) << 8) + self.operand[0] as u16)
          .wrapping_add(self.reg.Y as u16);
        if self.instr.cycle_inc_pbc
            && self.addr_abs.wrapping_shr(8) != self.operand[1] as u16 {
          self.cycles_instr += 1;
        }
      },
      OpMode::IND => {
        let ind: usize = ((self.operand[1] as usize) << 8) + self.operand[0] as usize;
        let addr1 = bus.read(ind);
        let mut addr2 = bus.read(ind + 1);
        if INDIRECT_BUG_JMP && self.operand[0] == 0xFF {
          addr2 = bus.read((self.operand[1] as usize) << 8);
        }
        self.addr_abs = ((addr2 as u16) << 8) + addr1 as u16;
      },
      OpMode::REL => {
        let offset: i8 = match self.operand[0] {
          0..=127 => {self.operand[0].try_into().unwrap()}
          128..=255 => {-1 as i8 - ((255 - self.operand[0]) as i8) }
        };
        self.addr_abs = self.reg.PC.wrapping_add(offset as u16);
        //print!(" {:#06x} {} {:#06x}", self.reg.PC, offset, self.addr_abs);
        self.addr_rel = offset;
      },
    }
  }

  //Logical and arithmetic commands:
  fn ADC(&mut self, bus: &mut Bus) {
    self.load_operand(bus);
    if !self.have_bcd || self.reg.P.get_D() == false {
      let carry: u8 = if self.reg.P.get_C() {1} else {0};
      let n_a: bool = if self.reg.A & 0b1000_0000 != 0 {true} else {false};
      let n_o: bool = if self.operand[0] & 0b1000_0000 != 0 {true} else {false};
      let (r, c) = self.reg.A.overflowing_add(self.operand[0]);
      let (r2, c2) = r.overflowing_add(carry);
      self.reg.A = r2;

      let n = if self.reg.A & 0b1000_0000 != 0 {true} else {false};
      self.reg.P.set_C(c || c2);
      self.reg.P.set_Z(if self.reg.A == 0 {true} else {false});
      self.reg.P.set_V(if (n && !n_a && !n_o)
        || (!n && n_a && n_o) {true} else {false});
      self.reg.P.set_N(n);
    }
  }

  fn SBC(&mut self, bus: &mut Bus) {
    self.load_operand(bus);
    self.operand[0] = !self.operand[0];
    self.ADC(bus);
  }

  fn BIT(&mut self, bus: &mut Bus) {
    self.load_operand(bus);
    let r = self.reg.A & self.operand[0];
    self.reg.P.set_Z(if r == 0 {true} else {false});
    self.reg.P.set_V(if self.operand[0] & 0b0100_0000 != 0 {true} else {false});
    self.reg.P.set_N(if self.operand[0] & 0b1000_0000 != 0 {true} else {false});
  }

  fn AND(&mut self, bus: &mut Bus) {
    self.load_operand(bus);
    self.reg.A &= self.operand[0];
    self.reg.P.set_Z(if self.reg.A == 0 {true} else {false});
    self.reg.P.set_N(if self.reg.A & 0b1000_0000 != 0 {true} else {false});
  }

  fn ORA(&mut self, bus: &mut Bus) {
    self.load_operand(bus);
    self.reg.A |= self.operand[0];
    self.reg.P.set_Z(if self.reg.A == 0 {true} else {false});
    self.reg.P.set_N(if self.reg.A & 0b1000_0000 != 0 {true} else {false});
  }

  fn EOR(&mut self, bus: &mut Bus) {
    self.load_operand(bus);
    self.reg.A ^= self.operand[0];
    self.reg.P.set_Z(if self.reg.A == 0 {true} else {false});
    self.reg.P.set_N(if self.reg.A & 0b1000_0000 != 0 {true} else {false});
  }

  fn CMP(&mut self, bus: &mut Bus) {
    self.load_operand(bus);
    self.reg.P.set_C(if self.reg.A >= self.operand[0] {true} else {false});
    self.reg.P.set_Z(if self.reg.A == self.operand[0] {true} else {false});
    self.reg.P.set_N(
      if self.reg.A.wrapping_sub(self.operand[0]) & 0b1000_0000 != 0 {
        true
      }
      else {
        false
      }
    );
  }

  fn CPX(&mut self, bus: &mut Bus) {
    self.load_operand(bus);
    self.reg.P.set_C(if self.reg.X >= self.operand[0] {true} else {false});
    self.reg.P.set_Z(if self.reg.X == self.operand[0] {true} else {false});
    self.reg.P.set_N(
      if self.reg.X.wrapping_sub(self.operand[0]) & 0b1000_0000 != 0 {
        true
      }
      else {
        false
      }
    );
  }

  fn CPY(&mut self, bus: &mut Bus) {
    self.load_operand(bus);
    self.reg.P.set_C(if self.reg.Y >= self.operand[0] {true} else {false});
    self.reg.P.set_Z(if self.reg.Y == self.operand[0] {true} else {false});
    self.reg.P.set_N(
      if self.reg.Y.wrapping_sub(self.operand[0]) & 0b1000_0000 != 0 {
        true
      }
        else {
        false
      }
    );
  }

  fn DEC(&mut self, bus: &mut Bus) {
    self.load_operand(bus);
    let mut r: u8 = self.operand[0];
    r = r.wrapping_sub(1);
    self.reg.P.set_Z(if r == 0 {true} else {false});
    self.reg.P.set_N(if r & 0b1000_0000 != 0 {true} else {false});
    bus.write(self.addr_abs.into(), r);
    self.operand[0] = r;
  }

  fn DEX(&mut self, _bus: &mut Bus) {
    let mut r: u8 = self.reg.X;
    r = r.wrapping_sub(1);
    self.reg.P.set_Z(if r == 0 {true} else {false});
    self.reg.P.set_N(if r & 0b1000_0000 != 0 {true} else {false});
    self.reg.X = r;
  }

  fn DEY(&mut self, _bus: &mut Bus) {
    let mut r: u8 = self.reg.Y;
    r = r.wrapping_sub(1);
    self.reg.P.set_Z(if r == 0 {true} else {false});
    self.reg.P.set_N(if r & 0b1000_0000 != 0 {true} else {false});
    self.reg.Y = r;
  }

  fn INC(&mut self, bus: &mut Bus) {
    self.load_operand(bus);
    let mut r: u8 = self.operand[0];
    r = r.wrapping_add(1);
    self.reg.P.set_Z(if r == 0 {true} else {false});
    self.reg.P.set_N(if r & 0b1000_0000 != 0 {true} else {false});
    bus.write(self.addr_abs.into(), r);
    self.operand[0] = r;
  }

  fn INX(&mut self, _bus: &mut Bus) {
    let mut r : u8 = self.reg.X;
    r = r.wrapping_add(1);
    self.reg.P.set_Z(if r == 0 {true} else {false});
    self.reg.P.set_N(if r & 0b1000_0000 != 0 {true} else {false});
    self.reg.X = r;
  }

  fn INY(&mut self, _bus: &mut Bus) {
    let mut r: u8 = self.reg.Y;
    r = r.wrapping_add(1);
    self.reg.P.set_Z(if r == 0 {true} else {false});
    self.reg.P.set_N(if r & 0b1000_0000 != 0 {true} else {false});
    self.reg.Y = r;
  }

  fn ASL(&mut self, bus: &mut Bus) {
    self.load_operand(bus);
    let mut r = self.operand[0];
    self.reg.P.set_C(if r & 0b1000_0000 != 0 {true} else {false});
    r = r.wrapping_shl(1);
    match self.instr.mode {
      OpMode::ACC => {self.reg.A = r},
      _ => {bus.write(self.addr_abs.into(), r)},
    }
    self.reg.P.set_Z(if r == 0 {true} else {false});
    self.reg.P.set_N(if r & 0b1000_0000 != 0 {true} else {false});
    self.operand[0] = r;
  }

  fn LSR(&mut self, bus: &mut Bus) {
    self.load_operand(bus);
    let mut r = self.operand[0];
    self.reg.P.set_C(if r & 0b0000_0001 != 0 {true} else {false});
    r = r.wrapping_shr(1);
    match self.instr.mode {
      OpMode::ACC => {self.reg.A = r},
      _ => {bus.write(self.addr_abs.into(), r)},
    }
    self.reg.P.set_Z(if r == 0 {true} else {false});
    self.reg.P.set_N(false);
    self.operand[0] = r;
  }

  fn ROL(&mut self, bus: &mut Bus) {
    self.load_operand(bus);
    let mut r = self.operand[0];
    let carry = if self.reg.P.get_C() {true} else {false};
    self.reg.P.set_C(if r & 0b1000_0000 != 0 {true} else {false});
    r = r.wrapping_shl(1);
    if carry {
      r |= 0b0000_0001;
    }
    match self.instr.mode {
      OpMode::ACC => {self.reg.A = r},
      _ => {bus.write(self.addr_abs.into(), r)},
    }
    self.reg.P.set_Z(if r == 0 {true} else {false});
    self.reg.P.set_N(if r & 0b1000_0000 != 0 {true} else {false});
    self.operand[0] = r;
  }

  fn ROR(&mut self, bus: &mut Bus) {
    self.load_operand(bus);
    let mut r = self.operand[0];
    let carry = if self.reg.P.get_C() {true} else {false};
    self.reg.P.set_C(if r & 0b0000_0001 != 0 {true} else {false});
    r = r.wrapping_shr(1);
    if carry {
      r |= 0b1000_0000;
    }
    match self.instr.mode {
      OpMode::ACC => {self.reg.A = r},
      _ => {bus.write(self.addr_abs.into(), r)},
    }
    self.reg.P.set_Z(if r == 0 {true} else {false});
    self.reg.P.set_N(if r & 0b1000_0000 != 0 {true} else {false});
    self.operand[0] = r;
  }

  //Move commands:
  fn LDA(&mut self, bus: &mut Bus) {
    self.load_operand(bus);
    self.reg.A = self.operand[0];
    self.reg.P.set_Z(if self.reg.A == 0 {true} else {false});
    self.reg.P.set_N(if self.reg.A & 0b1000_0000 != 0 {true} else {false});
  }

  fn LDX(&mut self, bus: &mut Bus) {
    self.load_operand(bus);
    self.reg.X = self.operand[0];
    self.reg.P.set_Z(if self.reg.X == 0 {true} else {false});
    self.reg.P.set_N(if self.reg.X & 0b1000_0000 != 0 {true} else {false});
  }

  fn LDY (&mut self, bus: &mut Bus) {
    self.load_operand(bus);
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
  }

  fn PHA(&mut self, bus: &mut Bus) {
    let addr = STACK_ADDR + self.reg.S as u16;
    bus.write(addr.into(), self.reg.A);
    self.reg.S = self.reg.S.wrapping_sub(1);
  }

  fn PHP(&mut self, bus: &mut Bus) {
    let addr = STACK_ADDR + self.reg.S as u16;
    bus.write(addr.into(), self.reg.P.set_B(3));
    self.reg.S = self.reg.S.wrapping_sub(1);
  }

  fn PLA(&mut self, bus: &mut Bus) {
    self.reg.S = self.reg.S.wrapping_add(1);
    let addr = STACK_ADDR + self.reg.S as u16;
    self.reg.A = bus.read(addr.into());
    self.reg.P.set_Z(if self.reg.A == 0 {true} else {false});
    self.reg.P.set_N(if self.reg.A & 0b1000_0000 != 0 {true} else {false});
  }

  fn PLP(&mut self, bus: &mut Bus) {
    self.reg.S = self.reg.S.wrapping_add(1);
    let addr = STACK_ADDR + self.reg.S as u16;
    self.reg.P.set_value(bus.read(addr.into()));
  }

  //Jump / Branch commands:
  fn branch(&mut self, _bus: &mut Bus) {
    self.cycles_instr += 1;
    if self.instr.cycle_inc_pbc && self.reg.PC.wrapping_shr(8) !=
        self.addr_abs.wrapping_shr(8) {
      self.cycles_instr += 1;
    }
    self.reg.PC = self.addr_abs;
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
    let msb: u8 = (self.reg.PC.wrapping_shr(8)).try_into().unwrap();
    let lsb: u8 = (self.reg.PC & 0xFF).try_into().unwrap();

    bus.write(stack_addr.into(), msb);
    self.reg.S = self.reg.S.wrapping_sub(1);
    let stack_addr = STACK_ADDR + self.reg.S as u16;
    bus.write(stack_addr.into(), lsb.wrapping_sub(1));
    self.reg.S = self.reg.S.wrapping_sub(1);
    self.reg.PC = self.addr_abs;
    self.as_jump = true;
  }

  fn RTS(&mut self, bus: &mut Bus) {
    self.reg.S = self.reg.S.wrapping_add(1);
    let stack_addr = STACK_ADDR + self.reg.S as u16;
    let mut lsb = bus.read(stack_addr.into());

    self.reg.S = self.reg.S.wrapping_add(1);
    let stack_addr = STACK_ADDR + self.reg.S as u16;
    let msb = bus.read(stack_addr.into());

    lsb = lsb.wrapping_add(1);
    self.reg.PC = ((msb as u16) << 8) + lsb as u16;
    self.as_jump = true;
  }

  //Interrupt commands:
  fn BRK(&mut self, bus: &mut Bus) { // TODO
    let msb: u8 = (self.reg.PC.wrapping_shr(8)) as u8;
    let lsb: u8 = (self.reg.PC & 0xFF) as u8;

    let stack_addr = STACK_ADDR + self.reg.S as u16;
    self.reg.P.set_B(1);
    bus.write(stack_addr.into(), msb);
    self.reg.S = self.reg.S.wrapping_sub(1);

    let stack_addr = STACK_ADDR + self.reg.S as u16;
    bus.write(stack_addr.into(), lsb);
    self.reg.S = self.reg.S.wrapping_sub(1);

    let stack_addr = STACK_ADDR + self.reg.S as u16;
    bus.write(stack_addr.into(), self.reg.P.value);
    self.reg.S = self.reg.S.wrapping_sub(1);
    self.reg.PC = (bus.read(0xFFFE) as u16) | (bus.read(0xFFFF) as u16).wrapping_shl(8);
  }

  fn IRQ(&mut self, bus: &mut Bus) { // TODO
    let addr = STACK_ADDR + self.reg.S as u16;
    bus.write(addr.into(), self.reg.P.set_B(2));
  }

  fn NMI(&mut self, bus: &mut Bus) { // TODO
    let msb: u8 = (self.reg.PC.wrapping_shr(8)) as u8;
    let lsb: u8 = (self.reg.PC & 0xFF) as u8;

    //println!(" NMI({:#06x}) {:#04x} ", self.reg.PC, self.reg.P.value);
    //println!("BEFORE:");
    //self.debug_print_stack(bus);
    let stack_addr = STACK_ADDR + self.reg.S as u16;
    self.reg.P.set_B(2);
    bus.write(stack_addr.into(), msb);
    self.reg.S = self.reg.S.wrapping_sub(1);

    let stack_addr = STACK_ADDR + self.reg.S as u16;
    bus.write(stack_addr.into(), lsb);
    self.reg.S = self.reg.S.wrapping_sub(1);

    let stack_addr = STACK_ADDR + self.reg.S as u16;
    bus.write(stack_addr.into(), self.reg.P.value);
    self.reg.S = self.reg.S.wrapping_sub(1);
    //println!("AFTER:");
    //self.debug_print_stack(bus);
    self.reg.PC = (bus.read(0xFFFA) as u16) | (bus.read(0xFFFB) as u16).wrapping_shl(8);
  }

  fn RTI(&mut self, bus: &mut Bus) {
    //self.debug_print_stack(bus);
    self.reg.S = self.reg.S.wrapping_add(1);
    let stack_addr = STACK_ADDR + self.reg.S as u16;
    self.reg.P.set_value(bus.read(stack_addr.into()));

    self.reg.S = self.reg.S.wrapping_add(1);
    let stack_addr = STACK_ADDR + self.reg.S as u16;
    let lsb = bus.read(stack_addr.into());

    self.reg.S = self.reg.S.wrapping_add(1);
    let stack_addr = STACK_ADDR + self.reg.S as u16;
    let msb = bus.read(stack_addr.into());

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

  //Illegal opcode:
  fn SLO(&mut self, bus: &mut Bus) {
    self.ASL(bus);
    self.ORA(bus);
  }
  fn RLA(&mut self, bus: &mut Bus) {
    self.ROL(bus);
    self.AND(bus);
  }
  fn SRE(&mut self, bus: &mut Bus) {
    self.LSR(bus);
    self.EOR(bus);
  }
  fn RRA(&mut self, bus: &mut Bus) {
    self.ROR(bus);
    self.ADC(bus);
  }
  fn SAX(&mut self, bus: &mut Bus) {
    bus.write(self.addr_abs.into(), self.reg.A & self.reg.X);
  }
  fn LAX(&mut self, bus: &mut Bus) {
    self.LDA(bus);
    self.LDX(bus);
  }
  fn DCP(&mut self, bus: &mut Bus) {
    self.DEC(bus);
    //print!("DCP{:#06x}:{:#04x},{:#04x}", self.addr_abs, bus.read(self.addr_abs.into()), self.operand[0]);
    self.CMP(bus);
  }
  fn ISC(&mut self, bus: &mut Bus) {
    self.INC(bus);
    self.SBC(bus);
  }
  //Imm + impl
  fn ANC(&mut self, bus: &mut Bus) {
    self.AND(bus);
    self.reg.P.set_C(if self.operand[0] & 0b1000_0000 != 0 {true} else {false});
  }
  fn ALR(&mut self, bus: &mut Bus) {
    self.AND(bus);
    self.LSR(bus);
  }
  fn ARR(&mut self, bus: &mut Bus) {
    self.AND(bus);
    self.ROR(bus);
  }
}
