mod palette;
mod register;
pub mod memory;

use crate::nes::{
  memory::{Memory},
  bus::Bus,
  clock::Clock,
};
use palette::{Palette, NesColor};
use register::*;
use memory::*;

use std::error::Error;

const FRAME_HEIGHT_NTSC: usize = 240;
const FRAME_HEIGHT_PAL: usize = 240;
const FRAME_WIDTH: usize = 256;
const PATTERN_TABLE_ADDR: u16 = 0x0000;
const NAMETABLE_ADDR: u16 = 0x2000;
const STARTING_SCANLINE: u32 = 0;

pub struct Frame {
  pub width: usize,
  pub height: usize,
  pixels: Vec<u8>,
}

impl Frame {
  pub fn new(width: usize, height: usize) -> Self {
    Frame {
      width,
      height,
      pixels: vec![0; width * height * 4],
    }
  }

  fn put_pixel(&mut self, x: usize, y: usize, color: NesColor) {
    self.pixels[((self.width * y + x) << 2) + 3] = 255;
    self.pixels[((self.width * y + x) << 2) + 2] = color.R;
    self.pixels[((self.width * y + x) << 2) + 1] = color.G;
    self.pixels[((self.width * y + x) << 2) + 0] = color.B;
  }

  pub fn get_texture_buffer(&self) -> &Vec<u8> {
    &self.pixels
  }

  pub fn clear(&mut self) {
    self.pixels = vec![0; self.width * self.height * 4];
  }
}

#[derive(Debug, Copy, Clone)]
pub struct PPUInfo {
  pub frame_w: usize,
  pub frame_h: usize,
}

pub enum PPUEvent {
  FrameFinish,
  ScanlineFinish,
}

pub struct PPU {
  palette: Palette,
  frame: Frame,
  palette_mem: Memory,
  scanline_n : u32,
  cycle_n: u32,
  work_cycle: u32,
  rendering_enable: bool,
  reg: Register,
  frame_finish: bool,
  frame_n: u32,
  cur_oam: usize,
  sprite_overflow: bool,
  oam_offset: usize,
}

impl Clock<bool> for PPU {
  fn tick(&mut self, bus: &mut Bus) -> bool {
    let mut r = false;
    //self.debug_print(bus);
    if self.cycle_n == 0 {
      self.frame_finish = false;
      self.cycle_n += 1;
    }
    else {
      if self.handle_scanline(bus) {
        r = true
      }
      else {
        self.cycle_n += 1;
      }
    }
    r
  }
}

impl PPU {
  pub fn new() -> Self {
    Self {
      palette: Palette::new(),
      frame: Frame::new(FRAME_WIDTH, FRAME_HEIGHT_NTSC),
      palette_mem: Memory::ram(32),
      scanline_n : STARTING_SCANLINE,
      cycle_n: 0,
      work_cycle: 0,
      rendering_enable: true,
      reg: Register::new(),
      frame_finish: false,
      frame_n: 0,
      cur_oam: 0,
      sprite_overflow: false,
      oam_offset: 0,
    }
  }

  pub fn reset(&mut self) {
  }

  pub fn load_palette(&mut self, filename: &str) -> Result<(), Box<dyn Error>> {
    self.palette.change_from_file(filename)
  }

  pub fn get_frame_status(&self) -> bool {
    self.frame_finish
  }

  //DCBA98 76543210
  //---------------
  //0HRRRR CCCCPTTT
  //|||||| |||||+++- T: Fine Y offset, the row number within a tile
  //|||||| ||||+---- P: Bit plane (0: "lower"; 1: "upper")
  //|||||| ++++----- C: Tile column
  //||++++---------- R: Tile row
  //|+-------------- H: Half of sprite table (0: "left"; 1: "right")
  //+--------------- 0: Pattern table is at $0000-$1FFF

  fn scanline_vblank(&mut self, bus: &mut Bus) {
    if self.cycle_n == 1  && self.scanline_n == 241 {
      bus.ppu_mem.set_interupt(true);
      //println!("VBLANK (241, 1): {:#010b} {}", bus.ppu_mem.read_ctrl(), bus.ppu_mem.read_ctrl() & 0b1000_0000 != 0);
    }
  }

  fn scanline_261(&mut self, bus: &mut Bus) {
    if self.cycle_n == 1 {
      self.frame_finish = true;
      self.frame_n += 1;
      bus.ppu_mem.set_interupt(false);
      bus.ppu_mem.set_sprite_0hit(false);
    }
    if bus.ppu_mem.read_mask() & 0b0000_1000 != 0 {
      if self.cycle_n >= 280  && self.cycle_n <= 304 {
        self.vert_eq(bus);
      }
      self.scanline_fetch(bus);
    }
  }

  fn oam_handle(&mut self, bus: &mut Bus) {
    if self.cycle_n == 1 {
      self.oam_clear();
    }
    else {
      if self.cycle_n % 2 == 0 && self.cycle_n >= 64 && self.cycle_n <= 256 {
        self.oam_write_secondary(bus);
      }
      if self.cycle_n >= 257 && self.cycle_n <= 320 {
        bus.ppu_mem.oam_addr = 0x0;
      }
    }
  }

  fn scanline_fetch(&mut self, bus: &mut Bus) {
    if self.cycle_n < 258 || (self.cycle_n > 320  && self.cycle_n <= 336) {
      if self.cycle_n == 256 {
        self.vert_inc(bus);
      }
      if self.cycle_n == 257 {
        self.hori_eq(bus);
      }
      match self.work_cycle {
        0 => {
          //println!("{}: {:#018b}, {:#018b} v={:#06x}", self.cycle_n, self.reg.shift_back_16[0], self.reg.shift_back_16[1], bus.ppu_mem.v);
        },
        1 => {self.read_NT_byte(bus);}, //fetch NT_byte
        3 => {self.read_AT_byte(bus);}, //fetch AT_byte
        5 => {self.read_PT_low(bus);},
        7 => {
          if self.cycle_n != 256 {
            self.hori_inc(bus);
          }
          self.read_PT_high(bus);
        },
        _ => (),

      }
    }
    else if self.cycle_n == 320 {
      self.load_SP(bus);
    }
  }

  fn bg_color(&mut self, bus: &mut Bus) -> (usize, bool) {
    let x = bus.ppu_mem.x;
    let mut color_index : u16 = ((self.reg.shift_back_16[0] >> x) & 1) | (((self.reg.shift_back_16[1] >> x) & 1) << 1);
    if color_index != 0 {
      let mut addr = (0x2000 | (bus.ppu_mem.v & 0x0FFF)) - 2;
      let mut attr = self.reg.shift_back_8[0];
      if (self.work_cycle as u8) + x >= 8 {
        addr += 1;
        attr = self.reg.shift_back_8[1];
      }
      addr = bus.ppu_mem.mirroring(addr.into()) as u16;
      let quadrant: u16 = ((addr & 2) >> 1) | ((addr & 0b0000_0000_0100_0000) >> 5);
      color_index += (((attr >> (quadrant << 1)) & 3) << 2) as u16;
      //println!("quadrant = {} {:#4x} {} {}", quadrant, color_index, (self.reg.shift_back_8[0] >> (quadrant << 1)) & 3, (self.reg.shift_back_16[0] & 1) | ((self.reg.shift_back_16[1] & 1) << 1));
      (color_index as usize, true)
    }
    else {
      (0, false)
    }
  }

  fn sprite_color(&mut self, bus: &mut Bus) -> (usize, bool, bool){
    let mut color_index: u16 = 0;
    let mut priority = false;
    let mut sprite0 = false;
    let mut find_color = false;

    let mut i = 0;
    while i < 8 {
      if self.reg.counter_sprite[i].0 == 0 && self.reg.counter_sprite[i].1 < 8 {
        let temp_color_index: u16 = ((self.reg.shift_sprite_low[i] & 1) | ((self.reg.shift_sprite_high[i] & 1) << 1)).into();
        if temp_color_index != 0  && (!find_color || !priority) {
          if self.reg.counter_sprite[i].2 {
            sprite0 = true;
          }
          color_index = temp_color_index + (((self.reg.latch_sprite[i] & 3) << 2) as u16);
          priority = if self.reg.latch_sprite[i] & 0b0010_0000 != 0 {false} else {true};
          color_index += 0x10;
          find_color = true;
        }
        self.reg.shift_sprite_low[i] >>= 1;
        self.reg.shift_sprite_high[i] >>= 1;
        self.reg.counter_sprite[i].1 += 1;
      }
      i += 1;
    }
    (color_index as usize, priority, sprite0)
  }

  fn scanline_render(&mut self, bus: &mut Bus) {
    let mut bg_color: usize = 0x00;
    let mut sp_color: usize = 0x00;
    let mut priority = false;
    let mut sprite0 = false;
    let mut opaque_bg = false;

    if bus.ppu_mem.read_mask() & 0b0000_1000 != 0 {
      (bg_color, opaque_bg) = self.bg_color(bus);
      //println!("v={:#06x} {:#04x}, sbr[0]={:#018b}, sbr[1]={:#018b}", 0x2000 | (bus.ppu_mem.v & 0x0FFF), self.reg.NT_byte, self.reg.shift_back_16[0], self.reg.shift_back_16[0]);
    }
    if bus.ppu_mem.read_mask() & 0b0001_0000 != 0 {
      (sp_color, priority, sprite0) = self.sprite_color(bus);
    }

    let mut index: usize = (bus.ppu_read((bg_color + 0x3F00) as usize) % 64) as usize;
    index = if bus.ppu_mem.read_ctrl() & 1 == 1 {index & 0x30} else {index};
    let mut color = self.palette.color[index];

    if sprite0 && opaque_bg {
      bus.ppu_mem.set_sprite_0hit(true);
    }
    if (priority && sp_color != 0x00) || !opaque_bg {
      let mut index: usize = (bus.ppu_read((sp_color + 0x3F00) as usize) % 64) as usize;
      index = if bus.ppu_mem.read_ctrl() & 1 == 1 {index & 0x30} else {index};
      color = self.palette.color[index];
    }
    self.frame.put_pixel((self.cycle_n - 1) as usize, self.scanline_n as usize, color);
  }

  fn handle_scanline(&mut self, bus: &mut Bus) -> bool{
    match self.scanline_n {
      0..=239 => {
        if self.work_cycle == 0 {
          self.reg.load_shift_reg();
        }
        if self.cycle_n < 257 && self.scanline_n < self.frame.height as u32 {
          self.scanline_render(bus);
          self.reg.counter_dec();
        }
        if bus.ppu_mem.read_mask() & 0b0000_1000 != 0 {
          self.scanline_fetch(bus);
        }
        if bus.ppu_mem.read_mask() & 0b0001_0000 != 0 {
          self.oam_handle(bus);
        }
      },
      261 => {self.scanline_261(bus);},
      _ => {self.scanline_vblank(bus);}
    }
    if ((self.scanline_n >= 0 && self.scanline_n <= 239) || self.scanline_n == 261) &&
      ((self.cycle_n > 0 && self.cycle_n <= 256) || (self.cycle_n > 320 && self.cycle_n <= 336)) {
      self.reg.shift_back_reg();
    }
    self.work_cycle += 1;
    self.work_cycle %= 8;

    if self.cycle_n >= 340 {
      self.cycle_n = 0;
      self.work_cycle = 0;
      self.scanline_n = self.scanline_n.wrapping_add(1);
      if self.scanline_n > 261 {
        self.scanline_n = 0;
      }
      true
    }
    else {
      false
    }
  }

  pub fn get_frame(&self) -> &Frame {
    &self.frame
  }

  pub fn get_cycles_info(&self) -> (u32, u32) {
    (self.scanline_n, self.cycle_n)
  }

  pub fn render_info(&self) -> PPUInfo {
    PPUInfo{frame_w: self.frame.width
      , frame_h: self.frame.height}
  }
}


//PPU Instruction
#[allow(non_snake_case)]
impl PPU {
  fn oam_clear(&mut self) {
    self.reg.oam_clear_secondary();
    self.sprite_overflow = false;
    self.cur_oam = 0;
    self.oam_offset = 0;
  }

  fn sprite_y_in_range(&self, bus: &mut Bus, sprite_y: u32) -> (bool, bool){
    let max = if bus.ppu_mem.read_ctrl() & 0b0010_0000 != 0 {16} else {8};
    if self.scanline_n >= sprite_y && self.scanline_n < sprite_y + 8 {
      (true, false)
    } else if self.scanline_n >= sprite_y && self.scanline_n < sprite_y + max {
      (true, true)
    } else {
      (false, false)
    }
  }

  fn oam_write_secondary(&mut self, bus: &mut Bus) {
    if self.cur_oam < 64 {
      let sprite_y = bus.ppu_mem.oam_read(self.cur_oam, self.oam_offset);

      let (in_range, next_tile) = self.sprite_y_in_range(bus, sprite_y.into());
      if !self.sprite_overflow && in_range {
        self.sprite_overflow = !self.reg.oam_add(Oam{
          y: sprite_y,
          tile: bus.ppu_mem.oam_read(self.cur_oam, 1),
          attr: bus.ppu_mem.oam_read(self.cur_oam, 2),
          x: bus.ppu_mem.oam_read(self.cur_oam, 3),
          is_sprite0: self.cur_oam == 0,
        });
      }
      self.cur_oam += 1;
      if self.sprite_overflow {
        if in_range {
          bus.ppu_mem.set_sprite_overflow(true);
        }
        else {
          self.oam_offset += 1;
          self.oam_offset %= 4;
        }
      }
    }
  }

  fn load_SP(&mut self, bus: &mut Bus) {
    let ctrl = bus.ppu_mem.read_ctrl();
    let mode_16 = ctrl & 0b0010_0000 != 0;

    for i in 0..8 {
      let oam = self.reg.oam_secondary[i];
      if oam.y != 0xFF {
        let mut addr: usize = {
          if !mode_16 {
            ((oam.tile as u16) << 4) as usize
          }
          else {
            (((oam.tile as u16) & 0b1111_1110) << 4) as usize
          }
        };
        let mut bottom_tile = if (self.scanline_n as usize) - (oam.y as usize) >= 8 {true} else {false};
        if !mode_16 && ctrl & 0b0000_1000 != 0 {
          addr += 0x1000;
        }
        else if mode_16 && oam.tile & 1 == 1{
          addr += 0x1000;
        }
        if oam.attr & 0b1000_0000 != 0 {
          addr += 7 - (((self.scanline_n as usize) - (oam.y as usize)) % 8);
          if mode_16 {
            bottom_tile = !bottom_tile;
          }
        }
        else {
          addr += ((self.scanline_n as usize) - (oam.y as usize)) % 8;
        }
        if bottom_tile {
          addr += 16;
        }
        if oam.attr & 0b0100_0000 != 0 {
          self.reg.shift_sprite_low[i] = bus.ppu_read(addr.into());
          addr += 8;
          self.reg.shift_sprite_high[i] = bus.ppu_read(addr.into());
        }
        else {
          self.reg.shift_sprite_low[i] = revert_bits(bus.ppu_read(addr.into()));
          addr += 8;
          self.reg.shift_sprite_high[i] = revert_bits(bus.ppu_read(addr.into()));
        }
        self.reg.counter_sprite[i].0 = oam.x;
        self.reg.counter_sprite[i].1 = 0;
        self.reg.counter_sprite[i].2 = oam.is_sprite0;
        self.reg.latch_sprite[i] = oam.attr;
      }
      else {
        self.reg.shift_sprite_low[i] = 0x0;
        self.reg.shift_sprite_high[i] = 0x0;
        self.reg.counter_sprite[i].0 = 0xFF;
        self.reg.counter_sprite[i].1 = 0;
        self.reg.counter_sprite[i].2 = false;
        self.reg.latch_sprite[i] = 0x0;
      }
    }
  }

  //yyy NN YYYYY XXXXX
  //||| || ||||| +++++-- coarse X scroll
  //||| || +++++-------- coarse Y scroll
  //||| ++-------------- nametable select
  //+++----------------- fine Y scroll

  fn hori_inc(&mut self, bus: &mut Bus) {
    if bus.ppu_mem.v & 0x001F == 31 {
      bus.ppu_mem.v &= !0x001F;
      bus.ppu_mem.v ^= 0x0400;
    }
    else {
      bus.ppu_mem.v += 1;
    }
  }

  fn vert_inc(&mut self, bus: &mut Bus) {
    let mut v = bus.ppu_mem.v;
    if (v & 0x7000) != 0x7000 {
      v += 0x1000;
    }
    else {
      v &= !0x7000;
      let mut y = (v & 0x03E0) >> 5;
      if y == 29 {
        y = 0;
        v ^= 0x0800;
      }
      else if y == 31 {
        y = 0;
      }
      else {
        y += 1;
      }
      v = (v & !0x03E0) | (y << 5);
    }
    bus.ppu_mem.v = v;
  }

  fn hori_eq(&mut self, bus: &mut Bus) {
    bus.ppu_mem.v = (bus.ppu_mem.t & 0b0000_1100_0001_1111)
      | (bus.ppu_mem.v & 0b1111_0011_1110_0000);
  }

  fn vert_eq(&mut self, bus: &mut Bus) {
    bus.ppu_mem.v = (bus.ppu_mem.t & 0b0011_1011_1110_0000)
      | (bus.ppu_mem.v & 0b0000_0100_0001_1111);
  }

  fn read_NT_byte(&mut self, bus: &mut Bus) {
    let addr = 0x2000 | (bus.ppu_mem.v & 0x0FFF);
    //println!("PPU_DEBUG: {:#06x} = {}", addr, bus.ppu_read(addr.into()));
    self.reg.NT_byte = bus.ppu_read(addr.into());
  }

  fn read_AT_byte(&mut self, bus: &mut Bus) {
    let v = bus.ppu_mem.v;
    let addr = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07);
    self.reg.AT_byte = bus.ppu_read(addr.into());
  }

  fn read_PT_low(&mut self, bus: &mut Bus) {
    let mut addr = ((self.reg.NT_byte as u16) << 4)
      + ((bus.ppu_mem.v & 0b0111_0000_0000_0000) >> 12);
    addr += ((bus.ppu_mem.read_ctrl() & 0b0001_0000) as u16) << 8;
    self.reg.latch_PT[0] = revert_bits(bus.ppu_read(addr.into()));
    //println!("PPU_DEBUG: ctrl{:#010b} {:#06x} = {:#010b} NT_BYTE = {:#04x}", bus.ppu_mem.read_ctrl(), addr, self.reg.latch_PT[0], self.reg.NT_byte);
    //println!("PT_low: {:#06x}, {:#010b}", pattern_addr , self.reg.latch_PT[0]);
  }

  fn read_PT_high(&mut self, bus: &mut Bus) {
    let mut addr = ((self.reg.NT_byte as u16) << 4)
      + ((bus.ppu_mem.v & 0b0111_0000_0000_0000) >> 12);
    addr += ((bus.ppu_mem.read_ctrl() & 0b0001_0000) as u16) << 8;
    addr += 8;
    self.reg.latch_PT[1] = revert_bits(bus.ppu_read(addr.into()));
  }
}

impl PPU {
  pub fn debug_draw_nametable(&mut self, bus: &mut Bus
      , nametable: usize, x_offset: usize, y_offset: usize) {
    for y in 0..32 {
      for x in 0..32 {
        let addr: usize = nametable + x + y * 32;
        let mut palette = (bus.ppu_read(addr) % 64) as usize;
        if palette == 36 {
          palette = 63;
        }
        let color = self.palette.color[palette];
        self.frame.put_pixel(x + x_offset, y + y_offset, color);
      }
    }
  }

  pub fn debug_print_nametable(&mut self, bus: &mut Bus, nametable: usize) {
    println!("==========================================================");
    for addr in nametable..nametable + 32 * 32 {
      if addr % 32 == 0 {
        println!("");
      }
      let value = bus.ppu_read(addr);
      print!("{:#04x}, ", value);
    }
  }

  pub fn debug_show_nametable(&mut self, bus: &mut Bus) -> &Frame {
    //self.frame.clear();
    self.debug_draw_nametable(bus, 0x2000, 0, 0);
    //self.debug_print_nametable(bus, 0x2000);
    self.debug_draw_nametable(bus, 0x2400, 35, 0);
    self.debug_draw_nametable(bus, 0x2800, 0, 35);
    self.debug_draw_nametable(bus, 0x2C00, 35, 35);
    &self.frame
  }

  pub fn debug_print(&self, bus: &mut Bus) {
    print!("PPU({}, {}, {})", self.scanline_n, self.cycle_n, self.work_cycle);
    print!(" t:{:#06x} v:{:#06x}", bus.ppu_mem.t, bus.ppu_mem.v);
    print!(" ctrl:{:#04x} mask{:#04x} status:{:#04x}", bus.ppu_mem.read_ctrl(), bus.ppu_mem.read_mask(), bus.ppu_mem.read_status());
    print!(" x:{:#04x}", bus.ppu_mem.x);
    print!(" NT_byte:{:#04x} AT_byte{:#04x} BSR[0]{:#018b}", self.reg.NT_byte, self.reg.AT_byte, self.reg.shift_back_16[0]);
    println!("");
  }
}

fn revert_bits(v: u8) -> u8 {
  ((v & 0b0000_0001) << 7)
  | ((v & 0b0000_0010) << 5)
  | ((v & 0b0000_0100) << 3)
  | ((v & 0b0000_1000) << 1)
  | ((v & 0b0001_0000) >> 1)
  | ((v & 0b0010_0000) >> 3)
  | ((v & 0b0100_0000) >> 5)
  | ((v & 0b1000_0000) >> 7)
}
