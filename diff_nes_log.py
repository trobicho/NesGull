#!/usr/bin/python

class Instruction:
  def __init__(self):
    self.PC = "";
    self.X = "";
    self.Y = "";
    self.A = "";
    self.P = "";
    self.SP = "";
    self.OP1 = "";
    self.OP2 = "";
    self.opcode = "";
    self.PPU = ("", "");
    self.cycle = "";
    self.instr = "";

  def from_log(self, log_line):
    nb_opcode = 0;
    log = log_line.split();
    if (len(log[2]) == 2):
      self.OP1 = log[2];
      nb_opcode += 1;
    if (len(log[3]) == 2):
      self.OP2 = log[3];
      nb_opcode += 1;
    self.PC = log[0];
    self.opcode = log[1]
    self.instr = log[nb_opcode + 2]

    self.cycle = log[-1];
    self.PPU = log[-2];
    self.SP = log[-4].split(':')[1].upper();
    self.P = log[-5].split(':')[1].upper();
    self.Y = log[-6].split(':')[1].upper();
    self.X = log[-7].split(':')[1].upper();
    if (len(log[-8].split(':')) == 1):
      print(log_line);
    self.A = log[-8].split(':')[1].upper();

  def from_out(self, out_line):
    nb_opcode = 0;
    out = out_line.split();
    self.PC = out[0].split('x')[1].upper();
    if (len(out) >= 11):
      self.OP1 = out[2].split('x')[1].upper();
      nb_opcode = 1;
    if (len(out) == 12):
      self.OP2 = out[3].split('x')[1].upper();
      nb_opcode = 2;

    self.cycle = out[-1];
    self.PPU = out[-2];
    if (len(out[-4].split('x')) == 1):
      print(out_line);
    self.SP = out[-4].split('x')[1].upper();
    self.P = out[-5].split('x')[1].upper();
    self.Y = out[-6].split('x')[1].upper();
    self.X = out[-7].split('x')[1].upper();
    self.A = out[-8].split('x')[1].upper();

  def cmp(self, other):
    error = (0, "");
    if self.OP1 != other.OP1:
      error = (1, error[1] + (" OP1: " + self.OP1 + " " + other.OP1));
    if self.OP2 != other.OP2:
      error = (1, error[1] + (" OP2: " + self.OP2 + " " + other.OP2));
    if self.A != other.A:
      error = (1, error[1] + (" A: " + self.A + " " + other.A));
    if self.X != other.X:
      error = (1, error[1] + (" X: " + self.X + " " + other.X));
    if self.Y != other.Y:
      error = (1, error[1] + (" Y: " + self.Y + " " + other.Y));
    if self.SP != other.SP:
      error = (1, error[1] + (" SP: " + self.SP + " " + other.SP));
    if self.P != other.P:
      error = (1, error[1] + (" P: " + self.P + " " + other.P));
    if self.cycle != other.cycle:
      error = (1, error[1] + (" cycle: " + self.cycle + " " + other.cycle));
    if self.PPU != other.PPU:
      error = (1, error[1] + (" PPU: " + self.PPU + " " + other.PPU));
    if self.PC != other.PC:
      return (2, ("PC: " + self.PC + " " + other.PC) + error[1]);
    return error;

log_f = open("./nes-test-roms/other/nestest.log")
out_f = open("./out")

log_lines = log_f.readlines();
out_lines = out_f.readlines();

nb_error = 0;
last_error = (0, "");
for i in range(0, len(out_lines) - 1):
  log_split = log_lines[i].split();
  out_split = out_lines[i].split();

  instr_log = Instruction();
  instr_out = Instruction();

  instr_log.from_log(log_lines[i]);
  instr_out.from_out(out_lines[i]);

  error = instr_out.cmp(instr_log);

  if (error[0] != 0):
    if (last_error != error):
      print(str(i) + " " + log_lines[i - 1].split('\n')[0]);
      print(str(i) + " " + out_lines[i - 1].split('\n')[0]);
      print(error[1])
      print(str(i + 1) + " " + log_lines[i].split('\n')[0]);
      print(str(i + 1) + " " + out_lines[i].split('\n')[0]);
    #print(log_split);
    #print(out_split);
    print();
    nb_error += 1;
  if (error[0] == 2):
    break;
  last_error = error;
  if (i == len(log_lines) - 1):
    print("Congrats!!!");
    break;
  
