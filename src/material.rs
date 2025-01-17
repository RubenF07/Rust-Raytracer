use crate::{point::{dot, rand_point, Point,normalize}, scene::Scene, RGB};

pub trait Material{
    fn get_color(&self,pos: &Point, ray_dir: &Point, normal: &Point, scene: &Scene, depth: usize) -> RGB;
}

pub struct NormalBased{}

impl Material for NormalBased{
    fn get_color(&self,pos: &Point, ray_dir: &Point, normal: &Point, scene: &Scene, depth: usize) -> RGB {
        RGB{
            r:(255.0*normal.x) as u8,
            g:(255.0*normal.y) as u8,
            b:(255.0*normal.z) as u8
        }
    }
}

// Better Diffuse Meathod
pub struct Lambertian{
    color: RGB
}
impl Lambertian {
    pub fn new(color: RGB) -> Lambertian{
        Lambertian { color: color }
    }
}

impl Material for Lambertian{
    fn get_color(&self,pos: &Point, ray_dir: &Point, normal: &Point, scene: &Scene, depth: usize) -> RGB {
        let mut dir: Point = normalize(&normal.at_time(0.001).add(&rand_point()));
        if dot(&dir, &normal) < 0.0{
            dir = dir.at_time(-1.0);
        }

        
        if let Some(scene_hit) = scene.get_color(&pos.add(&normal.at_time(0.01)), &dir, depth+1){
            return scene_hit.multi(0.5);
        }
        self.color.clone()
    }
}


pub struct Metal{
    luster: f32,
    color: RGB
}
impl Metal {
    pub fn new(luster: f32,color: RGB) -> Metal{
        assert!(luster >= 0.0 && luster <= 1.0);
        Metal { luster:luster ,color: color }
    }
}

impl Material for Metal{
    fn get_color(&self,pos: &Point, ray_dir: &Point, normal: &Point, scene: &Scene, depth: usize) -> RGB {
        // Reflections algorithm
        let dir: Point = ray_dir.sub(&normal.at_time(2.0 * dot(ray_dir, normal)));

        if let Some(scene_hit) = scene.get_color(&pos.add(&normal.at_time(0.01)), &dir, depth+1){
            // println!("({},{},{})",
            // pos.x,
            // pos.y,
            // pos.z
            // );
            // println!("({},{},{})",
            // normal.x,
            // normal.y,
            // normal.z
            // );
            // println!("({},{},{})",pos.add(&normal.at_time(20.0)).x,pos.add(&normal.at_time(20.0)).y,pos.add(&normal.at_time(20.0)).z);
            return scene_hit.multi(self.luster).add(self.color.multi(1.0 - self.luster));
        }
        
        scene.get_gradient((dir.y + 1.0) / 2.0).multi(self.luster).add(self.color.multi(1.0 - self.luster))
    }
}

pub struct BVHDebug{
    bad_threshold: u32
}
impl BVHDebug{
    pub fn new(threshold: u32) -> BVHDebug{
        BVHDebug{bad_threshold:threshold}
    }
}
impl Material for BVHDebug {
    fn get_color(&self, pos: &Point, ray_dir: &Point, normal: &Point, scene: &Scene, depth: usize) -> RGB {
        let value = normal.x as u32;

        if value > self.bad_threshold {
            // println!("Bad value: {}", value);
            return RGB { r: 255, g: 0, b: 0 };
        }

        let color = 255 - (value*255/self.bad_threshold) as u8;
        RGB { r: color, g: color, b: color }
        // if value > self.bad_threshold {
        //     RGB { r: 255, g: 0, b: 0 }
        // } else {
        //     let lower_bound = self.bad_threshold.saturating_sub(30);
        //     if value < lower_bound {
        //         RGB { r: 255, g: 255, b: 255 }
        //     } else {
        //         let percentage = (value - lower_bound) as f32 / 30.0;
        //         let white_component = 255 - (255.0 * percentage) as u8;
        //         RGB { r: 255, g: white_component, b: white_component }
        //     }
        // }
    }
}
