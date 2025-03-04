use crate::point::{Point,cross,normalize};
use rand::Rng;

pub struct Camera{
    pub pos:Point,
    pub width:u32,
    pub height:u32,
    aspect_ratio:f32,

    // degrees of x range converted to tan
    fov:f32,

    up:Point,
    forward:Point,
    right:Point,
}

impl Camera {
    pub fn new(position:Point,direction:Point,width:u32,height:u32,fov:f32) -> Camera{
        let forward = normalize(&direction);
        let right = normalize(&cross(&forward, &Point { x: 0.0, y: 1.0, z: 0.0 }));
        let up = normalize(&cross(&right, &forward));

        let aspect_ratio = width as f32/height as f32;
        let tan_fov = (fov.to_radians() / 2.0).tan();

        Camera { pos: position, width: width, height: height, aspect_ratio: aspect_ratio, fov: tan_fov, up: up, forward: forward, right: right}
    }

    pub fn get_ray(&self, pixel:&[u32;2], anti_aliasing_strength:&f32) -> Point{
        // Calculate ray direction
        let offset_pixel = get_offset(*pixel, *anti_aliasing_strength);
        let x_ndc = (2.0 * offset_pixel[0] / (self.width - 1) as f32 - 1.0) * self.aspect_ratio;
        let y_ndc = 1.0 - 2.0 * offset_pixel[1] / (self.height - 1) as f32;
        Point {
            x: self.forward.x + (self.right.x * x_ndc + self.up.x * y_ndc) * self.fov,
            y: self.forward.y + (self.right.y * x_ndc + self.up.y * y_ndc) * self.fov,
            z: self.forward.z + (self.right.z * x_ndc + self.up.z * y_ndc) * self.fov
        }
    }    
}

fn get_offset(pixel: [u32;2],range:f32) -> [f32;2]{
    if range == 0.0{
        return [pixel[0] as f32, pixel[1] as f32];
    }

    let mut rng = rand::thread_rng();
    let x: f32 = rng.gen_range(-range..range);
    let y: f32 = rng.gen_range(-range..range);
    [pixel[0] as f32 + x,pixel[1] as f32 + y]
}