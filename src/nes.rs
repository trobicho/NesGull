mod cpu;
mod old_ppu;
mod memory;
mod bus;
mod mapper;
pub mod cartridge;


use std::error::Error;
use bus::Bus;
use cpu::CPU;
use old_ppu::PPU;
use cartridge::Cartridge;

use sdl2::render::Canvas;

pub struct Nes {
  cpu : CPU,
  ppu : PPU,
  bus : Bus,
  cartridge : Cartridge,
}

impl Nes {
  pub fn new(cartridge: Cartridge) -> Self {
    Self {
      cpu: CPU::new(),
      ppu: PPU::new(),
      bus: Bus::new(&cartridge),
      cartridge,
    }
  }

  pub fn load_palette(&mut self, filename: &str) -> Result<(), Box<dyn Error>> {
    self.ppu.load_palette(filename)
  }

  pub fn reset_debug(&mut self) {
    self.cpu.reset_debug(&mut self.bus);
    match &self.cartridge.chr_rom {
      Some(chr_rom) => {self.ppu.load(&chr_rom, self.cartridge.chr_size)},
      None => {}
    }
    self.ppu.reset();
  }

  pub fn reset(&mut self) {
    self.cpu.reset(&mut self.bus);
    match &self.cartridge.chr_rom {
      Some(chr_rom) => {self.ppu.load(&chr_rom, self.cartridge.chr_size)},
      None => {}
    }
    self.ppu.reset();
  }

  pub fn render(&mut self, canvas: &mut Canvas<sdl2::video::Window>) {
    self.ppu.render(canvas);
  }

  pub fn show_mem(&self) {
    //self.cpu.show_mem();
    println!("==============================");
    //self.ppu.show_mem();
  }

  pub fn run(&mut self) {
    for _ in 0..10 {
      self.cpu.debug_read_instr(&mut self.bus);
    }
  }

  pub fn run_step(&mut self) {
    self.cpu.debug_exec_instr(&mut self.bus);
  }
}
