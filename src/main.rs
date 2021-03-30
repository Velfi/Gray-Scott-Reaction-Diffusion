mod gradient;
mod gray_scott_model;
mod utils;

#[allow(dead_code)]
mod gradient_presets;
#[allow(dead_code)]
mod model_presets;

use circular_queue::CircularQueue;
use gradient::ColorGradient;
use gray_scott_model::{ChemicalSpecies, ReactionDiffusionSystem};
use log::{debug, error, info};
use pixels::{Error, Pixels, SurfaceTexture};
use std::time::Instant;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WINDOW_WIDTH: u32 = 1600;
const WINDOW_HEIGHT: u32 = 1200;

const ASPECT_RATIO: f32 = WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32;

const MODEL_HEIGHT: u32 = 400;
const MODEL_WIDTH: u32 = (MODEL_HEIGHT as f32 * ASPECT_RATIO) as u32;

const HEIGHT_RATIO: f32 = WINDOW_HEIGHT as f32 / MODEL_HEIGHT as f32;
const WIDTH_RATIO: f32 = WINDOW_WIDTH as f32 / MODEL_WIDTH as f32;

const CURRENT_MODEL: (f32, f32) = model_presets::SOLITON_COLLAPSE;

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Gray Scott Reaction Diffusion")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(MODEL_WIDTH, MODEL_HEIGHT, surface_texture)?
    };
    let mut world = World::new();
    let mut frame_time = 0.16;
    let mut time_of_last_frame_start = Instant::now();

    let mut frame_counter = 0;
    let mut fps_values = CircularQueue::with_capacity(5);
    let mut time_of_last_fps_counter_update = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.get_frame());
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if input.mouse_pressed(0) {
                debug!("Pressed LMB");
                world.is_left_mouse_button_held_down = true
            } else if input.mouse_released(0) {
                debug!("Released LMB");
                world.is_left_mouse_button_held_down = false
            }

            if input.mouse_pressed(1) {
                debug!("Pressed RMB");
                world.is_right_mouse_button_held_down = true
            } else if input.mouse_released(1) {
                debug!("Released RMB");
                world.is_right_mouse_button_held_down = false
            }

            if let Some(xy) = input.mouse() {
                world.mouse_xy = xy;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize(size.width, size.height);
            }

            // Update internal state and request a redraw
            window.request_redraw();
            world.update(frame_time);

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
    pub fast_forward: bool,
    pub frames: usize,
    pub gradient: ColorGradient,
    pub is_left_mouse_button_held_down: bool,
    pub is_right_mouse_button_held_down: bool,
    pub previous_frame_start_time: Instant,
    pub reaction_diffusion_system: ReactionDiffusionSystem,
    pub mouse_xy: (f32, f32),
    pub previous_mouse_xy: Option<(f32, f32)>,
}

impl World {
    fn new() -> Self {
        Self {
            fast_forward: false,
            frames: 0,
            gradient: gradient_presets::new_rainbow(),
            is_left_mouse_button_held_down: false,
            is_right_mouse_button_held_down: false,
            previous_frame_start_time: Instant::now(),
            reaction_diffusion_system: ReactionDiffusionSystem::new(
                MODEL_WIDTH,
                MODEL_HEIGHT,
                CURRENT_MODEL.0,
                CURRENT_MODEL.1,
                1.0,
                0.5,
            ),
            mouse_xy: (0.0, 0.0),
            previous_mouse_xy: None,
        }
    }
}

impl World {
    fn update(&mut self, frame_time: f32) {
        if self.is_left_mouse_button_held_down {
            let (x, y) = self.mouse_xy;
            let (x, y) = (x / WIDTH_RATIO, y / HEIGHT_RATIO);

            self.reaction_diffusion_system
                .set(ChemicalSpecies::V, x as isize, y as isize, 0.99)
        }

        self.reaction_diffusion_system.update(frame_time);
    }

    fn draw(&mut self, frame: &mut [u8]) {
        let rds = &self.reaction_diffusion_system;
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let u = rds.get_by_index(ChemicalSpecies::U, i);
            let v = rds.get_by_index(ChemicalSpecies::V, i);
            let value = 0.5 + 0.5 * (20.0 * v + 10.0 * u).sin();
            let t = (value + 1.0) / 2.0;

            let rgb = self.gradient.color_at_t(t);

            pixel.copy_from_slice(&rgb);
        }
    }
}
