use sdl2::keyboard::Keycode;
use art_stamps::{Transform, SVG, HrefAndClipMask};
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
    pub stamps: Vec<TextureSurface<'r>>,
    pub inventory_map: HashMap<HrefAndClipMask, usize>,
    pub mouse: TextureSurface<'r>,
    pub cat: TextureSurface<'r>,
}
#[derive(Clone,Debug)]
pub struct LocationVel {
    pub location:Transform,
    pub vx:f32,
    pub vy:f32,
}
pub struct SceneState{
    pub cursor_x: i32,
    pub cursor_y: i32,
    mouse_location: Transform,
    cat_location: Transform,
    pub window_width: u32,
    pub window_height: u32,
    pub duration_per_frame: std::time::Duration,
    pub svg: SVG,
    camera_transform: Transform,
}

impl SceneState {
    pub fn new(width: u32, height:u32, svg: SVG) -> Self {
        SceneState{
            cursor_x:0,
            cursor_y:0,
	    mouse_location:Transform::new(32,32),
	    cat_location:Transform::new(256,64),
            duration_per_frame:std::time::Duration::from_millis(1),
            window_width: width,
            window_height: height,
	    svg:svg,
	    camera_transform:Transform::new(0,0),
        }
    }
    pub fn sim(&mut self) -> Result<(), String> {
	    Ok(())
    }
    pub fn draw_level<T:sdl2::render::RenderTarget>(&self, canvas: &mut sdl2::render::Canvas<T>, images: &mut Images) -> Result<(),String> {
	for g in self.svg.stamps.iter() {
            let texture_index = images.inventory_map.get(&g.rect.href).unwrap();
            let final_transform = art_stamps::compose(&self.camera_transform, &g.transform);
            let img = &mut images.stamps[*texture_index];
            img.texture.set_color_mod(g.rect.fill.r,g.rect.fill.g,g.rect.fill.b);
            canvas.copy_ex(
                &img.texture,
                None,
                Some(Rect::new(final_transform.tx as i32, final_transform.ty as i32, g.rect.width, g.rect.height)),
                final_transform.rotate,
                Point::new(final_transform.midx as i32, final_transform.midy as i32),
                false,
                false,
            ).map_err(|err| format!("{:?}", err))?;
	}
	Ok(())
    }
    pub fn render<T:sdl2::render::RenderTarget>(&self, canvas: &mut sdl2::render::Canvas<T>, images: &mut Images) -> Result<(),String> {
        let white = Color::RGBA(255, 255, 255, 255);
        canvas.set_draw_color(white);
        canvas.clear();
        canvas.copy_ex(
            &images.mouse.texture,
            None,
            Some(Rect::new(self.mouse_location.tx as i32, self.mouse_location.ty as i32,
                           images.mouse.surface.width(), images.mouse.surface.height())),
            0.0,
            Point::new(0,0),//centre
            false,// flip horiz
            false,// flip vert
        ).map_err(|err| format!("{:?}", err))?;
        canvas.copy_ex(
            &images.cat.texture,
            None,
            Some(Rect::new(self.cat_location.tx as i32, self.cat_location.ty as i32,
                           images.cat.surface.width(), images.cat.surface.height())),
            0.0,
            Point::new(0,0),//centre
            false,// flip horiz
            false,// flip vert
        ).map_err(|err| format!("{:?}", err))?;
        canvas.present();
        Ok(())
    }
    pub fn apply_keys(&mut self, keys_down: &HashMap<Keycode, ()>, new_key: Option<Keycode>, _repeat:bool) {
        let _is_shift_held = keys_down.contains_key(&Keycode::LShift) || keys_down.contains_key(&Keycode::RShift);
	
        if keys_down.contains_key(&Keycode::A) {
            self.cat_location.tx = self.cat_location.tx - 1.;
        }
        if keys_down.contains_key(&Keycode::D) {
            self.cat_location.tx = self.cat_location.tx +2.;
        }
        if keys_down.contains_key(&Keycode::W) {

        }
        if keys_down.contains_key(&Keycode::S) {

        }
        if keys_down.contains_key(&Keycode::Left) {
            self.mouse_location.tx -= 1.;
        }
        if keys_down.contains_key(&Keycode::Right) {
            self.mouse_location.tx += 1.;
        }
        if keys_down.contains_key(&Keycode::Up) {
	
        }
        if keys_down.contains_key(&Keycode::Down) {

        }
        if keys_down.contains_key(&Keycode::Escape) {
            std::process::exit(0);
        }
        if keys_down.contains_key(&Keycode::KpEnter) {
            self.click();
        }
        if let Some(Keycode::Return) = new_key {

        }
        if let Some(Keycode::Space) = new_key {

        }
    }
    pub fn click(&mut self) {
    }
}
