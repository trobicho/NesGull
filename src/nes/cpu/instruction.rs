fn ADC(&mut self, instr: Instruction) -> i32 {
  match instr.mode {
    OpMode::IMM =>
  }
}

fn AND(&mut self, instr: Instruction) -> i32 { //Instruction fn should receive operand dirrectly even if pointer
  match instr.mode {
    OpMode::IMM => self.reg.A &= self.operand[0],
  }
}
