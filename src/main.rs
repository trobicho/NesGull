#![allow(dead_code)]
extern crate sdl2;

mod arch;
mod rom;

use std::error::Error;
use arch::Nes;
use arch::Cartridge;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Point;
use sdl2::rect::Rect;
use std::time::Duration;

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
  let window = video_subsystem.window("Window", 800, 600)
    .opengl() // this line DOES NOT enable opengl, but allows you to create/get an OpenGL context from your window.
    .build()
    .map_err(|e| e.to_string())?;
  let mut canvas = window.into_canvas()
    .index(find_sdl_gl_driver().unwrap())
    .build()
    .map_err(|e| e.to_string())?;

  let texture_creator = canvas.texture_creator();
  canvas.set_draw_color(sdl2::pixels::Color::RGBA(0, 0, 0, 255));
  let timer = sdl_context.timer()?;
  let mut event_pump = sdl_context.event_pump()?;

  println!("Hello, world!");
  let nes_rom = rom::nes_rom_load("./roms/Bomberman (USA).nes")?;
  let cartridge = Cartridge::create_from_rom(&nes_rom);
  let mut nes = Nes::load_rom(&cartridge);
  nes.reset();
  nes.load_palette("./palettes/ntscpalette.pal")?;
  println!("=============================");
  //nes.show_mem();
  nes.run();
  println!("=============================");

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
        }
        _ => {}
      }
    }
    canvas.present();
    std::thread::sleep(Duration::from_millis(100));
  }
  Ok(())
}
