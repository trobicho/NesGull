#![allow(dead_code)]
extern crate sdl2;

mod nes;
mod rom;

use std::error::Error;
use nes::{Nes};
use nes::cartridge::Cartridge;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::rect::Rect;

fn find_sdl_gl_driver() -> Option<u32> {
  for (index, item) in sdl2::render::drivers().enumerate() {
    if item.name == "opengl" {
      return Some(index as u32);
    }
  }
  None
}

fn main() -> Result<(), Box<dyn Error>>{
  let sdl_context = sdl2::init()?;
  let video_subsystem = sdl_context.video()?;
  let window = video_subsystem.window("NES emulator", 1200, 800)
    .opengl()
    .build()
    .map_err(|e| e.to_string())?;
  let mut canvas = window.into_canvas()
    .index(find_sdl_gl_driver().unwrap())
    .build()
    .map_err(|e| e.to_string())?;

  let mut event_pump = sdl_context.event_pump()?;

  //let nes_rom = rom::nes_rom_load("./roms/Bomberman (USA).nes")?;
  //let nes_rom = rom::nes_rom_load("./roms/Donkey Kong Classics (USA, Europe).nes")?;
  let nes_rom = rom::nes_rom_load("./nes-test-roms/other/nestest.nes")?;
  let mut nes = Nes::new(Cartridge::create_from_rom(&nes_rom));
  nes.debug_reset();
  //nes.reset();
  nes.load_palette("./palettes/ntscpalette.pal")?;
  //println!("=============================");
  //nes.show_mem();
  //nes.run();
  //println!("=============================");

  let texture_creator: TextureCreator<_> = canvas.texture_creator();
  let ppu_info = nes.ppu_rendering_info();
  let mut frame_texture = texture_creator
    .create_texture_target(None, ppu_info.frame_w as u32, ppu_info.frame_h as u32)
    .map_err(|e| e.to_string())?;

  nes.render_frame();
  nes.tick_n(20000 * 12);
  let (height, width) = canvas.output_size()?;
  let frame_rect = Rect::new(0, 0, width as u32, height as u32);
  let mut running = true;
  while running {
    for event in event_pump.poll_iter() {
      match event {
        Event::Quit { .. }
        | Event::KeyDown {
            keycode: Some(Keycode::Escape),
           ..
        } => {
          running = false;
        },
        Event::KeyDown {keycode: Some(Keycode::Space), ..} => {
          nes.tick_n(12);
        }
        _ => {}
      }
    }
    canvas.set_draw_color(sdl2::pixels::Color::RGBA(200, 150, 0, 255));
    canvas.clear();
    let frame = nes.get_frame();
    frame_texture.update(frame_rect, frame.get_texture_buffer(), frame.width * 4);
    canvas.copy(&frame_texture, None, frame_rect)?;
    canvas.present();
    std::thread::sleep(Duration::from_millis(100));
  }
  Ok(())
}
