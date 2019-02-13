mod gray_scott_model;

use ggez::{
    event::{self, Keycode, Mod, MouseButton},
    graphics::{self, DrawParam, Point2},
    Context, GameResult,
};
use gray_scott_model::{ChemicalSpecies, ReactionDiffusionSystem};
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba, RgbaImage};
use std::{env, path};

const WINDOW_HEIGHT: usize = 810;
const WINDOW_WIDTH: usize = 1440;

const MODEL_HEIGHT: usize = 180;
const MODEL_WIDTH: usize = 320;

const HEIGHT_RATIO: f32 = WINDOW_HEIGHT as f32 / MODEL_HEIGHT as f32;
const WIDTH_RATIO: f32 = WINDOW_WIDTH as f32 / MODEL_WIDTH as f32;

struct MainState {
    frames: usize,
    reaction_diffusion_system: ReactionDiffusionSystem,
    fast_forward: bool,
}

impl MainState {
    fn new(_ctx: &mut Context) -> GameResult<MainState> {
        let s = MainState {
            frames: 0,
            reaction_diffusion_system: ReactionDiffusionSystem::new(
                MODEL_WIDTH,
                MODEL_HEIGHT,
                0.0545,
                0.062,
                1.0,
                0.5,
            ),
            fast_forward: false,
        };

        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let dt = ggez::timer::get_delta(ctx);
        self.reaction_diffusion_system
            .update(dt.as_millis() as f32 * 0.01);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        if !self.fast_forward {
            let dynamic_image =
                DynamicImage::ImageRgba8(rds_to_rgba_image(&self.reaction_diffusion_system));

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
                .set(&ChemicalSpecies::V, x as isize, y as isize, 1.0)
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

//fn rds_to_rgba_image(rds: &ReactionDiffusionSystem) -> RgbaImage {
//    ImageBuffer::from_fn(rds.width as u32, rds.height as u32, |x, y| {
//        let u = rds.get(&ChemicalSpecies::U, x as isize, y as isize);
//        let v = rds.get(&ChemicalSpecies::V, x as isize, y as isize);
//        let hue = (0.5 + 0.5 * (20.0 * v + 10.0 * u).sin()).to_degrees();
//
//        Rgba(hsvf32_to_rgba8(hue, 1.0, 1.0))
//    })
//}

fn rds_to_rgba_image(rds: &ReactionDiffusionSystem) -> RgbaImage {
    ImageBuffer::from_fn(rds.width as u32, rds.height as u32, |x, y| {
        let u = rds.get(&ChemicalSpecies::U, x as isize, y as isize);
        let v = rds.get(&ChemicalSpecies::V, x as isize, y as isize);
        let value = 0.5 + 0.5 * (20.0 * v + 10.0 * u).sin();
        let value = (value * 255.0) as u8;

        Rgba([value, value, value, 255])
    })
}

//fn hsvf32_to_rgba8(hue: f32, saturation: f32, value: f32) -> [u8; 4] {
//    let i = (hue * 6.0).floor() as isize;
//    let f = hue * 6.0 - i as f32;
//    let p = value * (1.0 - saturation);
//    let q = value * (1.0 - f * saturation);
//    let t = value * (1.0 - (1.0 - f) * saturation);
//
//    let (r, g, b) = match i % 6 {
//        0 => (value, t, p),
//        1 => (q, value, p),
//        2 => (p, value, t),
//        3 => (p, q, value),
//        4 => (t, p, value),
//        _ => (value, p, q),
//    };
//
//    [(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, 255]
//}
