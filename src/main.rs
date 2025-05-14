use circular_queue::CircularQueue;
use fontdue::Font;
use gray_scott_reaction_diffusion::{
    NutrientPattern, ReactionDiffusionSystem, lut_manager::LutManager, model_presets,
    renderer::Renderer,
};
use log::error;
use std::time::Instant;
use winit::dpi::LogicalSize;
use winit::event::{Event, MouseButton};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::{Window, WindowBuilder};
use winit_input_helper::WinitInputHelper;

const WINDOW_WIDTH: u32 = 1650;
const WINDOW_HEIGHT: u32 = 1050;
const ASPECT_RATIO: f32 = WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32;
const MODEL_HEIGHT: usize = WINDOW_HEIGHT as usize;
const MODEL_WIDTH: usize = (MODEL_HEIGHT as f32 * ASPECT_RATIO) as usize;

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

    // Create the renderer
    let mut renderer = futures::executor::block_on(Renderer::new(
        &window,
        MODEL_WIDTH as u32,
        MODEL_HEIGHT as u32,
    ));

    // Create the world asynchronously
    let mut world = futures::executor::block_on(World::new());

    // Initialize the selected LUT
    let available_luts = world.lut_manager.get_available_luts();
    if !available_luts.is_empty() {
        if let Ok(lut_data) = world
            .lut_manager
            .load_lut(&available_luts[world.current_lut_index])
        {
            renderer.update_lut(&lut_data);
        }
    }

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
            world.draw(&mut renderer, &window);
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                target.exit();
                return;
            }

            // Handle left mouse button for drawing
            if input.mouse_pressed(MouseButton::Left) {
                world.is_left_mouse_button_held_down = true
            } else if input.mouse_released(MouseButton::Left) {
                world.is_left_mouse_button_held_down = false
            }

            if let Some((x, y)) = input.cursor() {
                world.mouse_xy = (x, y);
            }

            // Keyboard controls
            if input.key_pressed(KeyCode::KeyC) {
                world.clear_screen();
            }
            if input.key_pressed(KeyCode::KeyN) {
                world.fill_with_noise();
            }
            if input.key_pressed(KeyCode::KeyG) {
                let shift_held =
                    input.key_held(KeyCode::ShiftLeft) || input.key_held(KeyCode::ShiftRight);
                world.cycle_lut(&mut renderer, shift_held);
            }
            if input.key_pressed(KeyCode::KeyP) {
                let shift_held =
                    input.key_held(KeyCode::ShiftLeft) || input.key_held(KeyCode::ShiftRight);
                world.cycle_preset(shift_held);
            }
            if input.key_pressed(KeyCode::KeyU) {
                let shift_held =
                    input.key_held(KeyCode::ShiftLeft) || input.key_held(KeyCode::ShiftRight);
                world.cycle_nutrient_pattern(shift_held);
            }
            if input.key_pressed(KeyCode::Slash) || input.key_pressed(KeyCode::Backslash) {
                world.show_help = !world.show_help;
            }
            if input.key_pressed(KeyCode::KeyF) {
                world.reverse_current_lut(&mut renderer);
            }
            if input.key_pressed(KeyCode::KeyY) {
                world.is_current_nutrient_pattern_reversed =
                    !world.is_current_nutrient_pattern_reversed;
                world
                    .reaction_diffusion_system
                    .toggle_nutrient_pattern_reversal();
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                renderer.resize(size);
            }

            // Update internal state and request a redraw
            window.request_redraw();
            world.update(&window);

            frame_counter += 1;

            // Update FPS counter at 30fps (every ~33.33ms)
            if time_of_last_fps_counter_update.elapsed().as_millis() > 33 {
                time_of_last_fps_counter_update = Instant::now();
                let _ = fps_values.push(frame_counter);
                frame_counter = 0;

                let fps_sum: i32 = fps_values.iter().sum();
                let avg_fps = fps_sum as f32 / fps_values.len() as f32;
                let (feed_rate, kill_rate) = world.get_current_preset_rates();
                window.set_title(&format!(
                    "Gray Scott Reaction Diffusion - {} (f={:.4}, k={:.4}) - {} - {} - FPS: {:.1}",
                    world.get_current_preset_name(),
                    feed_rate,
                    kill_rate,
                    world.get_current_nutrient_pattern_name(),
                    world.get_current_lut_name(),
                    avg_fps * 30.0
                ));
            }
        }
    });
}

pub struct World {
    pub is_left_mouse_button_held_down: bool,
    pub reaction_diffusion_system: ReactionDiffusionSystem,
    /// Physical mouse coordinates in window space (pixels).
    /// x ranges from 0 to window_width, y ranges from 0 to window_height.
    /// These are raw window coordinates before any scaling to simulation space.
    pub mouse_xy: (f32, f32),
    pub current_preset_index: usize,
    pub current_nutrient_pattern: NutrientPattern,
    pub is_current_nutrient_pattern_reversed: bool,
    pub show_help: bool,
    pub font: Font,
    pub lut_manager: LutManager,
    pub current_lut_index: usize,
    pub is_current_lut_reversed: bool,
}

impl World {
    async fn new() -> Self {
        // Set initial preset to Undulating
        let current_preset_index = 6;
        let (feed_rate, kill_rate) = match current_preset_index {
            0 => model_presets::BRAIN_CORAL,
            1 => model_presets::FINGERPRINT,
            2 => model_presets::MITOSIS,
            3 => model_presets::RIPPLES,
            4 => model_presets::SOLITON_COLLAPSE,
            5 => model_presets::U_SKATE_WORLD,
            6 => model_presets::UNDULATING,
            7 => model_presets::WORMS,
            _ => unreachable!(),
        };

        // Load the font
        let font = Font::from_bytes(
            include_bytes!("../Texturina-VariableFont_opsz,wght.ttf").as_ref(),
            fontdue::FontSettings::default(),
        )
        .expect("Font is valid");

        // Initialize LUT manager
        let lut_manager = LutManager::new();

        // Find the index of MATPLOTLIB_gist_ncar_r in available LUTs
        let available_luts = lut_manager.get_available_luts();
        let current_lut_index = available_luts
            .iter()
            .position(|name| name == "MATPLOTLIB_gist_ncar_r")
            .unwrap_or(0);
        let is_current_lut_reversed = false;

        // Create the world instance
        let mut world = Self {
            is_left_mouse_button_held_down: false,
            reaction_diffusion_system: ReactionDiffusionSystem::new(
                MODEL_WIDTH,
                MODEL_HEIGHT,
                feed_rate,
                kill_rate,
                1.0,
                0.5,
            )
            .await,
            mouse_xy: (0.0, 0.0),
            current_preset_index,
            current_nutrient_pattern: NutrientPattern::RadialGradient,
            is_current_nutrient_pattern_reversed: false,
            show_help: false,
            font,
            lut_manager,
            current_lut_index,
            is_current_lut_reversed,
        };

        // Fill with initial random noise
        world.fill_with_noise();

        // Set the initial nutrient pattern
        world.reaction_diffusion_system.set_nutrient_pattern(
            world.current_nutrient_pattern.as_u32(),
            world.is_current_nutrient_pattern_reversed,
        );

        world
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
                    if rand::random::<f32>() < 0.05 {
                        // 5% chance to add noise
                        let u = 0.5 + rand::random::<f32>() * 0.5; // Random U between 0.5 and 1.0
                        let v = 0.2 + rand::random::<f32>() * 0.6; // Random V between 0.2 and 0.8
                        (u, v)
                    } else {
                        (1.0, 0.0) // Default empty state
                    }
                })
            })
            .collect();

        self.reaction_diffusion_system.set_all(&values);
    }

    fn cycle_preset(&mut self, reverse: bool) {
        if reverse {
            self.current_preset_index = if self.current_preset_index == 0 {
                7
            } else {
                self.current_preset_index - 1
            };
        } else {
            self.current_preset_index = (self.current_preset_index + 1) % 8;
        }

        let (feed_rate, kill_rate) = match self.current_preset_index {
            0 => model_presets::BRAIN_CORAL,
            1 => model_presets::FINGERPRINT,
            2 => model_presets::MITOSIS,
            3 => model_presets::RIPPLES,
            4 => model_presets::SOLITON_COLLAPSE,
            5 => model_presets::U_SKATE_WORLD,
            6 => model_presets::UNDULATING,
            7 => model_presets::WORMS,
            _ => unreachable!(),
        };
        self.reaction_diffusion_system
            .update_rates(feed_rate, kill_rate);
    }

    fn cycle_nutrient_pattern(&mut self, reverse: bool) {
        let patterns = NutrientPattern::all();
        let current_idx = patterns
            .iter()
            .position(|&p| p == self.current_nutrient_pattern)
            .unwrap();
        let len = patterns.len();

        let new_idx = if reverse {
            if current_idx == 0 {
                len - 1
            } else {
                current_idx - 1
            }
        } else {
            (current_idx + 1) % len
        };

        self.current_nutrient_pattern = patterns[new_idx];
        self.reaction_diffusion_system.set_nutrient_pattern(
            self.current_nutrient_pattern.as_u32(),
            self.is_current_nutrient_pattern_reversed,
        );
    }

    fn cycle_lut(&mut self, renderer: &mut Renderer, reverse: bool) {
        let available_luts = self.lut_manager.get_available_luts();
        if !available_luts.is_empty() {
            let len = available_luts.len();
            if reverse {
                self.current_lut_index = if self.current_lut_index == 0 {
                    len - 1
                } else {
                    self.current_lut_index - 1
                };
            } else {
                self.current_lut_index = (self.current_lut_index + 1) % len;
            }

            // Reset the reversed state when changing LUTs
            self.is_current_lut_reversed = false;
            if let Ok(lut_data) = self
                .lut_manager
                .load_lut(&available_luts[self.current_lut_index])
            {
                renderer.update_lut(&lut_data);
            }
        }
    }

    fn get_current_lut_name(&self) -> String {
        let available_luts = self.lut_manager.get_available_luts();
        if available_luts.is_empty() {
            "No LUTs available".to_string()
        } else {
            available_luts[self.current_lut_index].clone()
        }
    }

    fn get_current_preset_name(&self) -> &'static str {
        match self.current_preset_index {
            0 => "Brain Coral",
            1 => "Fingerprint",
            2 => "Mitosis",
            3 => "Ripples",
            4 => "Soliton Collapse",
            5 => "U-Skate World",
            6 => "Undulating",
            7 => "Worms",
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
            7 => model_presets::WORMS,
            _ => unreachable!(),
        }
    }

    fn get_current_nutrient_pattern_name(&self) -> &'static str {
        self.current_nutrient_pattern.name()
    }

    fn update(&mut self, window: &Window) {
        if self.is_left_mouse_button_held_down {
            let physical_window_width = window.inner_size().width as f32;
            let physical_window_height = window.inner_size().height as f32;

            // Convert physical mouse coordinates to simulation coordinates
            let sim_x = ((self.mouse_xy.0 / physical_window_width) * MODEL_WIDTH as f32)
                .clamp(0.0, MODEL_WIDTH as f32 - 1.0) as isize;

            // Invert Y coordinate (window origin is top-left, so we need to flip Y)
            let sim_y = ((1.0 - (self.mouse_xy.1 / physical_window_height)) * MODEL_HEIGHT as f32)
                .clamp(0.0, MODEL_HEIGHT as f32 - 1.0) as isize;

            // Create a small area of effect
            let radius = 5;
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let nx = sim_x + dx;
                    let ny = sim_y + dy;
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
                        let nutrient_factor = 1.0; // The shader handles the nutrient pattern now

                        self.reaction_diffusion_system.set(
                            nx,
                            ny,
                            (0.5, 0.99 * factor * nutrient_factor),
                        );
                    }
                }
            }
        }

        self.reaction_diffusion_system.update();
    }

    fn draw(&mut self, renderer: &mut Renderer, window: &Window) {
        // Update the texture with the latest UV values
        let uvs = self.reaction_diffusion_system.uvs();
        renderer.update_texture(uvs);

        // Handle help text visibility
        if self.show_help {
            let formatted_help = format!(
                "Controls:
Left Mouse Button: Click and drag to seed the reaction
Right Mouse Button: Click and drag to interact with the reaction
C: Clear the screen
N: Fill the screen with noise
G: Cycle through different color gradients (hold SHIFT to cycle backwards)
P: Cycle through different reaction presets (hold SHIFT to cycle backwards)
U: Cycle through different nutrient patterns (hold SHIFT to cycle backwards)
F: Reverse current color gradient
Y: Reverse current nutrient pattern
? or \\: Toggle help overlay
ESC: Exit the application

Current Preset: {}
Current Nutrient Pattern: {} {}",
                self.get_current_preset_name(),
                self.get_current_nutrient_pattern_name(),
                if self.is_current_nutrient_pattern_reversed {
                    "(Reversed)"
                } else {
                    ""
                }
            );

            renderer.render_text(&formatted_help, &self.font, window.inner_size());
        } else {
            renderer.clear_text();
        }

        // Get the current frame's texture view
        if let Ok(frame) = renderer.surface.get_current_texture() {
            let view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            // Render the main scene
            if let Err(e) = renderer.render(&view) {
                error!("Render error: {}", e);
                return;
            }

            frame.present();
        }
    }

    fn reverse_current_lut(&mut self, renderer: &mut Renderer) {
        let available_luts = self.lut_manager.get_available_luts();
        if !available_luts.is_empty() {
            if let Ok(mut lut_data) = self
                .lut_manager
                .load_lut(&available_luts[self.current_lut_index])
            {
                // Toggle the reversed state
                self.is_current_lut_reversed = !self.is_current_lut_reversed;
                if self.is_current_lut_reversed {
                    lut_data.reverse();
                }
                renderer.update_lut(&lut_data);
            }
        }
    }

    #[allow(dead_code)]
    fn reverse_current_nutrient_pattern(&mut self) {
        self.is_current_nutrient_pattern_reversed = !self.is_current_nutrient_pattern_reversed;
        self.reaction_diffusion_system.set_nutrient_pattern(
            self.current_nutrient_pattern.as_u32(),
            self.is_current_nutrient_pattern_reversed,
        );
    }
}
