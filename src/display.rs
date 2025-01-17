use crate::RGB;
use crate::renderer::{render, RedererParams};
use std::fs::File;
use std::num::NonZero;
use std::rc::Rc;
use image::{codecs::png::PngEncoder, ExtendedColorType::Rgb8, ImageEncoder};

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
use std::time::{Duration, Instant};
use softbuffer::{Buffer, Context, Surface};

#[derive(Default)]
struct App {
    window: Option<Rc<Window>>,
    last_update: Option<Instant>,
    frame_buffer: Option<Vec<u32>>,
    frame_count: u32,
    height: u32,
    width: u32,
    renderer_params: Option<RedererParams>,
    surface: Option<Surface<Rc<Window>,Rc<Window>>>,
}

impl App {
    fn new(render_params: RedererParams) -> Self {
        Self {
            window: None,
            last_update: None,
            frame_buffer: Some(vec![0; (render_params.scene.camera.width * render_params.scene.camera.height * 3) as usize]),
            frame_count: 0,
            height: render_params.scene.camera.height,
            width: render_params.scene.camera.width,
            renderer_params: Some(render_params),
            surface: None,
        }
    }

    fn update_and_render(&mut self) {
        let now = Instant::now();
        
        // Check if we need to update (20 FPS = 50ms between frames)
        if let Some(last) = self.last_update {
            if now.duration_since(last) < Duration::from_millis(50) {
                return;
            }
        }
        // self.renderer_params.as_mut().unwrap().scene.camera.pos.y += 1.0;
        
        // Get new frame data and update the window
        if let (Some(buffer), Some(_window)) = (&mut self.frame_buffer, &self.window) {
            // Get the new frame data
            
            let new_buffer = getframe(self.renderer_params.as_ref().expect("Didn't have render details"));
            for i in 0..buffer.len(){
                buffer[i] += new_buffer[i] as u32;
            }
            self.frame_count += 1;
            
            let scalled_buffer: Vec<u8> = buffer.iter().map(|x| (x/self.frame_count) as u8).collect();
            // let scalled_buffer: Vec<u8> = new_buffer;

            // Convert RGB buffer to RGBA pixels (u32)
            let pixels: Vec<u32> = scalled_buffer.chunks_exact(3)
                .map(|rgb| {
                    let r = rgb[0] as u32;
                    let g = rgb[1] as u32;
                    let b = rgb[2] as u32;
                    (r << 16) | (g << 8) | b | (255 << 24) // RGBA
                })
                .collect();

            // Update the surface
            if let Some(surface) = &mut self.surface {
                let mut buffer = surface.buffer_mut().unwrap();
                buffer.copy_from_slice(&pixels);
                buffer.present().unwrap();
            }
        }
        
        self.last_update = Some(now);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attributes = Window::default_attributes()
            .with_title("3D Renderer")
            .with_inner_size(winit::dpi::PhysicalSize::new(self.width, self.height));

        let window = Rc::new(event_loop.create_window(attributes).unwrap());
        
        // Create context with window as display
        let context = softbuffer::Context::new(window.clone()).unwrap();
        let surface = softbuffer::Surface::new(&context, window.clone()).unwrap();
        
        
        self.window = Some(window);
        self.surface = Some(surface);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed. Stopping now");
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                if let Some(surface) = &mut self.surface {
                    surface.resize(
                        NonZero::new(self.width).unwrap(),
                        NonZero::new(self.height).unwrap(),
                    )
                    .unwrap();
                }
                self.update_and_render();
                // Request next frame
                self.window.as_ref().unwrap().request_redraw();
            },
            WindowEvent::Resized(size) => {
                // Update surface if window is resized
                if let Some(surface) = &mut self.surface {
                    surface.resize(
                        NonZero::new(self.width).unwrap(),
                        NonZero::new(self.height).unwrap(),
                    )
                    .unwrap();
                }
            },
            _ => (),
        }
    }
    
    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        // Ensure we keep updating even when no other events occur
        self.window.as_ref().unwrap().request_redraw();
    }
}

fn getframe(reder_params: &RedererParams) -> Vec<u8>{
    render(reder_params)
}


pub fn start_render(render_params: RedererParams) {
    let event_loop = EventLoop::new().unwrap();
    
    // Use Poll for smoother animation
    event_loop.set_control_flow(ControlFlow::Poll);
    
    let mut app = App::new(render_params);
    event_loop.run_app(&mut app);
}




pub fn display_image(w:u32,h:u32,pixels: &[u8]){
    let output = File::create("output.png").expect("Failed to create file!");
    let encoder = PngEncoder::new(output);
    encoder.write_image(pixels, w, h, Rgb8).expect("Failed to write to file!");
}
pub fn display_image_from_rgb(w:u32,h:u32,rgb_arr: Vec<RGB>){
    assert_eq!(rgb_arr.len(),(w*h) as usize);
    let mut pixels: Vec<u8> = vec![];
    for rgb in rgb_arr{
        pixels.push(rgb.r);
        pixels.push(rgb.g);
        pixels.push(rgb.b);
    }
    display_image(w, h, &pixels as &[u8]);
}