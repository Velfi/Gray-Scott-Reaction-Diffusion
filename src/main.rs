mod gradient;
mod gray_scott_model;
mod utils;

use ggez::{
    event::{self, Keycode, Mod, MouseButton},
    graphics::{self, DrawParam, Point2},
    Context, GameResult,
};
use gradient::ColorGradient;
use gray_scott_model::{ChemicalSpecies, ReactionDiffusionSystem};
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba, RgbaImage};
use std::{env, path};

const WINDOW_HEIGHT: usize = 810;
const WINDOW_WIDTH: usize = 1440;

const MODEL_HEIGHT: usize = 180;
const MODEL_WIDTH: usize = 320;

const HEIGHT_RATIO: f32 = WINDOW_HEIGHT as f32 / MODEL_HEIGHT as f32;
const WIDTH_RATIO: f32 = WINDOW_WIDTH as f32 / MODEL_WIDTH as f32;

const MODEL_BRAIN_CORAL: (f32, f32) = (0.0545, 0.062);
const MODEL_MITOSIS: (f32, f32) = (0.0367, 0.0649);
const MODEL_FINGERPRINT: (f32, f32) = (0.0545, 0.062);
const MODEL_U_SKATE_WORLD: (f32, f32) = (0.062, 0.061);
const MODEL_RIPPLES: (f32, f32) = (0.018, 0.051);
const MODEL_UNDULATING: (f32, f32) = (0.026, 0.051);
const MODEL_WORMS: (f32, f32) = (0.078, 0.061);
const MODEL_SOLITON_COLLAPSE: (f32, f32) = (0.022, 0.06);

const CURRENT_MODEL: (f32, f32) = MODEL_SOLITON_COLLAPSE;

struct MainState {
    frames: usize,
    reaction_diffusion_system: ReactionDiffusionSystem,
    fast_forward: bool,
    gradient: ColorGradient,
}

impl MainState {
    fn new(_ctx: &mut Context) -> GameResult<MainState> {
        let mut gradient = ColorGradient::from_colors([0, 0, 0], [255, 255, 255]);
        gradient.add_color_at_t(0.80, [0, 20, 230]);
        gradient.add_color_at_t(0.63, [200, 0, 255]);
        gradient.add_color_at_t(0.60, [255, 0, 0]);
        gradient.add_color_at_t(0.53, [0, 255, 255]);
        gradient.add_color_at_t(0.40, [0, 0, 0]);

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
            gradient,
        };

        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let dt = ggez::timer::get_delta(ctx);
        self.reaction_diffusion_system
            .update(dt.as_millis() as f32);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

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

            graphics::draw_ex(
                ctx,
                &image,
                DrawParam {
                    scale: Point2::new(WIDTH_RATIO, HEIGHT_RATIO),
                    ..Default::default()
                },
            )?;
        }
        graphics::present(ctx);

        self.frames += 1;
        if (self.frames % 100) == 0 {
            self.frames = 1;
            println!("FPS: {}", ggez::timer::get_fps(ctx));
        }

        Ok(())
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        state: ggez::event::MouseState,
        x: i32,
        y: i32,
        _xrel: i32,
        _yrel: i32,
    ) {
        if state.is_mouse_button_pressed(MouseButton::Left) {
            let x = x as f32 / WIDTH_RATIO;
            let y = y as f32 / HEIGHT_RATIO;

            self.reaction_diffusion_system
                .set(&ChemicalSpecies::V, x as isize, y as isize, 0.99)
        }
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: Keycode, _keymod: Mod, _repeat: bool) {
        match keycode {
            Keycode::Escape => ctx.quit().expect("Should never fail"),
            Keycode::F => {
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
    let ctx = &mut ggez::ContextBuilder::new("Reaction Diffusion System", "Zelda Hessler")
        .window_mode(ggez::conf::WindowMode {
            width: WINDOW_WIDTH as u32,
            height: WINDOW_HEIGHT as u32,
            ..Default::default()
        })
        .build()
        .expect("Failed to create a Context");

    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        ctx.filesystem.mount(&path, true);
    }

    let state = &mut MainState::new(ctx).unwrap();
    if let Err(e) = event::run(ctx, state) {
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
