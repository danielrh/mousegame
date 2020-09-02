extern crate sdl2;
extern crate mousegame;
mod main;
use std::time;
use std::string::String;
use std::collections::HashMap;
use std::path::Path;
use sdl2::event::Event;

use sdl2::keyboard::Keycode;
use sdl2::mouse::Cursor;
use sdl2::pixels::Color;
use sdl2::rect::{Rect, Point};
use sdl2::surface::Surface;
use sdl2::render::Texture;

const MOUSE_CONSTANT: i32 = 1;

struct TextureSurface<'r> {
    texture: Texture<'r>,
    surface: Surface<'r>,
    name: String,
}

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



pub struct Images<'r> {
    default_cursor: TextureSurface<'r>,
}

#[derive(Clone,PartialEq)]
struct CursorTransform {
    mouse_x: i32,
    mouse_y: i32,
}

pub struct SceneState{
    cursor_transform: CursorTransform,
    duration_per_frame: time::Duration, // how long to wait while key is held down
    cursor: Cursor,
    window_width: u32,
    window_height: u32,
    color: Color,
}

impl SceneState {
    fn render<T:sdl2::render::RenderTarget>(&self, canvas: &mut sdl2::render::Canvas<T>, images: &mut Images) -> Result<(),String> {
        canvas.set_draw_color(Color::RGBA(255, 255, 255, 255));
        canvas.clear();
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
        images.default_cursor.texture.set_color_mod(self.color.r,self.color.g,self.color.b);
        canvas.copy_ex(
            &images.default_cursor.texture,
            None,
            Some(Rect::new(self.cursor_transform.mouse_x, self.cursor_transform.mouse_y,
                           images.default_cursor.surface.width(), images.default_cursor.surface.height())),
            0.0,
            Point::new(0,0),//centre
             false,//horiz
            false,//vert
        ).map_err(|err| format!("{:?}", err))?;
        canvas.present();
        Ok(())
    }
    fn apply_keys(&mut self, keys_down: &HashMap<Keycode, ()>, new_key: Option<Keycode>, repeat:bool) {
        let _is_shift_held = keys_down.contains_key(&Keycode::LShift) || keys_down.contains_key(&Keycode::RShift);
	
        if keys_down.contains_key(&Keycode::Left) {
            self.cursor_transform.mouse_x -= MOUSE_CONSTANT;
        }
        if keys_down.contains_key(&Keycode::Right) {
            self.cursor_transform.mouse_x += MOUSE_CONSTANT;
        }
        if keys_down.contains_key(&Keycode::Up) {
            self.cursor_transform.mouse_y -= MOUSE_CONSTANT;
        }
        if keys_down.contains_key(&Keycode::Down) {
            self.cursor_transform.mouse_y += MOUSE_CONSTANT
        }
        if keys_down.contains_key(&Keycode::Escape) {
            std::process::exit(0);
        }
        if keys_down.contains_key(&Keycode::KpEnter) {
            self.click();
        }
        if let Some(Keycode::Return) = new_key {
            if !repeat {
                self.click();
            }
        }
        if let Some(Keycode::Space) = new_key {
            if !repeat {
                self.click();
            }
        }
    }
    fn click(&mut self) {
    }
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
            state.cursor_transform.mouse_x = x;
            state.cursor_transform.mouse_y = y;
            state.click();
        }
        Event::MouseMotion {x, y, ..} => {
            state.cursor_transform.mouse_x = x;
            state.cursor_transform.mouse_y = y;
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

pub fn run(dir: &Path) -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem.window("rust-sdl2 demo: Cursor", 800, 600)
      .position_centered()
      .build()
      .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().software().build().map_err(|e| e.to_string())?;
    let mut keys_down = HashMap::<Keycode, ()>::new();
    let surface = Surface::load_bmp(dir.join("cursor.bmp"))
        .map_err(|err| format!("failed to load cursor image: {}", err))?;
    let mut scene_state = SceneState {
        cursor_transform: CursorTransform{
            mouse_x:0,
            mouse_y:0,
        },
        duration_per_frame:START_DURATION_PER_FRAME,
        cursor:Cursor::from_surface(surface, 0, 0).map_err(
            |err| format!("failed to load cursor: {}", err))?,
        window_width: canvas.viewport().width(),
        window_height: canvas.viewport().height(),
        color:Color::RGBA(0,0,0,0),
    };
    let cursor_surface_path = dir.join("cursor.bmp");
    let cursor_surface_name = cursor_surface_path.to_str().unwrap().to_string();
    let cursor_surface = Surface::load_bmp(cursor_surface_path)
        .map_err(|err| format!("failed to load cursor image: {}", err))?;
    let texture_creator = canvas.texture_creator();
    
    let mut images = Images{
        default_cursor:make_texture_surface!(texture_creator, cursor_surface, cursor_surface_name)?,
    };
    scene_state.cursor.set();
    main::run_main_loop_infinitely(&mut main::MainLoopArg{sdl_context:&sdl_context, scene_state:&mut scene_state, canvas:&mut canvas, images:&mut images, keys_down:&mut keys_down, texture_creator:&texture_creator, main_loop:main_loop})
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
    if keys_down.len() != 0 {
        for event in events.poll_iter() {
            process(scene_state, images, event, keys_down)?; // always break
        }
        scene_state.render(canvas, images)?; // mut images only needed for color mod
        let mut process_time = loop_start_time.elapsed();
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
            for event in events.wait_iter() {
                process(scene_state, images, event, keys_down)?;
                break;
            }
        }
        scene_state.render(canvas, images)?;
    };
    Ok(())
}
