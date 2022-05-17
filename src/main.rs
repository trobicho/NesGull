#![allow(dead_code)]
extern crate sdl2;

mod nes;
mod rom;

use std::env;
use std::error::Error;

use nes::{Nes, DebugEvent};
use nes::cartridge::Cartridge;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use sdl2::render::{TextureCreator};
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
  let window = video_subsystem.window("NES emulator", 256 * 4, 224 * 4)
    .opengl()
    .build()
    .map_err(|e| e.to_string())?;
  let mut canvas = window.into_canvas()
    .index(find_sdl_gl_driver().unwrap())
    .build()
    .map_err(|e| e.to_string())?;

  let mut event_pump = sdl_context.event_pump()?;

  let args: Vec<String> = env::args().collect();
  println!("{:?}", args);
  let mut nes_rom: Vec<u8> = vec![0];
  if args.len() > 1 {
    nes_rom = rom::nes_rom_load(&args[1])?;
    println!("rom loaded: {}", &args[1]);
  }
  else {
    nes_rom = rom::nes_rom_load("./roms/Donkey Kong (U) (PRG1) [!p].nes")?;
  }

  //let nes_rom = rom::nes_rom_load("./roms/Donkey Kong Classics (USA, Europe).nes")?;
  //let nes_rom = rom::nes_rom_load("./roms/Donkey Kong (Japan).nes")?;
  //let nes_rom = rom::nes_rom_load("./roms/Mega Man (USA).nes")?;

  //let nes_rom = rom::nes_rom_load("./nes-test-roms/other/nestest.nes")?;
  //let nes_rom = rom::nes_rom_load("./nes-test-roms/scanline/scanline.nes")?;
  //let nes_rom = rom::nes_rom_load("././nes-test-roms/nmi_sync/demo_ntsc.nes")?;
  let mut nes = Nes::new(Cartridge::create_from_rom(&nes_rom));
  nes.reset();
  //nes.debug_reset();
  nes.load_palette("./palettes/ntscpalette.pal")?;
  //nes.load_palette("./palettes/SMM Palette 1.0.pal")?;
  //println!("=============================");
  //nes.show_mem();
  //nes.run();
  //println!("=============================");

  let texture_creator: TextureCreator<_> = canvas.texture_creator();
  let ppu_info = nes.ppu_rendering_info();
  let mut frame_texture = texture_creator
    .create_texture_target(None, ppu_info.frame_w as u32, ppu_info.frame_h as u32)
    .map_err(|e| e.to_string())?;

  let (height, width) = canvas.output_size()?;
  let frame_rect = Rect::new(0, 0, width as u32, height as u32);
  let mut running = true;
  let mut run = true;
  let mut show_nametable = false;
  let mut frame_nb = 0;
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
          run = false;
          nes.tick_n(4 * 8);
        },
        Event::KeyDown {keycode: Some(Keycode::F), ..} => {
          run = false;
          nes.tick_frame();
        },
        Event::KeyDown {keycode: Some(Keycode::S), ..} => {
          run = false;
          nes.tick_scanline();
        },
        Event::KeyDown {keycode: Some(Keycode::W), ..} => {
          nes.debug_event(DebugEvent::SHOW_CPU_WRAM);
        },
        Event::KeyDown {keycode: Some(Keycode::M), ..} => {
          nes.debug_event(DebugEvent::SHOW_PPU_VRAM);
        },
        Event::KeyDown {keycode: Some(Keycode::R), ..} => {
          run = !run;
        },
        Event::KeyDown {keycode: Some(Keycode::N), ..} => {
          show_nametable = !show_nametable;
        },
        _ => {},
      }
    }
    canvas.set_draw_color(sdl2::pixels::Color::RGBA(200, 150, 0, 255));
    canvas.clear();
    let frame = if (show_nametable) {nes.get_debug_frame()} else {nes.get_frame()};
    frame_texture.update(None, frame.get_texture_buffer(), frame.width * 4)?;
    canvas.copy(&frame_texture, None, None)?;
    canvas.present();
    if run {
      for _ in 0..10 {
        nes.tick_frame();
      }
      frame_nb += 10;
      println!("frame: {}", frame_nb);
    }
    //for _ in 0..=10 {
      //nes.tick_scanline();
    //}
  }
  Ok(())
}
