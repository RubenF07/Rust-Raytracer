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
use softbuffer::Surface;

#[derive(Default)]
struct App {
    window: Option<Rc<Window>>,
    last_update: Option<Instant>,
    frame_buffer: Option<Vec<u32>>,
    frame_count: u32,
    max_frame_count: u32,
    done_render: bool,
    height: u32,
    width: u32,
    current_window_size: (u32, u32),
    renderer_params: Option<RedererParams>,
    surface: Option<Surface<Rc<Window>,Rc<Window>>>,
}

impl App {
    fn new(render_params: RedererParams, max_frame_count:u32) -> Self {
        let width = render_params.scene.camera.width;
        let height = render_params.scene.camera.height;
        Self {
            window: None,
            last_update: None,
            frame_buffer: Some(vec![0; (width * height * 3) as usize]),
            frame_count: 0,
            max_frame_count: max_frame_count,
            done_render: false,
            height,
            width,
            current_window_size: (width, height),
            renderer_params: Some(render_params),
            surface: None,
        }
    }

    fn calculate_letterbox_dimensions(&self) -> (u32, u32, u32, u32) {
        let (window_width, window_height) = self.current_window_size;
        let source_aspect = self.width as f32 / self.height as f32;
        let window_aspect = window_width as f32 / window_height as f32;
        
        let (scaled_width, scaled_height, offset_x, offset_y) = if window_aspect > source_aspect {
            // Wider than source - vertical letterboxing
            let scaled_height = window_height;
            let scaled_width = (window_height as f32 * source_aspect) as u32;
            let offset_x = (window_width - scaled_width) / 2;
            (scaled_width, scaled_height, offset_x, 0)
        } else {
            // Taller than source - horizontal letterboxing
            let scaled_width = window_width;
            let scaled_height = (window_width as f32 / source_aspect) as u32;
            let offset_y = (window_height - scaled_height) / 2;
            (scaled_width, scaled_height, 0, offset_y)
        };
        
        (scaled_width, scaled_height, offset_x, offset_y)
    }

    fn update_and_render(&mut self) {
        let now = Instant::now();
        
        // Max of 20 FPS
        if let Some(last) = self.last_update {
            if now.duration_since(last) < Duration::from_millis(50) {
                return;
            }
        }
        
        // Get new frame data and update the window
        if let (Some(buffer), Some(_window)) = (&mut self.frame_buffer, &self.window) {
            // Get the new frame data
            if self.frame_count < self.max_frame_count{
                let new_buffer = getframe(self.renderer_params.as_ref().expect("Didn't have render details"));
                for i in 0..buffer.len(){
                    buffer[i] += new_buffer[i] as u32;
                }
                self.frame_count += 1;
            }
            else if self.frame_count == self.max_frame_count && !self.done_render{
                println!("Done rendering!");
                self.done_render = true;
            }
            
            let scalled_buffer: Vec<u8> = buffer.iter().map(|x| (x/self.frame_count) as u8).collect();

            // Convert RGB buffer to RGBA pixels (u32)
            let source_pixels: Vec<u32> = scalled_buffer.chunks_exact(3)
                .map(|rgb| {
                    let r = rgb[0] as u32;
                    let g = rgb[1] as u32;
                    let b = rgb[2] as u32;
                    (r << 16) | (g << 8) | b | (255 << 24) // RGBA
                })
                .collect();

            let (scaled_width, scaled_height, offset_x, offset_y) = self.calculate_letterbox_dimensions();
            if let Some(surface) = &mut self.surface {
                let mut buffer = surface.buffer_mut().unwrap();
                let (window_width, window_height) = self.current_window_size;
                
                // Fill entire buffer with black first
                for pixel in buffer.iter_mut() {
                    *pixel = 0xFF000000;
                }

                for y in 0..scaled_height {
                    for x in 0..scaled_width {
                        let src_x = (x * self.width / scaled_width) as usize;
                        let src_y = (y * self.height / scaled_height) as usize;
                        let src_index = src_y * self.width as usize + src_x;

                        let dst_x = x + offset_x;
                        let dst_y = y + offset_y;
                        let dst_index = dst_y * window_width + dst_x;
                        
                        if dst_index < buffer.len() as u32 && src_index < source_pixels.len() {
                            buffer[dst_index as usize] = source_pixels[src_index];
                        }
                    }
                }
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
                        NonZero::new(self.current_window_size.0).unwrap(),
                        NonZero::new(self.current_window_size.1).unwrap(),
                    )
                    .unwrap();
                }
                self.update_and_render();
                self.window.as_ref().unwrap().request_redraw();
            },
            WindowEvent::Resized(size) => {
                self.current_window_size = (size.width, size.height);
                
                if let Some(surface) = &mut self.surface {
                    surface.resize(
                        NonZero::new(size.width).unwrap(),
                        NonZero::new(size.height).unwrap(),
                    )
                    .unwrap();
                }
                self.window.as_ref().unwrap().request_redraw();
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


pub fn start_render(render_params: RedererParams, max_frame_count:u32) {
    let event_loop = EventLoop::new().unwrap();
    
    // Use Poll for smoother animation
    event_loop.set_control_flow(ControlFlow::Poll);
    
    let mut app = App::new(render_params,max_frame_count);
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