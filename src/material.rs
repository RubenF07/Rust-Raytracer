use std::f32::EPSILON;
use crate::{point::{dot, rand_point, Point,normalize}, scene::Scene, RGB};

pub struct MaterialData<'a>{
    pos: &'a Point,
    ray_dir: &'a Point,
    normal: &'a Point,
    scene: &'a Scene,
    depth: usize
}
impl MaterialData<'_> {
    pub fn new<'a>(pos: &'a Point, ray_dir: &'a Point, normal: &'a Point, scene: &'a Scene, depth: usize) -> MaterialData<'a>{
        MaterialData{
            pos: pos,
            ray_dir: ray_dir,
            normal: normal,
            scene: scene,
            depth: depth
        }
    }
    
}

pub trait Material{
    fn get_color(&self,data:MaterialData) -> RGB;
}

pub struct NormalBased{}

impl Material for NormalBased{
    fn get_color(&self,data:MaterialData) -> RGB {
        RGB{
            r:(255.0*data.normal.x) as u8,
            g:(255.0*data.normal.y) as u8,
            b:(255.0*data.normal.z) as u8
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
    fn get_color(&self,data:MaterialData) -> RGB {
        let (normal, pos, scene, depth) = (data.normal, data.pos, data.scene, data.depth);
        let dir: Point = normalize(&normal.at_time(1.0+EPSILON).add(&rand_point()));
        
        if let Some(scene_hit) = scene.get_color(&pos.add(&normal.at_time(EPSILON)), &dir, depth+1){
            return scene_hit.multi(0.5);
        }
        self.color.clone()
    }
}


pub struct Metal{
    luster: f32,
    blur: f32,
    color: RGB
}
impl Metal {
    pub fn new(luster: f32, blur: f32,color: RGB) -> Metal{
        assert!(luster == luster.clamp(0.0, 1.0) && blur == blur.clamp(0.0, 1.0));
        Metal { luster:luster, blur:blur, color: color }
    }
}

impl Material for Metal{
    fn get_color(&self,data:MaterialData) -> RGB {
        let (ray_dir, pos, scene,normal, depth) = (data.ray_dir, data.pos, data.scene, data.normal, data.depth);

        let dir: Point = if self.blur > 0.0 {
            normalize(&reflect(&ray_dir, &normal).add(&rand_point().at_time(self.blur*0.1)))
        }
        else{
            reflect(&ray_dir, &normal)
        };
        

        if let Some(scene_hit) = scene.get_color(&pos, &dir, depth+1){
            return scene_hit.multi(self.luster).add(self.color.multi(1.0 - self.luster));
        }
        
        scene.get_gradient((dir.y + 1.0) / 2.0).multi(self.luster).add(self.color.multi(1.0 - self.luster))
    }
}
fn reflect(ray: &Point, normal: &Point) -> Point{
    ray.sub(&normal.at_time(2.0 * dot(ray, normal)))
}