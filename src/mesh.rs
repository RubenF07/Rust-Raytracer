use rand::Rng;
use stl_io::{read_stl, IndexedMesh, Vector};
use std::f32::EPSILON;
use std::{collections::VecDeque, f32::INFINITY, fs::OpenOptions, vec};
use crate::point::{cross, get_tri_intersect, Point};
use crate::RGB;
use crate::camera::Camera;

#[derive(Debug)]
pub struct Tri{
    // indecies of mesh vertecies
    pub vertecies: [usize; 3],
    pub normal: Point,
    pub center: Point,
    pub area: f32
}

#[derive(Debug)]
pub struct Mesh{
    pub vertecies: Vec<Point>,
    pub tris: Vec<Tri>
}


pub struct BVHHitReturn{
    tri_idx: Option<usize>,
    bound_checks: u32,
    bounds_checked: Vec<usize>
}

#[derive(Debug,Clone)]
pub struct Bound{
    // index in bound list
    parent_index: usize,

    pub tris: Vec<usize>,
    // min-max bounds
    pub x:[f32;2],
    pub y:[f32;2],
    pub z:[f32;2],
    
    // index of BVHMesh bounds
    pub children:Vec<usize>,

    pub depth: usize
}
#[derive(Debug,Default)]
pub struct BVHMesh{
    pub bounds: Vec<Bound>,
    pub vertecies: Vec<Point>,
    pub tris: Vec<Tri>,
}
impl BVHMesh{

    // Called when requesting tri on a confirmed bvh hit
    // returns (tri center, tri normal)

    // // TODO DEBUG
    // pub fn get_final_tri_hit(&self, pos:&Point, dir: &Point) -> Option<(&Point,&Point)>{
    pub fn get_final_tri_hit(&self, pos:&Point, dir: &Point) -> Option<(&Point,&Point,u32)>{
        let tri: BVHHitReturn = self.get_tri_in_bvh(&dir, &pos, 0);

        if tri.tri_idx.is_some(){
            return Some((&self.tris[tri.tri_idx.unwrap()].center, &self.tris[tri.tri_idx.unwrap()].normal, tri.bound_checks));
        }
        // return Some((&self.tris[0].center, &Point{x:0.0,y:0.0,z:0.0},tri.bound_checks));
    
        None
    }

    fn get_leaf_bound_tri_hit(&self, ray:&Point, origin: &Point, bound: &Bound) -> Option<usize>{
        let mut best_dist = f32::INFINITY;
        let mut best_tri_idx = 0;

        for tri_idx in &bound.tris{
            let tri: &Tri = &self.tris[*tri_idx];

            let hit = get_tri_intersect(&origin ,&ray, &self.vertecies[tri.vertecies[0]], &self.vertecies[tri.vertecies[1]], &self.vertecies[tri.vertecies[2]]);
            if hit.is_some_and(|x| x < best_dist && x > 0.0){
                best_dist = hit.unwrap();
                best_tri_idx = *tri_idx;
            }
        }

        return if best_dist != f32::INFINITY{
            Some(best_tri_idx)
        } else {
            None
        };
    }

    // Private function to get the hit tris in bvh
    fn get_tri_in_bvh(&self,ray: &Point, origin: &Point, bound_idx: usize) -> BVHHitReturn{
        let bound: &Bound = &self.bounds[bound_idx];
        if bound.children.len() == 0{
            // Leaf Node
            return BVHHitReturn{tri_idx: self.get_leaf_bound_tri_hit(&ray, &origin, bound), bound_checks:0, bounds_checked:vec![bound_idx]};
        }

        // TODO bounds should only have 0 or 2 children
        if bound.children.len() == 1{
            println!("ERROR: Bound with 1 child");
            return self.get_tri_in_bvh(&ray, &origin, bound.children[0]);
        }
        
        let mut child_checks: u32 = 2;
        let mut child_bounds_checked: Vec<usize> = vec![bound_idx];

        let first_hit = self.get_bound_hit_dist(&ray, &origin, bound.children[0]);
        let second_hit = self.get_bound_hit_dist(&ray, &origin, bound.children[1]);

        // Both hit
        if first_hit.is_some() && second_hit.is_some(){
            let closer_bound_idx = if first_hit.unwrap() < second_hit.unwrap(){bound.children[0]} else {bound.children[1]};
            let farther_bound_idx = if first_hit.unwrap() < second_hit.unwrap(){bound.children[1]} else {bound.children[0]};

            let first_hit = self.get_tri_in_bvh(&ray, &origin, closer_bound_idx);
            child_bounds_checked.extend(first_hit.bounds_checked);
            child_checks += first_hit.bound_checks;

            if first_hit.tri_idx.is_some(){
                return BVHHitReturn{tri_idx: first_hit.tri_idx, bound_checks:child_checks, bounds_checked: child_bounds_checked};
            }

            let second_hit = self.get_tri_in_bvh(&ray, &origin, farther_bound_idx);
            child_bounds_checked.extend(second_hit.bounds_checked);
            child_checks += second_hit.bound_checks;
            
            if second_hit.tri_idx.is_some(){
                return BVHHitReturn{tri_idx: second_hit.tri_idx, bound_checks:child_checks, bounds_checked: child_bounds_checked};
            }
            return BVHHitReturn{tri_idx: None, bound_checks:child_checks, bounds_checked: child_bounds_checked};
        }
        
        // Only one hit
        if first_hit.is_some(){
            let first_hit = self.get_tri_in_bvh(&ray, &origin, bound.children[0]);
                child_bounds_checked.extend(first_hit.bounds_checked);
                child_checks += first_hit.bound_checks;
                
                if first_hit.tri_idx.is_some(){
                    return BVHHitReturn{tri_idx: first_hit.tri_idx, bound_checks:child_checks, bounds_checked: child_bounds_checked};
                }
            }
        if second_hit.is_some(){
            let second_hit = self.get_tri_in_bvh(&ray, &origin, bound.children[1]);
            child_bounds_checked.extend(second_hit.bounds_checked);
            child_checks += second_hit.bound_checks;

            if second_hit.tri_idx.is_some(){
                return BVHHitReturn{tri_idx: second_hit.tri_idx, bound_checks:child_checks, bounds_checked: child_bounds_checked};
            }
        }

        // None hit or all returned empty
        return BVHHitReturn{tri_idx: None, bound_checks:child_checks, bounds_checked: child_bounds_checked};
    
    }


    pub fn get_bound_hit_dist(&self, ray: &Point, origin: &Point, bound_idx: usize) -> Option<f32> {
        let bound = &self.bounds[bound_idx];
        let mut t_min = f32::NEG_INFINITY;
        let mut t_max = f32::INFINITY;

        for i in 0..3 {
            let (ray_dir, bound_min, bound_max, origin_coord) = match i {
                0 => (ray.x, bound.x[0], bound.x[1], origin.x),
                1 => (ray.y, bound.y[0], bound.y[1], origin.y),
                _ => (ray.z, bound.z[0], bound.z[1], origin.z),
            };

            if ray_dir.abs() < 1e-8 {
                if origin_coord < bound_min || origin_coord > bound_max {
                    return None;
                }
                continue;
            }

            let inv_d = 1.0 / ray_dir;
            let t0 = (bound_min - origin_coord) * inv_d;
            let t1 = (bound_max - origin_coord) * inv_d;

            let (t_near, t_far) = if inv_d < 0.0 { (t1, t0) } else { (t0, t1) };

            t_min = t_min.max(t_near);
            t_max = t_max.min(t_far);

            if t_min > t_max {
                return None;
            }
        }

        if t_min >= 0.0 {
            Some(t_min)
        } else if t_max >= 0.0 {
            Some(t_max)
        } else {
            None
        }
    }
}

fn mesh_center(vertecies: &Vec<Vector<f32>>) -> Point{
    let mut net_vertex = Point::default();
    for vertex in vertecies{
        net_vertex.x += vertex[0];
        net_vertex.y += vertex[1];
        net_vertex.z += vertex[2];
    }

    net_vertex.x /= vertecies.len() as f32;
    net_vertex.y /= vertecies.len() as f32;
    net_vertex.z /= vertecies.len() as f32;
    
    net_vertex
}

pub fn stl_to_mesh(file_name:&str, center: Point, scale: f32) -> Mesh {
    // import file
    let mut file: std::fs::File = OpenOptions::new().read(true).open(file_name).unwrap();
    let stl: IndexedMesh = read_stl(&mut file).unwrap();
    
    let offset: Point = center.sub(&mesh_center(&stl.vertices));

    // Parse Mesh
    let mut vertecies: Vec<Point> = vec![];
    for vertex in stl.vertices{
        // vertecies.push(Point{x: vertex[0]*scale,y: vertex[1]*scale,z: vertex[2]*scale}.add(&offset));
        // vertecies.push(Point{x: vertex[0]*scale,y: vertex[2]*scale,z: vertex[1]*scale}.add(&offset));
        vertecies.push(Point{x: vertex[0]*scale,y: vertex[2]*scale,z: -vertex[1]*scale}.add(&offset));
        // println!("({},{},{})",vertecies[vertecies.len()-1].x,vertecies[vertecies.len()-1].y,vertecies[vertecies.len()-1].z);
    }

    let mut tris: Vec<Tri> = vec![];
    for tri in stl.faces{
        let center: Point = calc_tri_center( &vertecies[tri.vertices[0]],&vertecies[tri.vertices[1]],&vertecies[tri.vertices[2]]);
        tris.push(Tri{vertecies: tri.vertices, normal:Point{x:tri.normal[0],y:tri.normal[2],z:tri.normal[1]}, center:center, area: calc_tri_area(&tri.vertices,&vertecies)});
        
        
        // let normal = normalize(&cross(&vertecies[tri.vertices[2]].sub(&vertecies[tri.vertices[0]]), &vertecies[tri.vertices[1]].sub(&vertecies[tri.vertices[0]])));
        // tris.push(Tri{vertecies: tri.vertices, normal:normal, center:center, area: calc_tri_area(&tri.vertices,&vertecies)});
    }

    println!("Parsed mesh, Tris: {}", tris.len());
    Mesh{tris:tris,vertecies:vertecies}
}

fn calc_tri_center(a:&Point,b:&Point,c:&Point) -> Point{
    Point{x:(a.x+b.x+c.x)/3.0,y:(a.y+b.y+c.y)/3.0,z:(a.z+b.z+c.z)/3.0}
}

fn calc_tri_area(vertex_idxs: &[usize;3], vetex_list: &Vec<Point>) -> f32{
    // Tri Area - https://math.stackexchange.com/questions/128991/how-to-calculate-the-area-of-a-3d-triangle
    cross(
        &vetex_list[vertex_idxs[1]].sub(&vetex_list[vertex_idxs[0]]),
        &vetex_list[vertex_idxs[2]].sub(&vetex_list[vertex_idxs[0]])
    ).len() * 0.5
}

// pub fn stl_to_bvh(file_name:&str,min_tri:usize, center: Point, scale: f32) -> BVHMesh {
pub fn stl_to_bvh(file_name:&str,max_depth:usize, center: Point, scale: f32) -> BVHMesh {
    let stl = stl_to_mesh(file_name, center, scale);
    
    let mut bounds: Vec<Bound> = vec![];
    let mut bvhq: VecDeque<Option<Bound>> = VecDeque::new();
    
    bvhq.push_back(make_bound(&stl.tris, &(0..stl.tris.len()).collect(), &stl.vertecies, 0, 3, 0.0, false, 0));
    
    let mut leaf_bounds:usize = 0;
    let mut leaf_tris:usize = 0;
    while bvhq.len() != 0{
        let popped_bound = bvhq.pop_front().expect("Error poping from queue");
        let bound: Bound;
        
        
        if popped_bound.is_none(){
            continue;
        }
        
        bound = popped_bound.unwrap();
        // println!("Parent: {}",bound.parent_index);
        bounds.push(bound.clone());
        
        // adds children to parent
        let cur_i = bounds.len() - 1;
        if bounds.len() != 1{
            bounds[bound.parent_index].children.push(cur_i);
        }
        // println!("Post Push Parent: {:?}",bounds[bound.parent_index]);

        // // last bound in brach if mithin tri max
        // if bound.tris.len() <= min_tri{
        //     // println!("Max Tris in Bound");
        //     continue;
        // }
        if bound.depth >= max_depth || bound.tris.len() == 1{
            leaf_bounds += 1;
            leaf_tris += bound.tris.len();
            // println!("Leaf Bound with {} Tris",bound.tris.len());
            continue;
        }

        
        // Surface Area Heuristic
        let mut best_div_axis: u8 = 3;
        let mut best_div_pos: f32 = 0.0;
        let mut best_div_cost: f32 = f32::INFINITY;
        
        // Debug
        let mut best_div_tri: usize = 0;

        let mut ran_gen = rand::thread_rng();
        // check tris per axis
        for axis in 0..3{
            // 15 rand tri
            for _tri_test in 0..15{
                let tri_idx = bound.tris[ran_gen.gen_range(0..bound.tris.len())];
                let tri = &stl.tris[tri_idx];


                let pos = match axis {
                    0 => tri.center.x,
                    1 => tri.center.y,
                    _ => tri.center.z,
                };

                let cost = calc_bvh_cost(&bound, &stl.tris, axis, pos);

                if cost < best_div_cost{
                    best_div_axis = axis;
                    best_div_pos = pos;
                    best_div_cost = cost;

                    // Debug
                    best_div_tri = tri_idx;
                }
            }
        }
        // println!("best cost at axis:{}, at pos:{}, with cost:{}",best_div_axis,best_div_pos,best_div_cost);

        // println!("{:?} --- {:?}",x1,x2);
        // println!("{:?} --- {:?}",y1,y2);
        // println!("{:?} --- {:?}\n",z1,z2);
        
        // adds children to queue
        let b1 = make_bound(
            &stl.tris, 
            &bound.tris, 
            &stl.vertecies, 
            cur_i, 
            best_div_axis, 
            best_div_pos,
            true,
            bound.depth + 1
        );
        let b2 = make_bound(
            &stl.tris, 
            &bound.tris, 
            &stl.vertecies, 
            cur_i, 
            best_div_axis, 
            best_div_pos,
            false,
            bound.depth + 1
        );

        
        // println!("Child 1 {:?}",b1);
        // println!("Child 2 {:?}",b2);
        bvhq.push_back(b1);
        bvhq.push_back(b2);

    }
                
    println!("Leaf bounds: {}    Leaf Tris: {}",leaf_bounds,leaf_tris);
    println!("Created BVH with {} bounds \nAverage of {} tris per leaf bound", bounds.len(), leaf_tris as f32/leaf_bounds as f32);
    BVHMesh { bounds: bounds, vertecies: stl.vertecies, tris: stl.tris }
}



fn make_bound(tri_list:&Vec<Tri>,tri_indecies:&Vec<usize>,vertex_list:&Vec<Point>,parent_index:usize,div_axis:u8,div_pos:f32,left_side:bool,depth:usize) -> Option<Bound>{
    // println!("Tri Count: {}",tri_list.len());
    // println!("Old Bounds: x-bounds: {:?},y-bounds: {:?},z-bounds: {:?}",x_bounds,y_bounds,z_bounds);
    let mut min_x: f32 = INFINITY;
    let mut min_y: f32 = INFINITY;
    let mut min_z: f32 = INFINITY;
    let mut max_x: f32 = -INFINITY;
    let mut max_y: f32 = -INFINITY;
    let mut max_z: f32 = -INFINITY;
    
    let mut new_tris:Vec<usize> = vec![];
    
    for tri_i in tri_indecies{
        let tri: &Tri = &tri_list[*tri_i];
        // checks if the center of the tri is in the bound
        let is_in_bounds: bool = 
            if div_axis == 3 {
                true // all triangles include for first bound - axis 3
            } else {
                let target_axis = match div_axis {
                    0 => tri.center.x,
                    1 => tri.center.y,
                    _ => tri.center.z,
                };
                
                match left_side {
                    true => target_axis < div_pos,
                    false => target_axis >= div_pos
                }
            };
        
        if !is_in_bounds {
            continue;
        }
        
        
        new_tris.push(*tri_i);
        for point_i in tri.vertecies{
            let point = &vertex_list[point_i];
            
            if point.x < min_x {min_x = point.x}
            if point.y < min_y {min_y = point.y}
            if point.z < min_z {min_z = point.z}
            
            if point.x > max_x {max_x = point.x}
            if point.y > max_y {max_y = point.y}
            if point.z > max_z {max_z = point.z}
        }
    }
    if new_tris.len() == 0{
        return None;
    }
    // println!("New Bound Tris: {}",new_tris.len());
    
    // println!("New Bounds: x-bounds: {:?},y-bounds: {:?},z-bounds: {:?}",[min_x,max_x],[min_y,max_y],[min_z,max_z]);
    let ret = Some(Bound { parent_index:parent_index, tris: new_tris, x: [min_x,max_x], y: [min_y,max_y], z: [min_z,max_z], children:vec![], depth});
    // println!("Calculated Bound: {:?}",ret);
    return ret;
}


fn calc_bvh_cost(bound: &Bound, tri_list:&Vec<Tri>, div_axis:u8, div_pos:f32) -> f32{

    let mut left_count:u32 = 0;
    let mut right_count:u32 = 0;
    let mut left_area:f32 = 0.0;   
    let mut right_area:f32 = 0.0;

    for tri_idx in &bound.tris{
        let tri = &tri_list[*tri_idx];

        if match div_axis {
            0 => tri.center.x,
            1 => tri.center.y,
            _ => tri.center.z
        } < div_pos {
            left_count += 1;
            left_area += tri.area;
        }
        else{
            right_count += 1;
            right_area += tri.area;
        }

    }

    (left_count as f32 *left_area) + (right_count as f32 *right_area)
}