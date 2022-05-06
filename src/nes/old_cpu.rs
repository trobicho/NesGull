mod opcode;


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

pub struct CPU {
  reg: Reg,
  memory: [u8; 0x10000], //0 to 0xFFFF
  cycles_since_startup: u32,
  cycles_instr: u32,
  operand: [u8; 2],
  addr_abs: u16,
  addr_rel: u16,
}

fn operand_lenght(instr: &opcode::InstructionInfo) -> u16 {
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

impl CPU {
  pub fn new() -> Self{
    Self {
      reg: Reg::new(),
      memory: [0; 0x10000],
      cycles_since_startup: 0,
      cycles_instr: 0,
      operand: [0, 0],
      addr_abs: 0,
      addr_rel: 0,
    }
  }

  pub fn load(&mut self, prg_ram: &[u8], prg_size: usize) {
    self.memory[0x8000..0x8000 + prg_size].clone_from_slice(&prg_ram);
    if prg_size == 0x4000 {
      self.memory[0xC000..0xC000 + prg_size].clone_from_slice(&prg_ram);
    }
    println!("prg_size: {:#04x}", prg_size);
  }

  pub fn reset(&mut self) {
    self.reg.reset();
    self.cycles_since_startup = 0;
    self.cycles_instr = 0;

    self.reg.PC = ((self.memory[0xFFFD] as u16) << 8) + self.memory[0xFFFC] as u16;
    println!("PC : {:#04x}", self.reg.PC);
  }

  fn mirroring(&mut self, addr : u16) {
    let value = self.memory[addr as usize];
    if addr < 0x2000 {
      let addr_mod = addr % 0x800;
      self.memory[addr_mod as usize] = value;
      self.memory[addr_mod as usize + 0x0800] = value;
      self.memory[addr_mod as usize + 0x1000] = value;
      self.memory[addr_mod as usize + 0x1800] = value;
    }
  }

  fn write_memory(&mut self, addr : u16, value : u8) {
    self.memory[addr as usize] = value;
    self.mirroring(addr);
  }

  fn read_memory(&self, addr : u16) -> u8{
    self.memory[addr as usize]
  }

  pub fn debug_read_instr(&mut self) {
    let instr = self.read_instr();
    print!("{}", instr.0);
    let op_len = operand_lenght(&instr.1);
    if op_len >= 1 {
      print!(": {:#02x}", self.operand[0]);
      if op_len == 2 {
        print!(", {:#02x}", self.operand[1]);
      }
    }
    println!("");
  }

  fn read_instr(&mut self) -> (opcode::Instruction, opcode::InstructionInfo) {
    let opcode = self.read_memory(self.reg.PC);
    let instr = opcode::opcode_to_enum(opcode);
    let op_len = operand_lenght(&instr.1);
    if op_len >= 1 {
      self.operand[0] = self.read_memory(self.reg.PC + 1);
    }
    if op_len == 2 {
      self.operand[1] = self.read_memory(self.reg.PC + 2);
    }
    self.reg.PC += op_len + 1;
    instr
  }

  pub fn show_mem(&self) {
    println!("prg memory:");
    let mut i = 0;
    for addr in &self.memory[0x8000..] {
      if i % 16 == 0 {
        println!("");
      }
      print!("{} ", addr);
      i += 1;
    }
    println!("");
  }
}

#[allow(non_snake_case)]
impl CPU {
  fn operand_addressing_mode(&self, instr: opcode::InstructionInfo) -> u8 {
    8 as u8
  }

  fn ADC(&mut self, instr: opcode::InstructionInfo) {
    //match instr.mode {
    //}
  }
}
