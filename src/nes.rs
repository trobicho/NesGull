mod cpu;
pub mod ppu;
pub mod apu;
mod memory;
pub mod save_state;
mod bus;
mod mapper;
pub mod cartridge;
pub mod controller;
mod clock;

use std::error::Error;

use bus::Bus;
use save_state::SaveState;
use cpu::CPU;
use ppu::{PPU, PPUInfo};
use apu::{APU};
use apu::mixer::Mixer;
use cartridge::Cartridge;
use clock::{Clock, SlaveClock};
use controller::Controller;
use mapper::{Mapper};

#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub enum DebugEvent {
  SHOW_PPU_REG,
  SHOW_PPU_OAM,
  SHOW_PPU_VRAM,
  SHOW_PPU_PALETTE,
  SHOW_CPU_WRAM,
  SHOW_APU_REG,
  SHOW_MAPPER,
  MUTE_GAME,
}

pub struct DebugFlag {
  show_instr: bool,
}

pub struct Nes {
  cpu: CPU,
  ppu: PPU,
  apu: APU,
  bus: Bus,
  cpu_clock: SlaveClock,
  ppu_clock: SlaveClock,
  apu_clock: SlaveClock,
  cartridge: Cartridge,

  cpu_nmi: bool,
  debug_no_nmi: bool,
  breakpoint: bool,
  mute: bool,
}

impl Nes {
  pub fn new(cartridge: Cartridge, controller: Box<dyn Controller>, mixer: Mixer) -> Result<Self, Box<dyn Error>> {
    let mut new = Self {
      cpu: CPU::new(),
      ppu: PPU::new(),
      apu: APU::new(),
      bus: Bus::new(controller, mixer),
      cpu_clock: SlaveClock::new(3),
      ppu_clock: SlaveClock::new(1),
      apu_clock: SlaveClock::new(6),
      cartridge,

      cpu_nmi: false,
      debug_no_nmi: false,
      breakpoint: false,
      mute: false,
    };
    let mapper = mapper::load_rom(&new.cartridge)?;
    new.bus.load_mapper(mapper);
    new.bus.mixer.set_mute(new.mute);
    Ok(new)
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
        if self.cpu.debug {
          println!("\tPPU: {},{}\tCYC:{}", s_index, cycles, self.cpu.get_cycles_frame());
          //self.bus.print_ppu_reg();
        }
      }
    }
    if self.apu_clock.tick() {
      self.apu.tick(&mut self.bus);
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

  pub fn set_cpu_debug(&mut self, debug: bool) {
   self.cpu.set_debug(debug); 
  }
}

impl Nes {
  pub fn debug_event(&mut self, event: DebugEvent) {
    match event {
      DebugEvent::SHOW_CPU_WRAM => {println!("{}", self.bus.wram);},
      DebugEvent::SHOW_APU_REG => {println!("{}", self.bus.apu_mem);},
      DebugEvent::SHOW_PPU_REG => {self.bus.print_ppu_mem();},
      DebugEvent::SHOW_PPU_VRAM => {},
      DebugEvent::SHOW_PPU_OAM=> {},
      DebugEvent::SHOW_PPU_PALETTE => {self.bus.ppu_mem.print_palette();},
      DebugEvent::MUTE_GAME => {
        self.mute = !self.mute;
        self.bus.mixer.set_mute(self.mute);
        println!("Game mute {}", self.mute);
      },
      //DebugEvent::SHOW_MAPPER => {println!("{}", self.bus.mapper);},
      _ => (),
    }
  }

  pub fn debug_save_state(&self) -> SaveState {
    let mut state = self.bus.save_state();
    state.cpu_reg = self.cpu.save_reg_state();
    state
  }

  pub fn debug_load_state(&mut self, state: &SaveState) {
    self.bus.load_state(state);
    self.cpu.load_reg_state(&state.cpu_reg);
  }
}
