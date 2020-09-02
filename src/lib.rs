#[macro_use]
extern crate serde_derive;
extern crate serde_xml_rs;
extern crate serde;
extern crate regex;
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Transform {
    pub midx: f64,
    pub midy: f64,
    pub rotate: f64,
    pub tx: f64,
    pub ty: f64,
    pub scale: f64,
}
impl Transform {
  pub fn new(width: u32, height: u32) -> Transform {
      Transform{
          scale:1.0,
          midx:width as f64/2.0,
          midy:height as f64/2.0,
          rotate:0.0,
          tx:0.0,
          ty:0.0,
      }
  }
  pub fn to_bbox(&self) -> [(f64,f64);4] {
      [ftransform(self, (0.,0.)),
       ftransform(self, (0., self.midy * 2.)),
       ftransform(self, (self.midx * 2., self.midy * 2.)),
       ftransform(self, (self.midx * 2., 0.)),
       ]
  }
  fn to_string(&self) -> Result<String, serde_xml_rs::Error> {
    let mut components = [String::new(),String::new(),String::new(),String::new(),String::new()];
    let mut num_components = 0usize;
    if self.scale != 1.0 {
      components[num_components] = format!("scale({})", self.scale);
      num_components += 1;
    }
    if self.tx != 0.0 || self.ty != 0.0 {
      components[num_components] = format!("translate({}, {})", self.tx, self.ty);
      num_components += 1;      
    }
    if self.midx != 0.0 || self.midy != 0.0 {
      components[num_components] = format!("translate({}, {})", self.midy, self.midy);
      num_components += 1;      
    }
    if self.rotate != 0.0 {
      components[num_components] = format!("rotate({})", self.rotate);
      num_components += 1;
    }
    if self.midx != 0.0 || self.midy != 0.0 {
      components[num_components] = format!("translate({}, {})", -self.midx, -self.midy);
      num_components += 1;      
    }
    return Ok(components[..num_components].join(" "))
  }
}
pub type F64Point = (f64, f64);



pub fn ftransform(t:&Transform, p: F64Point) -> F64Point {
    let centered = (p.0 - t.midx, p.1 - t.midy);
    let rotate_rad = -t.rotate * std::f64::consts::PI/180.;
    let rotated = (centered.0 * rotate_rad.cos() + centered.1 * rotate_rad.sin(),
                   -centered.0 * rotate_rad.sin() + centered.1 * rotate_rad.cos());
    let scaled = (rotated.0 * t.scale, rotated.1 * t.scale);
    let recentered = (scaled.0 + t.midx, scaled.1 + t.midy);
    (recentered.0 + t.tx, recentered.1 + t.ty)
}

pub fn itransform(t:&Transform, p: F64Point) -> F64Point {
    let untranslated = (p.0 - t.tx, p.1 - t.ty);
    let recentered = (untranslated.0 - t.midx, untranslated.1 - t.midy);
    let unscaled = (recentered.0/t.scale, recentered.1/t.scale);
    let rotate_rad = t.rotate * std::f64::consts::PI/180.;
    let rotated = (unscaled.0 * rotate_rad.cos() + unscaled.1 * rotate_rad.sin(),
                   -unscaled.0 * rotate_rad.sin() + unscaled.1 * rotate_rad.cos());
    let centered = (rotated.0 + t.midx, rotated.1 + t.midy);
    centered
}

#[derive(Debug, Default,Copy,Clone, Eq,PartialEq)]
pub struct Color{
    pub r:u8,
    pub g:u8,
    pub b:u8,
}
