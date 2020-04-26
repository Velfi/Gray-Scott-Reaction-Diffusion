mod gradient;
pub mod gradient_presets;
mod gray_scott_model;
pub mod model_presets;
mod utils;

use ggez::{
    event::{self, KeyCode, KeyMods, MouseButton},
    graphics::{self, Color, DrawParam},
    mint::Vector2,
    Context, GameResult,
};
use gradient::ColorGradient;
use gray_scott_model::{ChemicalSpecies, ReactionDiffusionSystem};
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba, RgbaImage};
use std::{env, path};

const WINDOW_HEIGHT: usize = 900;
const WINDOW_WIDTH: usize = 1440;

const ASPECT_RATIO: f32 = WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32;

const MODEL_HEIGHT: usize = 300;
const MODEL_WIDTH: usize = (MODEL_HEIGHT as f32 * ASPECT_RATIO) as usize;

const HEIGHT_RATIO: f32 = WINDOW_HEIGHT as f32 / MODEL_HEIGHT as f32;
const WIDTH_RATIO: f32 = WINDOW_WIDTH as f32 / MODEL_WIDTH as f32;

const CURRENT_MODEL: (f32, f32) = model_presets::BRAIN_CORAL;

struct MainState {
    frames: usize,
    reaction_diffusion_system: ReactionDiffusionSystem,
    fast_forward: bool,
    gradient: ColorGradient,
    is_mouse_button_pressed: bool,
}

impl MainState {
    fn new(_ctx: &mut Context) -> GameResult<MainState> {
        let s = MainState {
            frames: 0,
            reaction_diffusion_system: ReactionDiffusionSystem::new(
                MODEL_WIDTH,
                MODEL_HEIGHT,
                CURRENT_MODEL.0,
                CURRENT_MODEL.1,
                1.0,
                0.5,
            ),
            fast_forward: false,
            gradient: gradient_presets::new_rainbow(),
            is_mouse_button_pressed: false,
        };

        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let dt = ggez::timer::delta(ctx);
        self.reaction_diffusion_system.update(dt.as_millis() as f32);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::new(0.0, 0.0, 0.0, 1.0));

        if !self.fast_forward {
            let dynamic_image = DynamicImage::ImageRgba8(rds_to_rgba_image(
                &self.reaction_diffusion_system,
                &self.gradient,
            ));

            let image = graphics::Image::from_rgba8(
                ctx,
                dynamic_image.width() as u16,
                dynamic_image.height() as u16,
                &dynamic_image.to_rgba().into_raw(),
            )?;

            graphics::draw(
                ctx,
                &image,
                DrawParam {
                    scale: Vector2 {
                        x: WIDTH_RATIO,
                        y: HEIGHT_RATIO,
                    },
                    ..Default::default()
                },
            )?;
        }
        graphics::present(ctx)?;

        self.frames += 1;
        if (self.frames % 100) == 0 {
            self.frames = 1;
            println!("FPS: {}", ggez::timer::fps(ctx));
        }

        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
        if button == MouseButton::Left {
            self.is_mouse_button_pressed = true;
        }
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, _x: f32, _y: f32) {
        if button == MouseButton::Left {
            self.is_mouse_button_pressed = false;
        }
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _xrel: f32, _yrel: f32) {
        if self.is_mouse_button_pressed {
            let x = x as f32 / WIDTH_RATIO;
            let y = y as f32 / HEIGHT_RATIO;

            self.reaction_diffusion_system
                .set(&ChemicalSpecies::V, x as isize, y as isize, 0.99)
        }
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            KeyCode::Escape => event::quit(ctx),
            KeyCode::F => {
                self.fast_forward = !self.fast_forward;
                println!(
                    "Fast Forward {}",
                    if self.fast_forward { "On" } else { "Off" }
                );
            }
            _ => (),
        }
    }
}

pub fn main() {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let (ctx, events_loop) =
        &mut ggez::ContextBuilder::new("Reaction Diffusion System", "Zelda Hessler")
            .window_mode(ggez::conf::WindowMode {
                width: WINDOW_WIDTH as f32,
                height: WINDOW_HEIGHT as f32,
                ..Default::default()
            })
            .add_resource_path(resource_dir)
            .build()
            .expect("Failed to create a Context");

    let state = &mut MainState::new(ctx).unwrap();

    if let Err(e) = event::run(ctx, events_loop, state) {
        println!("Error encountered: {}", e);
    } else {
        println!("Game exited cleanly.");
    }
}

fn rds_to_rgba_image(rds: &ReactionDiffusionSystem, color_gradient: &ColorGradient) -> RgbaImage {
    ImageBuffer::from_fn(rds.width as u32, rds.height as u32, |x, y| {
        let u = rds.get(&ChemicalSpecies::U, x as isize, y as isize);
        let v = rds.get(&ChemicalSpecies::V, x as isize, y as isize);
        let value = 0.5 + 0.5 * (20.0 * v + 10.0 * u).sin();
        let t = (value + 1.0) / 2.0;

        let [r, g, b] = color_gradient.color_at_t(t);

        Rgba([r, g, b, 255])
    })
}
