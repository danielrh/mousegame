extern crate sdl2;
extern crate art_stamps;
mod main;
mod game;
pub use art_stamps::{SVG, HrefAndClipMask};
pub use game::{SceneState, Images, TextureSurface};
use std::time;
use std::string::String;
use std::collections::HashMap;
use std::path::Path;
use std::io;
use std::io::{Read,Write};
use std::fs;
use sdl2::event::Event;
use sdl2::rect::{Rect,Point};
use sdl2::keyboard::Keycode;
use sdl2::mouse::Cursor;
use sdl2::surface::Surface;

static DESIRED_DURATION_PER_FRAME:time::Duration = time::Duration::from_millis(1);
static START_DURATION_PER_FRAME:time::Duration = time::Duration::from_millis(200);
static DELTA_DURATION_PER_FRAME:time::Duration = time::Duration::from_millis(75);

macro_rules! make_texture_surface {
    ($texture_creator: expr, $surf: expr, $name: expr) => (match $texture_creator.create_texture_from_surface(&$surf) {
        Ok(tex) => Ok(TextureSurface{
            texture:tex,
            surface:$surf,
            name:$name,
        }),
        Err(e) => Err(format!("{:?}", e)),
    });
}



fn process(state: &mut SceneState, _images: &mut Images, event: sdl2::event::Event, keys_down: &mut HashMap<Keycode, ()>) -> Result<bool,String>{
    let mut key_encountered = false;
    match event {
        Event::Quit{..} => {
            return Err("Exit".to_string())
        },
        Event::KeyDown {keycode: Option::Some(key_code), ..} =>{
            let repeat;
             if let None = keys_down.insert(key_code, ()) {
                repeat = false;
            } else {
                return Ok(false);
            }
            key_encountered = true;
            state.apply_keys(&keys_down, Some(key_code), repeat);
        },
        Event::KeyUp {keycode: Option::Some(key_code), ..} =>
        {
            keys_down.remove(&key_code);
        },
        Event::MouseButtonDown {x, y, ..} => {
            state.cursor_x = x;
            state.cursor_y = y;
            state.click();
        }
        Event::MouseMotion {x, y, ..} => {
            state.cursor_x = x;
            state.cursor_y = y;
        }
        Event::Window{win_event:sdl2::event::WindowEvent::Resized(width,height),..} => {
          state.window_width = width as u32;
          state.window_height = height as u32;
        }
        Event::Window{win_event:sdl2::event::WindowEvent::SizeChanged(width,height),..} => {
          state.window_width = width as u32;
          state.window_height = height as u32;
        }
        _ => {}
    }
    Ok(key_encountered)
}


fn read_to_string(filename: &Path) ->  Result<String, io::Error> {
    let mut f = fs::File::open(filename)?;
    let mut buffer = String::new();
    f.read_to_string(&mut buffer)?;
    Ok(buffer)
}

pub fn run(dir: &Path) -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem.window("rust-sdl2 demo: Cursor", 800, 600)
      .position_centered()
      .build()
      .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().software().build().map_err(|e| e.to_string())?;
    let mut keys_down = HashMap::<Keycode, ()>::new();
    let mouse_surface = Surface::load_bmp(dir.join("cursor.bmp"))
        .map_err(|err| format!("failed to load cursor image: {}", err))?;
    let svg = if let Ok(file_data) = read_to_string(&dir.join("level.svg")) {
        SVG::from_str(&file_data).unwrap()
    } else {
        SVG::new(1024,768)
    };
    let mut scene_state = SceneState::new(canvas.viewport().width(), canvas.viewport().height(), svg);
    let hero_path = dir.join("mouse.bmp");
    let hero_name = hero_path.to_str().unwrap().to_string();
    let hero_surface = Surface::load_bmp(hero_path)
        .map_err(|err| format!("failed to load cursor image: {}", err))?;
    let texture_creator = canvas.texture_creator();
    
    let mut images = Images{
        hero:make_texture_surface!(texture_creator, hero_surface, hero_name)?,
	stamps:Vec::new(),
	inventory_map:HashMap::new(),
    };
    
    process_dir(&dir.join("stamps"), &mut |p:&fs::DirEntry| {
        let stamp_surface = Surface::load_bmp(p.path()).map_err(
            |err| io::Error::new(io::ErrorKind::Other, format!("{}: {}", p.path().to_str().unwrap_or("??"), err)))?;
        images.stamps.push(make_texture_surface!(texture_creator, stamp_surface, p.path().to_str().unwrap().to_string()).map_err(
            |err| io::Error::new(io::ErrorKind::Other, format!("{}: {}", p.path().to_str().unwrap_or("?X?"), err)))?);
        Ok(())
    }).map_err(|err| format!("Failed to load stamp {}", err))?;
    for (index, stamp) in images.stamps.iter().enumerate() {
        images.inventory_map.insert(HrefAndClipMask{url:stamp.name.clone(), clip:String::new()}, index);
    }

    let cursor = Cursor::from_surface(mouse_surface, 0, 0).map_err(
            |err| format!("failed to load cursor: {}", err))?;
    cursor.set();
    main::run_main_loop_infinitely(&mut main::MainLoopArg{sdl_context:&sdl_context, scene_state:&mut scene_state, canvas:&mut canvas, images:&mut images, keys_down:&mut keys_down, texture_creator:&texture_creator, main_loop:main_loop})
}


fn process_dir<F: FnMut(&fs::DirEntry) -> Result<(), io::Error>>(dir: &Path, cb: &mut F) -> Result<(), io::Error> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            process_dir(&path, cb)?;
        } else {
            cb(&entry)?;
        }
    }
    Ok(())
}

fn main() -> Result<(), String> {
    if let Err(e) = run(Path::new("assets")) {
        if e != "Exit" {
            return Err(e);
        }
    }
    Ok(())
}

fn main_loop<'a>(sdl_context: &sdl2::Sdl, scene_state: &mut SceneState, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, images: &mut Images<'a>, keys_down: &mut HashMap<Keycode, ()>, _texture_creator:&'a sdl2::render::TextureCreator<sdl2::video::WindowContext>) -> Result<(), String> {
    let loop_start_time = time::Instant::now();
    let mut events = sdl_context.event_pump()?;
    let mut process_time = std::time::Duration::default();
    if keys_down.len() != 0 {
        for event in events.poll_iter() {
            process(scene_state, images, event, keys_down)?; // always break
        }
        scene_state.render(canvas, images)?; // mut images only needed for color mod
        process_time = loop_start_time.elapsed();
        if keys_down.len() != 0 {
            while process_time < scene_state.duration_per_frame {
                process_time = loop_start_time.elapsed();
                let mut any_events = false;
                for event in events.poll_iter() {
                    process(scene_state, images, event, keys_down)?; // always break
                    any_events = true;
                }
                if any_events {

                    scene_state.render(canvas, images)?;
                    scene_state.sim()?;
                }
            }
            if scene_state.duration_per_frame > DELTA_DURATION_PER_FRAME + DESIRED_DURATION_PER_FRAME {
                scene_state.duration_per_frame -= DELTA_DURATION_PER_FRAME;
            } else {
                scene_state.duration_per_frame = DESIRED_DURATION_PER_FRAME;
            }
            scene_state.apply_keys(&keys_down, None, true);
            scene_state.render(canvas, images)?;
        }
    } else {
        scene_state.duration_per_frame = START_DURATION_PER_FRAME;
        if main::IS_EMSCRIPTEN {
            for event in events.poll_iter() {
                process(scene_state, images, event, keys_down)?;
                break;
            }
        } else {
            for event in events.wait_timeout_iter((DESIRED_DURATION_PER_FRAME - process_time).as_millis() as u32) {
                process(scene_state, images, event, keys_down)?;
                break;
            }
        }
        scene_state.render(canvas, images)?;
	scene_state.sim()?;
    };
    Ok(())
}

#[derive(Clone, Debug)]
struct InventoryItem {
    stamp_index: usize,
    stamp_name: String,
}


