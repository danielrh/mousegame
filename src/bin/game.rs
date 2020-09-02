use sdl2::keyboard::Keycode;
use std::collections::HashMap;
use sdl2::surface::Surface;
use sdl2::render::Texture;
use sdl2::pixels::Color;
use sdl2::rect::{Rect, Point};

pub struct TextureSurface<'r> {
    pub texture: Texture<'r>,
    pub surface: Surface<'r>,
    pub name: String,
}
pub struct Images<'r> {
    pub hero: TextureSurface<'r>,
}

pub struct SceneState{
    pub cursor_x: i32,
    pub cursor_y: i32,
    pub window_width: u32,
    pub window_height: u32,
    pub duration_per_frame: std::time::Duration,
}

impl SceneState {
    pub fn new(width: u32, height:u32) -> Self {
        SceneState{
            cursor_x:0,
            cursor_y:0,
            duration_per_frame:std::time::Duration::from_millis(1),
            window_width: width,
            window_height: height,
        }
    }
    pub fn render<T:sdl2::render::RenderTarget>(&self, canvas: &mut sdl2::render::Canvas<T>, images: &mut Images) -> Result<(),String> {
        canvas.set_draw_color(Color::RGBA(255, 255, 255, 255));
        canvas.clear();
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
        canvas.copy_ex(
            &images.hero.texture,
            None,
            Some(Rect::new(self.cursor_x, self.cursor_y,
                           images.hero.surface.width(), images.hero.surface.height())),
            0.0,
            Point::new(0,0),//centre
             false,//horiz
            false,//vert
        ).map_err(|err| format!("{:?}", err))?;
        canvas.present();
        Ok(())
    }
    pub fn apply_keys(&mut self, keys_down: &HashMap<Keycode, ()>, new_key: Option<Keycode>, repeat:bool) {
        let _is_shift_held = keys_down.contains_key(&Keycode::LShift) || keys_down.contains_key(&Keycode::RShift);
	
        if keys_down.contains_key(&Keycode::Left) {
            self.cursor_x -= 1;
        }
        if keys_down.contains_key(&Keycode::Right) {
            self.cursor_x += 1;
        }
        if keys_down.contains_key(&Keycode::Up) {
            self.cursor_y -= 1;
        }
        if keys_down.contains_key(&Keycode::Down) {
            self.cursor_y += 1;
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
    pub fn click(&mut self) {
    }
}
