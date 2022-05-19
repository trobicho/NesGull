#![allow(dead_code)]
extern crate sdl2;

mod nes;
mod rom;

use std::env;
use std::error::Error;
use std::time::Duration;
use std::time::Instant;

use nes::{Nes, DebugEvent};
use nes::cartridge::Cartridge;
use nes::controller::{Controller, basic::NesController};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{TextureCreator};
use sdl2::rect::Rect;
use sdl2::controller::GameController;
use sdl2::GameControllerSubsystem;

const millis_per_frame : u128 = (1_000_000.0 / 60.0988) as u128; 

fn find_sdl_gl_driver() -> Option<u32> {
  for (index, item) in sdl2::render::drivers().enumerate() {
    if item.name == "opengl" {
      return Some(index as u32);
    }
  }
  None
}

fn find_controller(game_controller_subsystem: &GameControllerSubsystem) -> Result<GameController, Box<dyn Error>> {
  let available = game_controller_subsystem
    .num_joysticks()
    .map_err(|e| format!("can't enumerate joysticks: {}", e))?;

  println!("{} joysticks available", available);

  // Iterate over all available joysticks and look for game controllers.
  let mut controller = (0..available)
    .find_map(|id| {
        if !game_controller_subsystem.is_game_controller(id) {
        println!("{} is not a game controller", id);
        return None;
        }

        println!("Attempting to open controller {}", id);

        match game_controller_subsystem.open(id) {
        Ok(c) => {
        // We managed to find and open a game controller,
        // exit the loop
        println!("Success: opened \"{}\"", c.name());
        Some(c)
        }
        Err(e) => {
        println!("failed: {:?}", e);
        None
        }
        }
        })
  .expect("Couldn't open any controller");
  println!("Controller mapping: {}", controller.mapping());
  Ok(controller)
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
  let game_controller_subsystem = sdl_context.game_controller()?;
  let mut controller = NesController::new(find_controller(&game_controller_subsystem)?, 0);

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
  let mut nes = Nes::new(Cartridge::create_from_rom(&nes_rom), Box::new(controller))?;
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
  let mut run = false;
  let mut show_nametable = false;
  let mut cpu_debug = false;
  let mut frame_nb = 0;
  let mut time = Instant::now();
  let mut last_time = time;
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
        Event::KeyDown {keycode: Some(Keycode::P), ..} => {
          nes.debug_event(DebugEvent::SHOW_PPU_PALETTE);
        },
        Event::KeyDown {keycode: Some(Keycode::R), ..} => {
          run = !run;
        },
        Event::KeyDown {keycode: Some(Keycode::N), ..} => {
          show_nametable = !show_nametable;
        },
        Event::KeyDown {keycode: Some(Keycode::D), ..} => {
          cpu_debug = !cpu_debug;
          nes.set_cpu_debug(cpu_debug);
        },
        _ => {},
      }
    }
    if run {
      let elapsed = time.elapsed().as_micros();
      if elapsed < millis_per_frame {
        std::thread::sleep(Duration::from_micros((millis_per_frame - elapsed).try_into().unwrap()));
      }
      time = Instant::now();
      nes.tick_frame();
      frame_nb += 1;
      println!("frame: {}", frame_nb);
    }
    canvas.set_draw_color(sdl2::pixels::Color::RGBA(200, 150, 0, 255));
    canvas.clear();
    let frame = if (show_nametable) {nes.get_debug_frame()} else {nes.get_frame()};
    frame_texture.update(None, frame.get_texture_buffer(), frame.width * 4)?;
    canvas.copy(&frame_texture, None, None)?;
    canvas.present();
    //for _ in 0..=10 {
      //nes.tick_scanline();
    //}
  }
  Ok(())
}
