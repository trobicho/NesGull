mod cpu;
pub mod ppu;
mod memory;
mod bus;
mod mapper;
pub mod cartridge;
pub mod controller;
mod clock;

use std::error::Error;
use bus::Bus;
use cpu::CPU;
use ppu::{PPU, PPUInfo};
use cartridge::Cartridge;
use clock::{Clock, SlaveClock};
use controller::Controller;

#[allow(non_camel_case_types)]
pub enum DebugEvent {
  SHOW_PPU_REG,
  SHOW_PPU_OAM,
  SHOW_PPU_VRAM,
  SHOW_PPU_PALETTE,
  SHOW_CPU_WRAM,
  SHOW_CPU_MAP_REG,
  SHOW_MAPPER,
}

pub struct DebugFlag {
  show_instr: bool,
}

pub struct Nes {
  cpu: CPU,
  ppu: PPU,
  bus: Bus,
  cpu_clock: SlaveClock,
  ppu_clock: SlaveClock,
  cartridge: Cartridge,

  cpu_nmi: bool,
  debug_no_nmi: bool,
  breakpoint: bool,
}

impl Nes {
  pub fn new(cartridge: Cartridge, controller: Box<dyn Controller>) -> Self {
    Self {
      cpu: CPU::new(),
      ppu: PPU::new(),
      bus: Bus::new(&cartridge, controller),
      cpu_clock: SlaveClock::new(3),
      ppu_clock: SlaveClock::new(1),
      cartridge,

      cpu_nmi: false,
      debug_no_nmi: false,
      breakpoint: false,
    }
  }

  pub fn load_palette(&mut self, filename: &str) -> Result<(), Box<dyn Error>> {
    self.ppu.load_palette(filename)
  }

  pub fn debug_reset(&mut self) {
    self.cpu.debug_reset(&mut self.bus);
    self.ppu.reset();
  }

  pub fn reset(&mut self) {
    self.cpu.reset(&mut self.bus);
    self.ppu.reset();
    self.bus.mapper.debug_print_vec();
  }

  pub fn get_frame(&self) -> &ppu::Frame{
    self.ppu.get_frame()
  }

  pub fn get_debug_frame(&mut self) -> &ppu::Frame{
    self.ppu.debug_show_nametable(&mut self.bus)
  }

  pub fn tick(&mut self) -> bool{
    let mut b = false;

    self.breakpoint = false;
    if self.ppu_clock.tick() {
      b = self.ppu.tick(&mut self.bus);
    }

    if self.cpu_clock.tick() {
      if self.cpu.tick(&mut self.bus) {
        let (s_index, cycles) = self.ppu.get_cycles_info();
        //println!("\tPPU: {},{}\tCYC:{}", s_index, cycles, self.cpu.get_cycles_frame());
        //self.bus.print_ppu_reg();
      }
    }
    b
  }

  pub fn tick_n(&mut self, t: u32) {
    for _t in 0..t {
      self.tick();
    }
  }

  pub fn tick_scanline(&mut self) {
    loop {
      if self.tick(){
        break;
      }
    }
  }

  pub fn tick_scanline_n(&mut self, t: u32) {
    for _t in 0..t {
      self.tick_scanline();
    }
  }

  pub fn tick_frame(&mut self) {
    loop {
      if self.tick() && self.ppu.get_frame_status() {
        self.bus.input.update();
        self.bus.input.debug_print();
        break;
      }
      if self.breakpoint {
        self.breakpoint = false;
        break;
      }
    }
  }
  /*
  pub fn run_step(&mut self) {
    let cycles = self.cpu.cycles_since_startup();
    self.cpu.debug_exec_instr(&mut self.bus);
    println!("\t\t{}", cycles);
  }
  */

  pub fn ppu_rendering_info(&self) -> PPUInfo {
    self.ppu.render_info()
  }
}

impl Nes {
  pub fn debug_event(&mut self, event: DebugEvent) {
    match event {
      DebugEvent::SHOW_CPU_WRAM => {println!("{}", self.bus.wram);},
      DebugEvent::SHOW_CPU_MAP_REG=> {println!("{}", self.bus.cpu_mapped_reg);},
      DebugEvent::SHOW_PPU_REG => {self.bus.print_ppu_mem();},
      DebugEvent::SHOW_PPU_VRAM => {;},
      DebugEvent::SHOW_PPU_OAM=> {;},
      DebugEvent::SHOW_PPU_PALETTE => {self.bus.ppu_mem.print_palette();},
      DebugEvent::SHOW_MAPPER => {println!("{}", self.bus.mapper);},
    }
  }
}
