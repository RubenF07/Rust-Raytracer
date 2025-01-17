// use std::time::Instant;

use std::time::Instant;

use rust_raytracer::camera::Camera;
use rust_raytracer::RGB;
use rust_raytracer::display::{self, start_render,display_image};
use rust_raytracer::material::{BVHDebug, Lambertian, Metal, NormalBased};
use rust_raytracer::renderer::{render,RedererParams};
use rust_raytracer::point::Point;
use rust_raytracer::scene::{MeshObject, Sphere, Scene};
// use rust_raytracer::display::display_image;

fn main() {
    let start = Instant::now();

    let mut scene = Scene::new(
        Camera::new(
        Point{x:130.0,y:50.0,z:-30.0},
        // Point{x:130.0,y:130.0,z:130.0},
        Point{x:-1.0,y:-0.0,z:0.0},
        // Point{x:-0.5,y:-0.7,z:-1.0},
        990,
        470,
        60.0),
        10
    );
    
    // Adding objects
    // scene.add_object(Box::new(MeshObject::new(
    //     &"croc.stl",
    //     7,
    //     Point { x: 0.0, y: 10.0, z: 0.0 },
    //     3.0,
    //     Box::new(Lambertian::new(RGB{r:255,g:0,b:0}))
    // )));
    // scene.add_object(Box::new(MeshObject::new(
    //     &"cube.stl",
    //     5,
    //     // Point { x: -50.0, y: 0.0, z: 50.0 },
    //     Point { x: -30.0, y: 0.0, z: 0.0 },
    //     2.0,
    //     // Box::new(Lambertian::new(RGB{r:0,g:255,b:0}))
    //     Box::new(Metal::new(0.8,RGB{r:0,g:255,b:0}))
    // )));
    // scene.add_object(Box::new(Sphere::new(
    //     Point{x: 10.0,y: 0.0,z: 0.0},
    //     17.0,
    //     Box::new(Lambertian::new(RGB{r:0,g:255,b:0}))
    //     // Box::new(Metal::new(1.0,RGB{r:0,g:0,b:255}))
    // )));
    scene.add_object(Box::new(Sphere::new(
        Point{x: 0.0,y: -2015.0,z: 0.0},
        2000.0,
        Box::new(Lambertian::new(RGB{r:255,g:0,b:0}))
        // Box::new(Metal::new(0.7,RGB{r:255,g:255,b:0}))
    )));
    
    
    // 131,688 Tris
    // scene.add_object(Box::new(MeshObject::new(
    //     &"f1.stl",
    //     12,
    //     Point { x: -70.0, y: 120.0, z: -10.0 },
    //     3.0,
    //     // Box::new(Lambertian::new(RGB{r:255,g:0,b:0}))
    //     Box::new(BVHDebug::new(400))
    // )));
    
    // 1,022 Tris 50 rays
    // Frame time: 75.00s
    // Total time: 75.18s

    //   
    scene.add_object(Box::new(MeshObject::new(
        &"polydragon.stl",
        1,
        Point { x: 0.0, y: 0.0, z: 30.0 },
        8.0,
        // Box::new(Metal::new(1.0,RGB{r:255,g:0,b:0}))
        Box::new(Lambertian::new(RGB{r:255,g:255,b:255}))
        // Box::new(BVHDebug::new(100))
    )));


    
    let render_params = RedererParams::new(
        scene,
        10,
        0.1);

    // start_render(render_params);
    display_image(render_params.scene.camera.width, render_params.scene.camera.height, &render(&render_params));
    
    // println!("Total render time: {}s",Instant::elapsed(&start).as_secs_f32());
    

}
