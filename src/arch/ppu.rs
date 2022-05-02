use std::fs;
use std::error::Error;

use sdl2::render::Canvas;

#[allow(non_snake_case)]
#[derive(Debug, Copy, Clone)]
pub struct NesColor {
  R: u8,
  G: u8,
  B: u8,
}

pub struct PPU {
  memory: [u8; 0x4000],
  palette: [NesColor; 64],
}

impl PPU {
  pub fn new() -> Self {
    Self {
      memory: [0; 0x4000],
      palette: [NesColor{R: 0, G: 0, B: 0}; 64],
    }
  }

  pub fn reset(&mut self) {
  }

  pub fn load(&mut self, chr_ram : &[u8], chr_size : usize) {
    self.memory[0x0..chr_size].clone_from_slice(&chr_ram);
  }

  pub fn load_palette(&mut self, filename: &str) -> Result<(), Box<dyn Error>> {
    let file = fs::read(filename)?;
    for i in 0..64 {
      self.palette[i] = NesColor {
        R: file[i * 3],
        G: file[i * 3 + 1],
        B: file[i * 3 + 2],
      };
      //println!("{:#2x} : {:?}", i, self.palette[i]);
    }
    Ok(())
  }

  pub fn render(&mut self, canvas: &mut Canvas<sdl2::video::Window>) {
    self.show_palette(canvas, sdl2::rect::Rect::new(0, 0, 16 * 20, 4 * 20));
    self.show_chr(canvas, sdl2::rect::Rect::new(0, 100, 256 * 4, 128 * 4));
  }

  //==============================DEBUG==================================
  pub fn show_palette(&mut self, canvas: &mut Canvas<sdl2::video::Window>
      , rect: sdl2::rect::Rect) {
    for i in 0..64 {
      let x: i32 = (i as i32 % 16) * rect.w / 16;
      let y: i32 = (i as i32 / 16) * rect.h / 4;
      canvas.set_draw_color(sdl2::pixels::Color::RGB(
        self.palette[i].R,
        self.palette[i].G,
        self.palette[i].B,
      ));
      canvas.fill_rect(sdl2::rect::Rect::new(rect.x + x, rect.y + y,
        (rect.w / 16) as u32, (rect.h / 4) as u32)
      ).unwrap();
    }
  }

  pub fn get_chr(&self, index: usize, chr : &mut [u8; 8 * 8]) {
    let addr = index * 16;
    for i in 0..8 {
      let value1 = self.memory[addr + i];
      let value2 = self.memory[addr + 8 + i];
      chr[7 + i * 8] = (value1 & 0b0000_0001) + (value2 & 0b0000_0001);
      chr[6 + i * 8] = ((value1 & 0b0000_0010) >> 1) + ((value2 & 0b0000_0010) >> 1);
      chr[5 + i * 8] = ((value1 & 0b0000_0100) >> 2) + ((value2 & 0b0000_0100) >> 2);
      chr[4 + i * 8] = ((value1 & 0b0000_1000) >> 3) + ((value2 & 0b0000_1000) >> 3);
      chr[3 + i * 8] = ((value1 & 0b0001_0000) >> 4) + ((value2 & 0b0001_0000) >> 4);
      chr[2 + i * 8] = ((value1 & 0b0010_0000) >> 5) + ((value2 & 0b0010_0000) >> 5);
      chr[1 + i * 8] = ((value1 & 0b0100_0000) >> 6) + ((value2 & 0b0100_0000) >> 6);
      chr[0 + i * 8] = ((value1 & 0b1000_0000) >> 7) + ((value2 & 0b1000_0000) >> 7);
    }
  }

  pub fn show_chr(&mut self, canvas: &mut Canvas<sdl2::video::Window>
      , rect: sdl2::rect::Rect) {
    let mut chr: [u8; 64] = [0; 64];
    let scale_x = rect.w / 256;
    let scale_y = rect.h / 128;
    for i in 0..256 {
      self.get_chr(i as usize, &mut chr);
      let x = i % 16 * (rect.w / 32);
      let y = i / 16 * (rect.h / 16);
      for ix in 0..8 {
        for iy in 0..8 {
          match chr[(ix + iy * 8) as usize] {
            0 => {canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0))}
            1 => {canvas.set_draw_color(sdl2::pixels::Color::RGB(80, 80, 80))}
            2 => {canvas.set_draw_color(sdl2::pixels::Color::RGB(220, 220, 220))}
            _ => {canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 16, 240))}
          }
          canvas.fill_rect(sdl2::rect::Rect::new(
            rect.x + x + ix * scale_x ,
            rect.y + y + iy * scale_y ,
            scale_x as u32, scale_y as u32
          )).unwrap();
        }
      }
    }
    for i in 0..256 {
      self.get_chr(256 + i as usize, &mut chr);
      let x = i % 16 * (rect.w / 32) + rect.w / 2;
      let y = i / 16 * (rect.h / 16);
      for ix in 0..8 {
        for iy in 0..8 {
          match chr[(ix + iy * 8) as usize] {
            0 => {canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0))}
            1 => {canvas.set_draw_color(sdl2::pixels::Color::RGB(80, 80, 80))}
            2 => {canvas.set_draw_color(sdl2::pixels::Color::RGB(220, 220, 220))}
            _ => {canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 16, 240))}
          }
          canvas.fill_rect(sdl2::rect::Rect::new(
            rect.x + x + ix * scale_x ,
            rect.y + y + iy * scale_y ,
            scale_x as u32, scale_y as u32
          )).unwrap();
        }
      }
    }
  }

  pub fn show_mem(&self) {
    println!("chr memory:");
    let mut i = 0;
    for addr in &self.memory[..] {
      if i % 16 == 0 {
        println!("");
      }
      print!("{} ", addr);
      i += 1;
    }
    println!("");
  }
  //==============================DEBUG==================================
}
