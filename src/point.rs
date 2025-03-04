use rand::{thread_rng, Rng};

#[derive(Debug,Default,Clone)]
pub struct Point{
    pub x:f32,
    pub y:f32,
    pub z:f32
}
impl Point {
    pub fn add(&self,p:&Point) -> Point{
        Point { x: self.x+p.x, y: self.y+p.y, z: self.z+p.z }
    }
    pub fn sub(&self,p:&Point) -> Point{
        Point { x: self.x-p.x, y: self.y-p.y, z: self.z-p.z }
    }
    pub fn at_time(&self, t:f32) -> Point{
        Point { x: self.x*t, y: self.y*t, z: self.z*t }
    }
    pub fn len(&self) -> f32{
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
    pub fn length_squared(&self) -> f32{
        self.x*self.x + self.y*self.y + self.z*self.z
    }
}

pub fn rand_point() -> Point{
    let mut rng = thread_rng();
    normalize(&Point{x:rng.gen_range(-1.0..1.0),y:rng.gen_range(-1.0..1.0),z:rng.gen_range(-1.0..1.0)})
}

pub fn normalize(p: &Point) -> Point {
    let len = p.len();
    Point {
        x: p.x / len,
        y: p.y / len,
        z: p.z / len
    }
}
pub fn cross(a: &Point, b: &Point) -> Point {
    Point {
        x: a.y * b.z - a.z * b.y,
        y: a.z * b.x - a.x * b.z,
        z: a.x * b.y - a.y * b.x
    }
}
pub fn dot(a: &Point, b: &Point) -> f32 {
    a.x*b.x + a.y*b.y + a.z*b.z
}

pub fn get_tri_intersect(pos:&Point,dir:&Point,a:&Point,b:&Point,c:&Point,normal:&Point) -> Option<f32>{        
    // Möller-Trumbore algorithm
    let edge_ab = b.sub(&a);
    let edge_ac = c.sub(&a);
    
    let denom = -dot(&dir,&normal);

    if denom>=0.0001{
        let recip = 1.0/denom;
        
        let ao = pos.sub(&a);
        let dao = cross(&ao, &dir);
        
        let u = dot(&edge_ac,&dao)*recip;
        let v = -dot(&edge_ab,&dao)*recip;
        let t = dot(&ao,&normal)*recip;
        
        if t>=0.0 && u>=0.0 && v>=0.0 && (u+v) <= 1.0{
            return Some(t);
        }
    }
    None
}