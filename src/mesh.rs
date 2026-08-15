use rand::Rng;
use stl_io::{read_stl, IndexedMesh, Vector};
use std::{collections::VecDeque, f32::INFINITY, fs::OpenOptions, vec};
use crate::point::{cross, get_tri_intersect, normalize, Point};

#[derive(Debug)]
pub struct Tri{
    // indices of mesh vertices
    pub vertices: [usize; 3],
    pub normal: Point,
    pub center: Point,
    pub area: f32,
    pub determ: Point,
}

#[derive(Debug)]
pub struct Mesh{
    pub vertices: Vec<Point>,
    pub tris: Vec<Tri>
}


pub struct BVHHitReturn{
    tri_idx: Option<usize>,
    tri_dist: Option<f32>,
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
    pub vertices: Vec<Point>,
    pub tris: Vec<Tri>,
}
impl BVHMesh{
    // returns (tri dist, tri normal)
    pub fn get_final_tri_hit(&self, pos:&Point, dir: &Point) -> Option<(f32,&Point)>{
        let tri: BVHHitReturn = self.get_tri_in_bvh(&dir, &pos, 0);

        if tri.tri_idx.is_some(){
            return Some((tri.tri_dist.unwrap(), &self.tris[tri.tri_idx.unwrap()].normal));
        }
        
        // No hit
        None
    }

    // return (tri_idx, dist)
    fn get_leaf_bound_tri_hit(&self, ray:&Point, origin: &Point, bound: &Bound) -> (Option<usize>,Option<f32>){
        let mut best_dist = f32::INFINITY;
        let mut best_tri_idx = 0;

        for tri_idx in &bound.tris{
            let tri: &Tri = &self.tris[*tri_idx];

            let hit = get_tri_intersect(&origin ,&ray, &self.vertices[tri.vertices[0]], &self.vertices[tri.vertices[1]], &self.vertices[tri.vertices[2]],&tri.determ);
            if hit.is_some_and(|x| x < best_dist && x > 0.0){
                best_dist = hit.unwrap();
                best_tri_idx = *tri_idx;
            }
        }

        return if best_dist != f32::INFINITY{
            (Some(best_tri_idx),Some(best_dist))
        } else {
            (None,None)
        };
    }

    fn get_tri_in_bvh(&self,ray: &Point, origin: &Point, bound_idx: usize) -> BVHHitReturn{
        let bound: &Bound = &self.bounds[bound_idx];
        if bound.children.len() == 0{
            // Leaf Node
            let tri_info = self.get_leaf_bound_tri_hit(&ray, &origin, bound);
            return BVHHitReturn{tri_idx: tri_info.0, tri_dist: tri_info.1, bound_checks:0, bounds_checked:vec![bound_idx]};
        }

        // Bounds should only have 0 or 2 children
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
            let (closer_bound_idx,farther_bound_idx,farther_dist) = 
                if first_hit.unwrap() < second_hit.unwrap(){
                    (bound.children[0],bound.children[1],second_hit.unwrap())
                } else {
                    (bound.children[1],bound.children[0],first_hit.unwrap())
                };

            let closer_hit = self.get_tri_in_bvh(&ray, &origin, closer_bound_idx);
            child_bounds_checked.extend(closer_hit.bounds_checked);
            child_checks += closer_hit.bound_checks;

            if closer_hit.tri_idx.is_some() && closer_hit.tri_dist.unwrap() < farther_dist{
                return BVHHitReturn{tri_idx: closer_hit.tri_idx, tri_dist:closer_hit.tri_dist, bound_checks:child_checks, bounds_checked: child_bounds_checked};
            }
            
            let farther_hit = self.get_tri_in_bvh(&ray, &origin, farther_bound_idx);
            child_bounds_checked.extend(farther_hit.bounds_checked);
            child_checks += farther_hit.bound_checks;
            
            if farther_hit.tri_idx.is_some(){
                if closer_hit.tri_idx.is_some() && farther_hit.tri_dist.unwrap() > closer_hit.tri_dist.unwrap(){
                    return BVHHitReturn{tri_idx: closer_hit.tri_idx, tri_dist:closer_hit.tri_dist, bound_checks:child_checks, bounds_checked: child_bounds_checked};
                }
                return BVHHitReturn{tri_idx: farther_hit.tri_idx, tri_dist:farther_hit.tri_dist, bound_checks:child_checks, bounds_checked: child_bounds_checked};
            }
            else if closer_hit.tri_idx.is_some() {
                return BVHHitReturn{tri_idx: closer_hit.tri_idx, tri_dist:closer_hit.tri_dist, bound_checks:child_checks, bounds_checked: child_bounds_checked};
            }

            return BVHHitReturn{tri_idx: None, tri_dist:None, bound_checks:child_checks, bounds_checked: child_bounds_checked};
        }
        
        // Only one hit
        if first_hit.is_some() || second_hit.is_some(){
            let only_hit = if first_hit.is_some() {self.get_tri_in_bvh(&ray, &origin, bound.children[0])} else {self.get_tri_in_bvh(&ray, &origin, bound.children[1])};
                child_bounds_checked.extend(only_hit.bounds_checked);
                child_checks += only_hit.bound_checks;
                
                if only_hit.tri_idx.is_some(){
                    return BVHHitReturn{tri_idx: only_hit.tri_idx, tri_dist:only_hit.tri_dist, bound_checks:child_checks, bounds_checked: child_bounds_checked};
                }
        }

        // None hit or all returned empty
        return BVHHitReturn{tri_idx: None, tri_dist:None, bound_checks:child_checks, bounds_checked: child_bounds_checked};
    
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

fn mesh_center(vertices: &Vec<Vector<f32>>) -> Point{
    let mut net_vertex = Point::default();
    for vertex in vertices{
        net_vertex.x += vertex[0];
        net_vertex.y += vertex[1];
        net_vertex.z += vertex[2];
    }

    net_vertex.x /= vertices.len() as f32;
    net_vertex.y /= vertices.len() as f32;
    net_vertex.z /= vertices.len() as f32;
    
    net_vertex
}

pub fn stl_to_mesh(file_name:&str, center: Point, scale: f32) -> Mesh {
    // import file
    let mut file: std::fs::File = OpenOptions::new().read(true).open(file_name).unwrap();
    let stl: IndexedMesh = read_stl(&mut file).unwrap();
    
    let offset: Point = center.sub(&mesh_center(&stl.vertices));

    // Parse Mesh
    let mut vertices: Vec<Point> = vec![];
    for vertex in stl.vertices{
        vertices.push(Point{x: vertex[0]*scale,y: vertex[2]*scale,z: -vertex[1]*scale}.add(&offset));
        // println!("({},{},{})",vertices[vertices.len()-1].x,vertices[vertices.len()-1].y,vertices[vertices.len()-1].z);
    }

    let mut tris: Vec<Tri> = vec![];
    for tri in stl.faces{
        let center: Point = calc_tri_center( &vertices[tri.vertices[0]],&vertices[tri.vertices[1]],&vertices[tri.vertices[2]]);

        let edge_ab = vertices[tri.vertices[1]].sub(&vertices[tri.vertices[0]]);
        let edge_ac = vertices[tri.vertices[2]].sub(&vertices[tri.vertices[0]]);
        
        let normal = normalize(&cross(&vertices[tri.vertices[1]].sub(&vertices[tri.vertices[0]]), &vertices[tri.vertices[2]].sub(&vertices[tri.vertices[0]])));
        tris.push(Tri{vertices: tri.vertices, normal:normal, center:center, area: calc_tri_area(&tri.vertices,&vertices), determ: cross(&edge_ab, &edge_ac)});
    }

    println!("Parsed mesh, Tris: {}", tris.len());
    Mesh{tris:tris,vertices:vertices}
}

fn calc_tri_center(a:&Point,b:&Point,c:&Point) -> Point{
    Point{x:(a.x+b.x+c.x)/3.0,y:(a.y+b.y+c.y)/3.0,z:(a.z+b.z+c.z)/3.0}
}

fn calc_tri_area(vertex_idxs: &[usize;3], vertex_list: &Vec<Point>) -> f32{
    // Tri Area - https://math.stackexchange.com/questions/128991/how-to-calculate-the-area-of-a-3d-triangle
    cross(
        &vertex_list[vertex_idxs[1]].sub(&vertex_list[vertex_idxs[0]]),
        &vertex_list[vertex_idxs[2]].sub(&vertex_list[vertex_idxs[0]])
    ).len() * 0.5
}

pub fn stl_to_bvh(file_name:&str,max_depth:usize, center: Point, scale: f32) -> BVHMesh {
    let stl = stl_to_mesh(file_name, center, scale);
    
    let mut bounds: Vec<Bound> = vec![];
    let mut bvhq: VecDeque<Option<Bound>> = VecDeque::new();
    
    bvhq.push_back(make_bound(&stl.tris, &(0..stl.tris.len()).collect(), &stl.vertices, 0, 3, 0.0, false, 0));
    
    let mut leaf_bounds:usize = 0;
    let mut leaf_tris:usize = 0;
    while bvhq.len() != 0{
        let popped_bound = bvhq.pop_front().expect("Error poping from queue");
        let bound: Bound;
        
        if popped_bound.is_none(){
            continue;
        }
        
        bound = popped_bound.unwrap();
        bounds.push(bound.clone());
        
        // adds children to parent
        let cur_i = bounds.len() - 1;
        if bounds.len() != 1{
            bounds[bound.parent_index].children.push(cur_i);
        }

        if bound.depth >= max_depth || bound.tris.len() == 1{
            leaf_bounds += 1;
            leaf_tris += bound.tris.len();
            continue;
        }

        // Surface Area Heuristic
        let mut best_div_axis: u8 = 3;
        let mut best_div_pos: f32 = 0.0;
        let mut best_div_cost: f32 = f32::INFINITY;

        let mut ran_gen = rand::thread_rng();
        // check tris per axis
        for axis in 0..3{
            // 10 rand tri, 1 check of split axis
            for tri_test in 0..11{
                let pos = if tri_test == 0 {
                    // First check split axis
                    match axis {
                        0 => (bound.x[1]+bound.x[0])*0.5,
                        1 => (bound.y[1]+bound.y[0])*0.5,
                        _ => (bound.z[1]+bound.z[0])*0.5,
                    }
                }
                else{
                    let tri_idx = bound.tris[ran_gen.gen_range(0..bound.tris.len())];
                    let tri = &stl.tris[tri_idx];
                    match axis {
                        0 => tri.center.x,
                        1 => tri.center.y,
                        _ => tri.center.z,
                    }
                };
                
                let cost = calc_bvh_cost(&bound, &stl.tris, axis, pos);

                if cost < best_div_cost{
                    best_div_axis = axis;
                    best_div_pos = pos;
                    best_div_cost = cost;
                }
            }
        }
        
        // adds children to queue
        let b1 = make_bound(
            &stl.tris, 
            &bound.tris, 
            &stl.vertices, 
            cur_i, 
            best_div_axis, 
            best_div_pos,
            true,
            bound.depth + 1
        );
        let b2 = make_bound(
            &stl.tris, 
            &bound.tris, 
            &stl.vertices, 
            cur_i, 
            best_div_axis, 
            best_div_pos,
            false,
            bound.depth + 1
        );

        bvhq.push_back(b1);
        bvhq.push_back(b2);
    }
                
    println!("Leaf bounds: {}    Leaf Tris: {}",leaf_bounds,leaf_tris);
    println!("Created BVH with {} bounds \nAverage of {} tris per leaf bound", bounds.len(), (leaf_tris as f32 * 100.0/leaf_bounds as f32).round() * 0.01);
    BVHMesh { bounds: bounds, vertices: stl.vertices, tris: stl.tris }
}



fn make_bound(tri_list:&Vec<Tri>,tri_indices:&Vec<usize>,vertex_list:&Vec<Point>,parent_index:usize,div_axis:u8,div_pos:f32,left_side:bool,depth:usize) -> Option<Bound>{
    let mut min_x: f32 = INFINITY;
    let mut min_y: f32 = INFINITY;
    let mut min_z: f32 = INFINITY;
    let mut max_x: f32 = -INFINITY;
    let mut max_y: f32 = -INFINITY;
    let mut max_z: f32 = -INFINITY;
    
    let mut new_tris:Vec<usize> = vec![];
    
    for tri_i in tri_indices{
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
        for point_i in tri.vertices{
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
    
    let ret = Some(Bound { parent_index:parent_index, tris: new_tris, x: [min_x,max_x], y: [min_y,max_y], z: [min_z,max_z], children:vec![], depth});
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