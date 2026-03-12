use std::time::Instant;

use rust_raytracer::camera::Camera;
use rust_raytracer::RGB;
use rust_raytracer::display::{start_render,display_image};
use rust_raytracer::material::{Lambertian, Metal};
use rust_raytracer::renderer::{render,RedererParams};
use rust_raytracer::point::Point;
use rust_raytracer::scene::{MeshObject, Sphere, Scene};

fn main() {
    let start = Instant::now();

    let mut scene = Scene::new(
        Camera::new(
        // Point{x:-85.0,y:50.0,z:-85.0},
        // Point{x:0.5,y:-0.1,z:0.5},
        Point{x:85.0,y:50.0,z:85.0},
        Point{x:-0.5,y:-0.1,z:-0.5},
        990,
        470,
        60.0),
        30
    );
    
    // Adding objects
    // scene.add_object(Box::new(MeshObject::new(
    //     &"croc.stl",
    //     7,
    //     Point { x: 0.0, y: 10.0, z: 0.0 },
    //     3.0,
    //     Box::new(Metal::new(1.0,RGB{r:255,g:0,b:0}))
    //     // Box::new(Lambertian::new(RGB{r:255,g:0,b:0}))
    // )));
    scene.add_object(Box::new(MeshObject::new(
        &"cube.stl",
        3,
        // Point { x: -50.0, y: 0.0, z: 50.0 },
        Point { x: -120.0, y: -10.0, z: 70.0 },
        2.8,
        // Box::new(NormalBased{})
        // Box::new(Lambertian::new(RGB{r:255,g:255,b:255}))
        Box::new(Metal::new(0.7, 0.1,RGB{r:255,g:0,b:255}))
    )));
    scene.add_object(Box::new(Sphere::new(
        Point{x: 80.0,y: 28.0,z: -70.0},
        38.0,
        Box::new(Metal::new(0.9, 0.0,RGB{r:0,g:0,b:255}))
    )));
    scene.add_object(Box::new(Sphere::new(
        Point{x: 0.0,y: -2015.0,z: 0.0},
        2000.0,
        Box::new(Lambertian::new(RGB{r:255,g:0,b:0}))
    )));
    
    
    // 131,688 Tris
    // scene.add_object(Box::new(MeshObject::new(
    //     &"f1.stl",
    //     12,
    //     Point { x: 20.0, y: 50.0, z: 10.0 },
    //     1.0,
    //     Box::new(Lambertian::new(RGB{r:0,g:255,b:0}))
    // )));
    
    // 1,022 Tris
    scene.add_object(Box::new(MeshObject::new(
        &"polydragon.stl",
        11,
        Point { x: 0.0, y: 0.0, z: 30.0 },
        8.0,
        Box::new(Lambertian::new(RGB{r:255,g:255,b:255}))
    )));

    // ##############################
    //  ↓ Use only one render test ↓
    // ##############################

    // Single render test
    let render_params = RedererParams::new(
        scene,
        30,
        0.15);
    display_image(render_params.scene.camera.width, render_params.scene.camera.height, &render(&render_params));
    println!("Total render time: {}s",Instant::elapsed(&start).as_secs_f32());
    
    // Constant render test
    // let render_params = RedererParams::new(
    //     scene,
    //     1,
    //     0.2);
    // start_render(render_params,30);
    
    
    
}
