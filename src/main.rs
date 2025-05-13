mod gradient;
mod gray_scott_model;
mod nutrient_presets;
mod utils;

#[allow(dead_code)]
mod gradient_presets;
#[allow(dead_code)]
mod model_presets;

use circular_queue::CircularQueue;
use fontdue::Font;
use gradient::ColorGradient;
use gray_scott_model::ReactionDiffusionSystem;
use log::{error, trace};
use nutrient_presets::NutrientPattern;
use pixels::{Pixels, SurfaceTexture};
use rand::SeedableRng;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use rayon::slice::ParallelSliceMut;
use std::time::Instant;
use winit::dpi::LogicalSize;
use winit::event::{Event, MouseButton};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WINDOW_WIDTH: u32 = 1920;
const WINDOW_HEIGHT: u32 = 1080;
const ASPECT_RATIO: f32 = WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32;
const MODEL_HEIGHT: usize = 1080;
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

    // Create the world asynchronously
    let mut world = futures::executor::block_on(World::new());

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

            // Keyboard controls
            if input.key_pressed(KeyCode::KeyC) {
                world.clear_screen();
            }
            if input.key_pressed(KeyCode::KeyN) {
                world.fill_with_noise();
            }
            if input.key_pressed(KeyCode::KeyG) {
                world.cycle_gradient();
            }
            if input.key_pressed(KeyCode::KeyP) {
                world.cycle_preset();
            }
            if input.key_pressed(KeyCode::KeyU) {
                world.cycle_nutrient_pattern();
            }
            if input.key_pressed(KeyCode::Slash) || input.key_pressed(KeyCode::Backslash) {
                world.show_help = !world.show_help;
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
            world.update(&pixels);

            frame_counter += 1;

            if time_of_last_fps_counter_update.elapsed().as_secs() > 1 {
                time_of_last_fps_counter_update = Instant::now();
                let _ = fps_values.push(frame_counter);
                frame_counter = 0;

                let fps_sum: i32 = fps_values.iter().sum();
                let avg_fps = fps_sum as f32 / fps_values.len() as f32;
                let (feed_rate, kill_rate) = world.get_current_preset_rates();
                window.set_title(&format!(
                    "Gray Scott Reaction Diffusion - {} (f={:.4}, k={:.4}) - {} - FPS: {:.1}",
                    world.get_current_preset_name(),
                    feed_rate,
                    kill_rate,
                    world.get_current_nutrient_pattern_name(),
                    avg_fps
                ));
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
    pub current_gradient_index: usize,
    pub current_preset_index: usize,
    pub current_nutrient_pattern_index: usize,
    pub nutrient_patterns: Vec<NutrientPattern>,
    pub show_help: bool,
    pub font: Font,
}

impl World {
    async fn new() -> Self {
        let nutrient_patterns = vec![
            nutrient_presets::uniform(),
            nutrient_presets::checkerboard(),
            nutrient_presets::diagonal_gradient(),
            nutrient_presets::radial_gradient(),
            nutrient_presets::vertical_stripes(),
            nutrient_presets::horizontal_stripes(),
            nutrient_presets::noise(),
        ];

        // Load the font
        let font = match include_bytes!("../Texturina-VariableFont_opsz,wght.ttf") {
            font_data if !font_data.is_empty() => {
                match Font::from_bytes(font_data.as_ref(), fontdue::FontSettings::default()) {
                    Ok(font) => font,
                    Err(e) => {
                        error!("Failed to load Texturina font: {}", e);
                        // Fallback to a simple built-in font
                        Font::from_bytes(
                            include_bytes!("../resources/fonts/DejaVuSansMono.ttf").as_ref(),
                            fontdue::FontSettings::default(),
                        )
                        .expect("Failed to load fallback font")
                    }
                }
            }
            _ => {
                error!("Font file is empty");
                Font::from_bytes(
                    include_bytes!("../resources/fonts/DejaVuSansMono.ttf").as_ref(),
                    fontdue::FontSettings::default(),
                )
                .expect("Failed to load fallback font")
            }
        };

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
            )
            .await,
            mouse_xy: (0.0, 0.0),
            current_gradient_index: 0,
            current_preset_index: 0,
            current_nutrient_pattern_index: 0,
            nutrient_patterns,
            show_help: false,
            font,
        }
    }

    fn clear_screen(&mut self) {
        let values: Vec<(f32, f32)> = vec![
            (1.0, 0.0);
            self.reaction_diffusion_system.width
                * self.reaction_diffusion_system.height
        ];
        self.reaction_diffusion_system.set_all(&values);
    }

    fn fill_with_noise(&mut self) {
        let values: Vec<(f32, f32)> = (0..self.reaction_diffusion_system.height)
            .flat_map(|_y| {
                (0..self.reaction_diffusion_system.width).map(move |_| {
                    let v = if rand::random::<f32>() > 0.5 {
                        0.0
                    } else {
                        0.8
                    };
                    (1.0, v)
                })
            })
            .collect();

        self.reaction_diffusion_system.set_all(&values);
    }

    fn cycle_gradient(&mut self) {
        self.current_gradient_index = (self.current_gradient_index + 1) % 5;
        self.gradient = match self.current_gradient_index {
            0 => gradient_presets::new_rainbow(),
            1 => gradient_presets::new_pink_and_blue(),
            2 => gradient_presets::new_protanopia_friendly(),
            3 => gradient_presets::new_magma(),
            4 => gradient_presets::new_monochrome(),
            _ => unreachable!(),
        };
    }

    fn cycle_preset(&mut self) {
        self.current_preset_index = (self.current_preset_index + 1) % 7;
        let (feed_rate, kill_rate) = match self.current_preset_index {
            0 => model_presets::SOLITON_COLLAPSE,
            1 => model_presets::BRAIN_CORAL,
            2 => model_presets::FINGERPRINT,
            3 => model_presets::MITOSIS,
            4 => model_presets::RIPPLES,
            5 => model_presets::U_SKATE_WORLD,
            6 => model_presets::UNDULATING,
            _ => unreachable!(),
        };
        self.reaction_diffusion_system
            .update_rates(feed_rate, kill_rate);
    }

    fn cycle_nutrient_pattern(&mut self) {
        self.current_nutrient_pattern_index =
            (self.current_nutrient_pattern_index + 1) % self.nutrient_patterns.len();
        self.reaction_diffusion_system
            .set_nutrient_pattern(self.current_nutrient_pattern_index as u32);
    }

    fn get_current_preset_name(&self) -> &'static str {
        match self.current_preset_index {
            0 => "Soliton Collapse",
            1 => "Brain Coral",
            2 => "Fingerprint",
            3 => "Mitosis",
            4 => "Ripples",
            5 => "U-Skate World",
            6 => "Undulating",
            _ => unreachable!(),
        }
    }

    fn get_current_preset_rates(&self) -> (f32, f32) {
        match self.current_preset_index {
            0 => model_presets::BRAIN_CORAL,
            1 => model_presets::FINGERPRINT,
            2 => model_presets::MITOSIS,
            3 => model_presets::RIPPLES,
            4 => model_presets::SOLITON_COLLAPSE,
            5 => model_presets::U_SKATE_WORLD,
            6 => model_presets::UNDULATING,
            _ => unreachable!(),
        }
    }

    fn get_current_nutrient_pattern_name(&self) -> &'static str {
        match self.current_nutrient_pattern_index {
            0 => "Uniform",
            1 => "Checkerboard",
            2 => "Diagonal Gradient",
            3 => "Radial Gradient",
            4 => "Vertical Stripes",
            5 => "Horizontal Stripes",
            6 => "Noise",
            _ => unreachable!(),
        }
    }
}

impl World {
    fn update(&mut self, pixels: &Pixels) {
        if self.is_left_mouse_button_held_down {
            if let Ok((x, y)) = pixels.window_pos_to_pixel(self.mouse_xy) {
                let x = x as isize;
                let y = y as isize;

                // Create a small area of effect
                let radius = 5;
                for dy in -radius..=radius {
                    for dx in -radius..=radius {
                        let nx = x + dx;
                        let ny = y + dy;
                        if nx >= 0
                            && nx < self.reaction_diffusion_system.width as isize
                            && ny >= 0
                            && ny < self.reaction_diffusion_system.height as isize
                        {
                            // Calculate normalized distance from center (0 to 1)
                            let distance = ((dx * dx + dy * dy) as f32).sqrt() / radius as f32;

                            // Hard edge brush - constant value within radius
                            let factor = if distance < 1.0 { 1.0 } else { 0.0 };

                            // Apply nutrient pattern
                            let nutrient_factor = (self.nutrient_patterns
                                [self.current_nutrient_pattern_index])(
                                nx as usize,
                                ny as usize,
                                self.reaction_diffusion_system.width,
                                self.reaction_diffusion_system.height,
                            );

                            self.reaction_diffusion_system.set(
                                nx,
                                ny,
                                (0.5, 0.99 * factor * nutrient_factor),
                            );
                        }
                    }
                }
            }
        }

        self.reaction_diffusion_system.update();
    }

    fn draw(&mut self, frame: &mut [u8]) {
        let uvs = self.reaction_diffusion_system.uvs().par_iter();
        let pixels = frame.par_chunks_exact_mut(4);
        pixels.zip_eq(uvs).for_each(|(pixel, (u, v))| {
            let value = 0.5 + 0.5 * (20.0 * v + 10.0 * u).sin();
            let t = (value + 1.0) / 2.0;

            // Display as color gradient
            let rgb = self.gradient.color_at_t(t);
            pixel.copy_from_slice(&rgb);
        });

        // Draw help text if enabled
        if self.show_help {
            let help_text = [
                "Controls:",
                "Left Mouse Button: Draw",
                "Right Mouse Button: Erase",
                "C: Clear screen",
                "N: Fill with noise",
                "G: Cycle gradient",
                "P: Cycle preset",
                "U: Cycle nutrient pattern",
                "/ or \\: Toggle help",
                "ESC: Exit",
            ];

            // Draw a semi-transparent background
            let bg_color = [0, 0, 0, 128];
            let margin = 20; // Margin size in pixels
            for y in margin..(500 + margin) {
                for x in margin..(600 + margin) {
                    let idx = (y * MODEL_WIDTH + x) * 4;
                    if idx + 3 < frame.len() {
                        frame[idx..idx + 4].copy_from_slice(&bg_color);
                    }
                }
            }

            // Draw text with proper font rendering
            let mut y = 40.0 + margin as f32; // Add margin to starting y position
            for line in help_text.iter() {
                let mut x = 20.0 + margin as f32; // Add margin to starting x position
                for c in line.chars() {
                    let (metrics, bitmap) = self.font.rasterize(c, 32.0);

                    // Calculate the baseline offset using ymin
                    let baseline_offset = metrics.height as f32 + metrics.ymin as f32;

                    for (i, alpha) in bitmap.iter().enumerate() {
                        let px = x + (i % metrics.width) as f32;
                        let py = y + (i / metrics.width) as f32 - baseline_offset;

                        if px >= 0.0
                            && px < MODEL_WIDTH as f32
                            && py >= 0.0
                            && py < MODEL_HEIGHT as f32
                        {
                            let idx = (py as usize * MODEL_WIDTH + px as usize) * 4;
                            if idx + 3 < frame.len() {
                                // Set white text with alpha
                                frame[idx] = 255;
                                frame[idx + 1] = 255;
                                frame[idx + 2] = 255;
                                frame[idx + 3] = *alpha;
                            }
                        }
                    }
                    x += metrics.advance_width;
                }
                y += 40.0;
            }
        }
    }
}
