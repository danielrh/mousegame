use super::sdl2;
use sdl2::keyboard::Keycode;

use std::collections::HashMap;

pub struct MainLoopArg<'a, 'b>{
    pub sdl_context: &'a sdl2::Sdl,
    pub scene_state: &'a mut super::SceneState,
    pub canvas: &'a mut sdl2::render::Canvas<sdl2::video::Window>,
    pub images: &'a mut super::Images<'b>,
    pub keys_down: &'a mut HashMap<Keycode, ()>,
    pub texture_creator:&'b sdl2::render::TextureCreator<sdl2::video::WindowContext>,
    pub main_loop: fn(sdl_context: &sdl2::Sdl, scene_state: &mut super::SceneState, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, images: &mut super::Images<'b>, keys_down: &mut HashMap<Keycode, ()>, texture_creator:&'b sdl2::render::TextureCreator<sdl2::video::WindowContext>) -> Result<(), String>
}

#[cfg(not(any(target_arch = "wasm32", target_arch = "asmjs")))]
pub fn run_main_loop_infinitely(arg:&mut MainLoopArg) -> Result<(), String> {
    loop {
        (arg.main_loop)(arg.sdl_context, arg.scene_state, arg.canvas, arg.images, arg.keys_down, arg.texture_creator)?;
    }
}
#[cfg(any(target_arch = "wasm32", target_arch = "asmjs"))]
extern "C" {
    fn emscripten_set_main_loop_arg(f: unsafe extern "C" fn(*mut std::ffi::c_void), arg: *mut std::ffi::c_void, fps: i32, sim_infinite_loop:i32);
    fn emscripten_cancel_main_loop();
}
#[cfg(any(target_arch = "wasm32", target_arch = "asmjs"))]
pub const IS_EMSCRIPTEN:bool=true;
#[cfg(not(any(target_arch = "wasm32", target_arch = "asmjs")))]
pub const IS_EMSCRIPTEN:bool=false;

    
#[cfg(any(target_arch = "wasm32", target_arch = "asmjs"))]
unsafe extern "C" fn packaged_main_loop(parg: *mut std::ffi::c_void) {
    let arg = &mut *(parg as *mut MainLoopArg);
    if let Err(_) = (arg.main_loop)(arg.sdl_context, arg.scene_state, arg.canvas, arg.images, arg.keys_down, arg.texture_creator) {
        emscripten_cancel_main_loop();
    }
}

#[cfg(any(target_arch = "wasm32", target_arch = "asmjs"))]
fn run_main_loop_infinitely<'a>(arg:&mut MainLoopArg) -> Result<(), String> {
    unsafe{emscripten_set_main_loop_arg(packaged_main_loop, arg as *mut _ as *mut std::ffi::c_void, -1, 0);}
    Ok(())
}

