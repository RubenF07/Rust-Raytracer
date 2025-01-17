use crate::camera::Camera;
use crate::material::Material;
use crate::mesh::{stl_to_bvh, BVHMesh};
use crate::point::{dot, normalize, Point};
use crate::RGB;

pub struct Scene{
    objects: Vec<Box<dyn Hittable + Sync>>,
    pub camera: Camera,
    max_ray_depth: usize
}

impl Scene{
    pub fn new(cam: Camera,max_ray_depth: usize) -> Scene{
        Scene { objects: vec![], camera: cam, max_ray_depth: max_ray_depth }
    }

    pub fn add_object(&mut self,obj: Box<dyn Hittable + Sync>){
        self.objects.push(obj);
    }


    pub fn get_color(&self, pos: &Point, ray_dir: &Point, depth: usize) -> Option<RGB> {
        // TODO use single mesh hit check
        if depth > self.max_ray_depth{
            return Some(RGB::default());
        }

        let mut hits: Vec<(usize,f32)> = Vec::new();
        
        // Collect all hits with their indices
        for obj_i in 0..self.objects.len() {
            if let Some(distance) = self.objects[obj_i].hit_dist(pos, ray_dir) {
                hits.push((obj_i,distance));
            }
        }
        
    
        // Sort hits by distance
        hits.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    
        for hit in hits{
            if let Some(color) = self.objects[hit.0].get_color(pos, ray_dir,&self,depth){
                return Some(color)
            }
        }
        None
    }

    pub fn get_gradient(&self, height: f32) -> RGB {
        // return RGB{r:255 as u8,g:255 as u8,b:255};

        let gradient = 90.0 + height*70.0;
        RGB{r:gradient as u8,g:gradient as u8,b:255}        
    }
}




// Objects

pub trait Hittable{
    fn hit_dist(&self, pos: &Point, ray_dir: &Point) -> Option<f32>;
    fn get_color(&self, pos: &Point, ray_dir: &Point, scene: &Scene, depth: usize) -> Option<RGB>;
}

pub struct MeshObject{
    bvh: BVHMesh,
    material: Box<dyn Material + Sync>
}
impl MeshObject{
    pub fn new(stl_file: &str, max_bound_depth: usize, center: Point,scale: f32, mat: Box<dyn Material + Sync>) -> MeshObject{
        let bvh = stl_to_bvh(stl_file, max_bound_depth, center, scale);
        MeshObject { bvh: bvh, material: mat }
    }
}
impl Hittable for MeshObject{

    fn hit_dist(&self, pos: &Point, ray_dir: &Point) -> Option<f32> {
        self.bvh.get_bound_hit_dist(&ray_dir, &pos,0)
    }
    fn get_color(&self, pos: &Point, ray_dir: &Point, scene: &Scene, depth: usize) -> Option<RGB> {
        // // TODO TEST DEBUG
        if let Some(pos_normal) = self.bvh.get_final_tri_hit(pos,ray_dir){
            return Some(self.material.get_color(&pos_normal.0, &ray_dir,&pos_normal.1,scene,depth))
            // return Some(self.material.get_color(&pos_normal.0, &ray_dir,&Point { x: pos_normal.2 as f32, y: 0.0, z: 0.0 },scene,depth))
        }
        None
    }
}

pub struct Sphere{
    center: Point,
    radius: f32,
    material: Box<dyn Material + Sync>
}
impl Sphere{
    pub fn new(center: Point, radius: f32, mat: Box<dyn Material + Sync>) -> Sphere{
        Sphere{
            center: center,
            radius: radius,
            material: mat
        }
    }
}
impl Hittable for Sphere{
    fn hit_dist(&self, pos: &Point, ray_dir: &Point) -> Option<f32> {
        let oc = self.center.sub(&pos);
        let a = ray_dir.length_squared();
        let h = dot(ray_dir, &oc);
        let c = oc.length_squared() - self.radius*self.radius;
        let disciminant = h*h - a*c;

        if disciminant>0.0{
            let t = (h - f32::sqrt(disciminant))/a;
            if t > 0.0 {
                return Some(t);
            }
        }
        None
    }
    fn get_color(&self, pos: &Point, ray_dir: &Point, scene: &Scene, depth: usize) -> Option<RGB> {
        // let n = ray_dir.at_time(self.hit_dist(pos, ray_dir).unwrap()).add(&pos).sub(&self.center).at_time(1.0/self.radius);
        let n = normalize(&ray_dir.at_time(self.hit_dist(pos, ray_dir).unwrap()).add(&pos).sub(&self.center));
        Some(self.material.get_color(&ray_dir.at_time(self.hit_dist(pos, ray_dir).unwrap()).add(&pos), &ray_dir,&n,scene,depth))
    }
}