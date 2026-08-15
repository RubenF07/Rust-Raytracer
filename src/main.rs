use std::time::Instant;

use rust_raytracer::camera::Camera;
use rust_raytracer::RGB;
use rust_raytracer::display::{start_render,display_image};
use rust_raytracer::material::{Lambertian, Metal};
use rust_raytracer::renderer::{render,RendererParams};
use rust_raytracer::point::Point;
use rust_raytracer::scene::{MeshObject, Sphere, Scene};

fn main() {
    let start = Instant::now();

    let mut scene = Scene::new(
        Camera::new(
        // Point{x:-85.0,y:50.0,z:-85.0},
        // Point{x:0.5,y:-0.1,z:0.5},
        Point{x:-180.0/1.3,y:80.0,z:45.0 / 1.3},
        Point{x:0.37,y:-0.13,z:-0.1},
        820,
        470,
        60.0),
        30
    );
    
    // Adding objects

    scene.add_object(Box::new(MeshObject::new(
        &"cube.stl",
        3,
        // Point { x: -50.0, y: 0.0, z: 50.0 },
        Point { x: 10.0, y: -10.0, z: -80.0 },
        2.8,
        // Box::new(NormalBased{})
        // Box::new(Lambertian::new(RGB{r:255,g:255,b:255}))
        Box::new(Metal::new(0.9, 0.1,RGB{r:0,g:255,b:0}))
    )));
    scene.add_object(Box::new(Sphere::new(
        Point{x: 35.0,y: 32.0,z: 50.0},
        40.0,
        Box::new(Metal::new(0.9, 0.0,RGB{r:0,g:0,b:255}))
    )));
    scene.add_object(Box::new(Sphere::new(
        Point{x: 0.0,y: -2015.0,z: 0.0},
        2000.0,
        Box::new(Lambertian::new(RGB{r:204,g:61,b:61}))
    )));
    

    // ##############################
    //  ↓ Use only one render mode ↓
    // ##############################

    // Single render
    let render_params = RendererParams::new(
        scene,
        50,
        0.18);
    display_image(render_params.scene.camera.width, render_params.scene.camera.height, &render(&render_params));
    println!("Total render time: {}s",Instant::elapsed(&start).as_secs_f32());
    
    // Constant render
    // let render_params = RendererParams::new(
    //     scene,
    //     1,
    //     0.2);
    // start_render(render_params,30);
    
    
    
}
