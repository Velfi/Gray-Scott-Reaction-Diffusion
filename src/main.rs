mod gradient;
mod gray_scott_model;
mod utils;

#[allow(dead_code)]
mod gradient_presets;
#[allow(dead_code)]
mod model_presets;

use circular_queue::CircularQueue;
use gradient::ColorGradient;
use gray_scott_model::ReactionDiffusionSystem;
use log::{error, info, trace};
use pixels::{Pixels, SurfaceTexture};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use rayon::slice::ParallelSliceMut;
use std::time::Instant;
use winit::dpi::LogicalSize;
use winit::event::{Event, MouseButton};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;
const ASPECT_RATIO: f32 = WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32;
const MODEL_HEIGHT: usize = 270;
const MODEL_WIDTH: usize = (MODEL_HEIGHT as f32 * ASPECT_RATIO) as usize;
const CURRENT_MODEL: (f32, f32) = model_presets::SOLITON_COLLAPSE;

fn main() {
    let _ = dotenv::dotenv();
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64);
        match WindowBuilder::new()
            .with_title("Gray Scott Reaction Diffusion")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
        {
            Ok(w) => w,
            Err(e) => {
                eprintln!("Failed to create window: {}", e);
                return;
            }
        }
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        match Pixels::new(MODEL_WIDTH as u32, MODEL_HEIGHT as u32, surface_texture) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to create pixels surface: {}", e);
                return;
            }
        }
    };
    let mut world = World::new();
    let mut frame_time = 0.16;
    let mut time_of_last_frame_start = Instant::now();

    let mut frame_counter = 0;
    let mut fps_values = CircularQueue::with_capacity(5);
    let mut time_of_last_fps_counter_update = Instant::now();

    let _ = event_loop.run(move |event, target| {
        // Draw the current frame
        if let Event::WindowEvent {
            event: winit::event::WindowEvent::RedrawRequested,
            ..
        } = event
        {
            world.draw(pixels.frame_mut());
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                target.exit();
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                target.exit();
                return;
            }

            if input.mouse_pressed(MouseButton::Left) {
                trace!("Pressed LMB");
                world.is_left_mouse_button_held_down = true
            } else if input.mouse_released(MouseButton::Left) {
                trace!("Released LMB");
                world.is_left_mouse_button_held_down = false
            }

            if input.mouse_pressed(MouseButton::Right) {
                trace!("Pressed RMB");
                world.is_right_mouse_button_held_down = true
            } else if input.mouse_released(MouseButton::Right) {
                trace!("Released RMB");
                world.is_right_mouse_button_held_down = false
            }

            if let Some((x, y)) = input.cursor() {
                world.mouse_xy = (x, y);
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                let _ = pixels.resize_surface(size.width, size.height);
            }

            // Update internal state and request a redraw
            window.request_redraw();
            world.update(&pixels, frame_time);

            frame_time = time_of_last_frame_start.elapsed().as_secs_f32();
            time_of_last_frame_start = Instant::now();

            frame_counter += 1;

            if time_of_last_fps_counter_update.elapsed().as_secs() > 1 {
                time_of_last_fps_counter_update = Instant::now();
                let _ = fps_values.push(frame_counter);
                frame_counter = 0;

                let fps_sum: i32 = fps_values.iter().sum();
                let avg_fps = fps_sum as f32 / fps_values.len() as f32;
                info!("FPS {}", avg_fps.trunc());
            }
        }
    });
}

struct World {
    pub gradient: ColorGradient,
    pub is_left_mouse_button_held_down: bool,
    pub is_right_mouse_button_held_down: bool,
    pub reaction_diffusion_system: ReactionDiffusionSystem,
    pub mouse_xy: (f32, f32),
}

impl World {
    fn new() -> Self {
        Self {
            gradient: gradient_presets::new_rainbow(),
            is_left_mouse_button_held_down: false,
            is_right_mouse_button_held_down: false,
            reaction_diffusion_system: ReactionDiffusionSystem::new(
                MODEL_WIDTH,
                MODEL_HEIGHT,
                CURRENT_MODEL.0,
                CURRENT_MODEL.1,
                1.0,
                0.5,
            ),
            mouse_xy: (0.0, 0.0),
        }
    }
}

impl World {
    fn update(&mut self, pixels: &Pixels, frame_time: f32) {
        if self.is_left_mouse_button_held_down {
            if let Ok((x, y)) = pixels.window_pos_to_pixel(self.mouse_xy) {
                let x = x as isize;
                let y = y as isize;
                if x >= 0
                    && x < self.reaction_diffusion_system.width as isize
                    && y >= 0
                    && y < self.reaction_diffusion_system.height as isize
                {
                    self.reaction_diffusion_system.set(x, y, (0.0, 0.99));
                }
            }
        }

        self.reaction_diffusion_system.update(frame_time);
    }

    fn draw(&mut self, frame: &mut [u8]) {
        let uvs = self.reaction_diffusion_system.uvs().par_iter();
        let pixels = frame.par_chunks_exact_mut(4);
        pixels.zip_eq(uvs).for_each(|(pixel, (u, v))| {
            let value = 0.5 + 0.5 * (20.0 * v + 10.0 * u).sin();
            let t = (value + 1.0) / 2.0;

            // Display as color gradient
            let rgb = self.gradient.color_at_t(t);
            // // Display as grayscale
            // let t = (t * 255.0).round().clamp(0.0, 255.0) as u8;
            // let rgb = [t, t, t, 255];

            pixel.copy_from_slice(&rgb);
        });
    }
}
