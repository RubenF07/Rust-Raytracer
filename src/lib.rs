pub mod mesh;
pub mod renderer;
pub mod display;
pub mod camera;
pub mod point;
pub mod scene;
pub mod material;

#[derive(Debug,Default,Clone)]
pub struct RGB{
    pub r:u8,
    pub g:u8,
    pub b:u8,
}
impl RGB {
    pub fn multi(&self,a: f32) -> RGB {
        RGB{
            r: (self.r as f32 * a) as u8,
            g: (self.g as f32 * a) as u8,
            b: (self.b as f32 * a) as u8,
        }
    }
    pub fn add(&self,a: RGB) -> RGB {
        RGB{
            r: self.r + a.r,
            g: self.g + a.g,
            b: self.b + a.b,
        }
    }
}