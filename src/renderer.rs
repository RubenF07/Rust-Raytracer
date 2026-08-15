use crate::camera::Camera;
use crate::scene::Scene;
use std::time::Instant;
use rayon::prelude::*;

pub struct RendererParams{
    pub scene:Scene, 
    pub samples_per_pixel:u16, 
    pub anti_aliasing_strength:f32
}
impl RendererParams{
    pub fn new(scene: Scene, samples_per_pixel:u16, anti_aliasing_strength:f32) -> RendererParams{
        RendererParams{
            scene: scene,
            samples_per_pixel: samples_per_pixel,
            anti_aliasing_strength: anti_aliasing_strength
        }
    }
}

pub fn render(params: &RendererParams) -> Vec<u8>{
    let scene = &params.scene;
    let camera= &params.scene.camera;
    let samples_per_pixel = params.samples_per_pixel;
    let anti_aliasing_strength = params.anti_aliasing_strength;


    let start = Instant::now();
    let mut pixel_arr: Vec<u8> = vec![0;(camera.width*camera.height*3)as usize];
    
    let lines: Vec<(usize,&mut [u8])> = pixel_arr.chunks_mut(camera.width as usize * 3).enumerate().collect();
    
    lines.into_par_iter().for_each(|(h,line)| {
        render_line(h, line, &scene, &camera, &samples_per_pixel, &anti_aliasing_strength);
    });


    let end = Instant::elapsed(&start).as_secs_f32();
    println!("Frame rendered in {} seconds, {} fps",end,1.0/end);


    pixel_arr
}

fn render_line(row:usize, line: &mut [u8], scene:&Scene, camera:&Camera, samples_per_pixel:&u16, anti_aliasing_strength:&f32){
    let sky = scene.get_gradient(row as f32 / camera.height as f32);
    for w in 0..camera.width{
        let mut partial_color: [f32;3] = [0.0,0.0,0.0];
        for _ in 0..*samples_per_pixel{
            if let Some(color) = scene.get_color(&camera.pos, &camera.get_ray(&[w,row as  u32],&anti_aliasing_strength),0){
                partial_color[0] += (color.r as f32)/(*samples_per_pixel as f32);
                partial_color[1] += (color.g as f32)/(*samples_per_pixel as f32);
                partial_color[2] += (color.b as f32)/(*samples_per_pixel as f32);
            }
            else{
                partial_color[0] += (sky.r as f32)/(*samples_per_pixel as f32);
                partial_color[1] += (sky.g as f32)/(*samples_per_pixel as f32);
                partial_color[2] += (sky.b as f32)/(*samples_per_pixel as f32);
            }

        }
        line[(w as usize*3) + 0] = partial_color[0] as u8;
        line[(w as usize*3) + 1] = partial_color[1] as u8;
        line[(w as usize*3) + 2] = partial_color[2] as u8;
    }
    // println!("Line Rendered {}/{}",row+1,camera.height);
}
