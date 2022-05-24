use sdl2::controller::{GameController, Button};
use crate::nes::{
  controller::Controller,
};

#[allow(non_snake_case)]
pub struct NesController {
  A: bool,
  B: bool,
  Start: bool,
  Select: bool,
  DPad: (bool, bool, bool, bool),

  controller: GameController,
  opposite_dpad: bool,
  input: u8,
  output: u8,
  port: usize,
  report_count: u8,
  strobe: bool,
}

impl NesController {
  pub fn new(controller: GameController, port: usize) -> Self {
    Self {
      A: false,
      B: false,
      Start: false,
      Select: false,
      DPad: (false, false, false, false),
      controller,
      opposite_dpad: false,
      input: 0x00,
      output: 0x00,
      port,
      report_count: 0,
      strobe: false,
    }
  }

  pub fn set_opposite_dpad(&mut self, v: bool) {
    self.opposite_dpad = v;
  }

  fn dpad_active(&self) -> bool {
    if self.DPad.0 || self.DPad.1 || self.DPad.2 || self.DPad.3 {
      true
    } else {
      false
    }
  }

  fn set_strobe(&mut self, value: bool) {
    //println!("Strobe Set to {}", value);
    self.strobe = value;
  }

  fn port_report(&mut self) -> u8 {
    if (self.strobe) {
      self.report_count = 0;
    }
    let report = match (self.report_count) {
      0 => self.A,
      1 => self.B,
      2 => self.Select,
      3 => self.Start,
      4 => self.DPad.0,
      5 => self.DPad.1,
      6 => self.DPad.2,
      7 => self.DPad.3,
      _ => true,
    };
    self.report_count += 1;
    let report: u8 = if (report) {1} else {0};
    report
  }
}

impl Controller for NesController {
  fn read(&mut self, addr: usize) -> u8{
    let port_addr = if self.port % 2 == 0 {0x4016} else {0x4017};
    if addr == port_addr {
      let r = self.port_report();
      //println!("read controller{:#06x}, report:{}", addr, r);
      r
    } else {
      0
    }
  }

  fn write(&mut self, addr: usize, value: u8) {
    match (addr) {
      0x4016 => {self.set_strobe(if value & 1 == 1 {true} else {false})}
      _ => (),
    }
  }

  fn update(&mut self) {
    self.A = self.controller.button(Button::X);
    self.B = self.controller.button(Button::B);
    self.Start = self.controller.button(Button::Start);
    self.Select = self.controller.button(Button::Back);
    self.DPad.0 = self.controller.button(Button::DPadUp);
    self.DPad.1 = self.controller.button(Button::DPadDown);
    self.DPad.2 = self.controller.button(Button::DPadLeft);
    self.DPad.3 = self.controller.button(Button::DPadRight);
    if self.opposite_dpad == false {
      if self.DPad.0 && self.DPad.1 {
        self.DPad.1 = false;
      }
      if self.DPad.2 && self.DPad.3 {
        self.DPad.2 = false;
      }
    }
    self.report_count = 0;
  }

  fn debug_print(&self) {
    if self.A {print!("A:{}, ", self.A);}
    if self.B {print!("B:{}, ", self.B);}
    if self.Start {print!("Start:{}, ", self.Start);}
    if self.Select {print!("Select:{}, ", self.Select);}
    if self.dpad_active() {
      print!("DPAD:[left:{}, Down:{}, Right:{}, Up:{}]"
        , self.DPad.0, self.DPad.1, self.DPad.2, self.DPad.3);
    }
    if (self.A || self.B || self.dpad_active()) {
      println!("");
    }
  }
}
