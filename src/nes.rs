mod cpu;
pub mod ppu;
mod memory;
mod bus;
mod mapper;
pub mod cartridge;
mod clock;


use std::error::Error;
use bus::Bus;
use cpu::CPU;
use ppu::{PPU, PPUInfo};
use cartridge::Cartridge;
use clock::{Clock, SlaveClock};

use sdl2::render::Canvas;

pub struct Nes {
  cpu: CPU,
  ppu: PPU,
  bus: Bus,
  cpu_clock: SlaveClock,
  ppu_clock: SlaveClock,
  cartridge: Cartridge,
}

impl Nes {
  pub fn new(cartridge: Cartridge) -> Self {
    Self {
      cpu: CPU::new(),
      ppu: PPU::new(),
      bus: Bus::new(&cartridge),
      cpu_clock: SlaveClock::new(12),
      ppu_clock: SlaveClock::new(4),
      cartridge,
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
  }

  pub fn render_frame(&mut self) -> &ppu::Frame{
    self.ppu.render_frame(&mut self.bus)
  }

  pub fn show_mem(&self) {
    //self.cpu.show_mem();
    println!("==============================");
    //self.ppu.show_mem();
  }

  pub fn tick(&mut self) {
    if self.ppu_clock.tick() {
      self.ppu.tick(&mut self.bus);
    }

    if self.cpu_clock.tick() {
      if self.cpu.tick(&mut self.bus) {
        let (s_index, cycles) = self.ppu.get_cycles_info();
        println!("\tPPU: {} {}\t{}", s_index, cycles
          , self.cpu.get_cycles_frame());
      }
    }
  }

  pub fn tick_n(&mut self, t: u32) {
    for _t in 0..t {
      self.tick();
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
