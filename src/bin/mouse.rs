extern crate sdl2;
extern crate mouse;
use std::time;
use std::string::String;
use std::collections::HashMap;
use std::env;
use std::vec::Vec;
use std::path::Path;
use std::fs;
use sdl2::event::Event;

use sdl2::keyboard::Keycode;
use sdl2::mouse::Cursor;
use sdl2::pixels::Color;
use std::io;
use std::io::{Read, Write};
use sdl2::rect::{Rect, Point};
use sdl2::surface::Surface;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Texture;

const MOUSE_CONSTANT: i32 = 1;

struct TextureSurface<'r> {
    texture: Texture<'r>,
    surface: Surface<'r>,
    name: String,
}

static DESIRED_DURATION_PER_FRAME:time::Duration = time::Duration::from_millis(1);
static START_DURATION_PER_FRAME:time::Duration = time::Duration::from_millis(200);
static RELAXED_DURATION_PER_FRAME:time::Duration = time::Duration::from_millis(1);
static DELTA_DURATION_PER_FRAME:time::Duration = time::Duration::from_millis(75);

fn mouse_move(delta:i32, repeat:time::Duration) -> i32 {
    if repeat <= DESIRED_DURATION_PER_FRAME {
        delta * 4
    } else {
        delta
    }
}


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



struct Images<'r> {
    default_cursor: TextureSurface<'r>,
}

#[derive(Clone,PartialEq)]
struct CursorTransform {
    mouse_x: i32,
    mouse_y: i32,
    transform: mouse::Transform,    
}

struct SceneState{
    cursor_transform: CursorTransform,
    duration_per_frame: time::Duration, // how long to wait while key is held down
    last_return_mouse: Option<CursorTransform>,
    cursor: Cursor,
    active_stamp: Option<usize>,
    stamp_used: bool,
    camera_transform: mouse::Transform,
    save_file_name: String,
    window_width: u32,
    window_height: u32,
    color: mouse::Color,
    locked: bool,
}

impl SceneState {
    fn render<T:sdl2::render::RenderTarget>(&self, canvas: &mut sdl2::render::Canvas<T>, images: &mut Images) -> Result<(),String> {
        canvas.set_draw_color(Color::RGBA(255, 255, 255, 255));
        canvas.clear();
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
        //canvas.fill_rect(Rect::new(self.mouse_x, self.mouse_y, 1, 1))?;
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
    fn mouse_lock_x(&self, mouse_coord:i32) -> i32 {
        self.mouse_lock(mouse_coord)
    }
    fn mouse_lock_y(&self, mouse_coord:i32) -> i32 {
        self.mouse_lock(mouse_coord)
    }
    fn mouse_lock(&self, mouse_coord:i32) -> i32 {
        if self.locked {
            return (mouse_coord/4)*4;
                
        }
        mouse_coord
    }
    fn apply_keys(&mut self, keys_down: &HashMap<Keycode, ()>, new_key: Option<Keycode>, repeat:bool) {
        if keys_down.len() != 0{
            //eprintln!("KEY PRESS {:?}; REPEAT {} {:?}?", keys_down, repeat, new_key);
        }
        let shifted_index = (keys_down.contains_key(&Keycode::LShift) as usize) | (keys_down.contains_key(&Keycode::RShift) as usize);
        if keys_down.contains_key(&Keycode::Left) {
            self.cursor_transform.mouse_x -= mouse_move(MOUSE_CONSTANT, self.duration_per_frame);
            self.clear_cursor_if_stamp_used();
        }
        if keys_down.contains_key(&Keycode::Right) {
            self.cursor_transform.mouse_x += mouse_move(MOUSE_CONSTANT, self.duration_per_frame);
            self.clear_cursor_if_stamp_used();
        }
        if keys_down.contains_key(&Keycode::Up) {
            self.cursor_transform.mouse_y -= mouse_move(MOUSE_CONSTANT, self.duration_per_frame);
            self.clear_cursor_if_stamp_used();
        }
        if keys_down.contains_key(&Keycode::Down) {
            self.cursor_transform.mouse_y += mouse_move(MOUSE_CONSTANT, self.duration_per_frame);
            self.clear_cursor_if_stamp_used();
        }
        if keys_down.contains_key(&Keycode::Escape) {
            std::process::exit(0);
        }
        if keys_down.contains_key(&Keycode::KpEnter) {
            if let Some(last_transform) = &self.last_return_mouse {
                if *last_transform != self.cursor_transform || !repeat {
                    self.click();
                }
            } else {
                self.click();
            }
            self.last_return_mouse = Some(self.cursor_transform.clone())
        } else {
            self.last_return_mouse = None; // other keypresses clear this
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
    fn clear_cursor_if_stamp_used(&mut self) {
    }
}

fn process(state: &mut SceneState, images: &mut Images, event: sdl2::event::Event, keys_down: &mut HashMap<Keycode, ()>) -> Result<bool,String>{
    let mut key_encountered = false;
    match event {
        Event::Quit{..} => {
            return Err("Exit".to_string())
        },
        Event::KeyDown {keycode: Option::Some(key_code), ..} =>{
            let repeat;
             if let None = keys_down.insert(key_code, ()) {
                repeat = false;
                for (key,_)in keys_down.iter() {
                    eprintln!("Key is down {}\n", *key)
                }
            } else {
                //eprintln!("EXTRA?");
                return Ok(false);
            }
            key_encountered = true;
            state.apply_keys(&keys_down, Some(key_code), repeat);
        },
        Event::KeyUp {keycode: Option::Some(key_code), ..} =>
        {
            state.last_return_mouse = None; // other keypresses clear this
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
            state.clear_cursor_if_stamp_used();
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

pub fn run(save_file_name: &str, dir: &Path) -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    //let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;
    let window = video_subsystem.window("rust-sdl2 demo: Cursor", 800, 600)
      .position_centered()
      .build()
      .map_err(|e| e.to_string())?;

    let wsize = window.size();
    let mut canvas = window.into_canvas().software().build().map_err(|e| e.to_string())?;
    let mut keys_down = HashMap::<Keycode, ()>::new();
    let surface = Surface::load_bmp(dir.join("cursor.bmp"))
        .map_err(|err| format!("failed to load cursor image: {}", err))?;
    let mut scene_state = SceneState {
        cursor_transform: CursorTransform {
            mouse_x:0,
            mouse_y:0,
            transform: mouse::Transform::new(0,0),
        },
        duration_per_frame:START_DURATION_PER_FRAME,
        last_return_mouse: None,
        active_stamp: None,
        stamp_used: false,
        camera_transform: mouse::Transform::new(0, 0),
        cursor:Cursor::from_surface(surface, 0, 0).map_err(
            |err| format!("failed to load cursor: {}", err))?,
        save_file_name: save_file_name.to_string(),
        window_width: canvas.viewport().width(),
        window_height: canvas.viewport().height(),
        color:mouse::Color{r:0,g:0,b:0},
        locked:false,
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
    run_main_loop_infinitely(&mut MainLoopArg{sdl_context:&sdl_context, scene_state:&mut scene_state, canvas:&mut canvas, images:&mut images, keys_down:&mut keys_down, texture_creator:&texture_creator})
}

struct MainLoopArg<'a, 'b>{
    sdl_context: &'a sdl2::Sdl,
    scene_state: &'a mut SceneState,
    canvas: &'a mut sdl2::render::Canvas<sdl2::video::Window>,
    images: &'a mut Images<'b>,
    keys_down: &'a mut HashMap<Keycode, ()>,
    texture_creator:&'b sdl2::render::TextureCreator<sdl2::video::WindowContext>
}

#[cfg(not(any(target_arch = "wasm32", target_arch = "asmjs")))]
fn run_main_loop_infinitely(arg:&mut MainLoopArg) -> Result<(), String> {
    loop {
        main_loop(arg.sdl_context, arg.scene_state, arg.canvas, arg.images, arg.keys_down, arg.texture_creator)?;
    }
}
extern "C" {
    fn emscripten_set_main_loop_arg(f: unsafe extern "C" fn(*mut std::ffi::c_void), arg: *mut std::ffi::c_void, fps: i32, sim_infinite_loop:i32);
    fn emscripten_cancel_main_loop();
}
#[cfg(any(target_arch = "wasm32", target_arch = "asmjs"))]
const is_emscripten:bool=true;
#[cfg(not(any(target_arch = "wasm32", target_arch = "asmjs")))]
const is_emscripten:bool=false;

    
#[cfg(any(target_arch="x86_64",target_arch = "wasm32", target_arch = "asmjs"))]
unsafe extern "C" fn packaged_main_loop(parg: *mut std::ffi::c_void) {
    let arg = &mut *(parg as *mut MainLoopArg);
    if let Err(_) = main_loop(arg.sdl_context, arg.scene_state, arg.canvas, arg.images, arg.keys_down, arg.texture_creator) {
        emscripten_cancel_main_loop();
    }
}

/*
fn emscripten_set_main_loop_arg(f: unsafe extern "C" fn(*mut std::ffi::c_void), arg: *mut std::ffi::c_void, fps: i32, sim_infinite_loop:i32) {
    loop {
        unsafe{f(arg)};
    }
}*/

#[cfg(any(target_arch = "wasm32", target_arch = "asmjs"))]
fn run_main_loop_infinitely<'a>(arg:&mut MainLoopArg) -> Result<(), String> {
    unsafe{emscripten_set_main_loop_arg(packaged_main_loop, arg as *mut _ as *mut std::ffi::c_void, -1, 0);}
    Ok(())
}

fn main_loop<'a>(sdl_context: &sdl2::Sdl, scene_state: &mut SceneState, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, images: &mut Images<'a>, keys_down: &mut HashMap<Keycode, ()>, texture_creator:&'a sdl2::render::TextureCreator<sdl2::video::WindowContext>) -> Result<(), String> {
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
        if is_emscripten {
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

fn main() -> Result<(), String> {
    let mut args: Vec<_> = env::args().collect();

    while args.len() < 2 {
        println!("Usage: cargo run /path/to/result");
        args.push("example.svg".to_string())

    }
    {
        let save_file_name = &Path::new(&args[1]);
        let ret = run(&args[1], Path::new("assets"));
        match ret {
            Err(x) => {
                if x == "Exit" {
                    Ok(())
                } else {
                    Err(x)
                }
            },
            ret => ret,
        }
    }
}
