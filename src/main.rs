use ggez::event;
use ggez::graphics;
use ggez::graphics::Point2;
use ggez::{Context, GameResult};
use std::env;
use std::path;

mod gray_scott_model;

use gray_scott_model::ChemicalSpecies;
use gray_scott_model::ReactionDiffusionSystem;

const WINDOW_HEIGHT: usize = 200;
const WINDOW_WIDTH: usize = 200;

struct MainState {
    frames: usize,
    reaction_diffusion_system: ReactionDiffusionSystem,
}

impl MainState {
    fn new(_ctx: &mut Context) -> GameResult<MainState> {
        let s = MainState {
            frames: 0,
            reaction_diffusion_system: ReactionDiffusionSystem::new(
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
                0.055,
                0.062,
                1.0,
                0.5,
            ),
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

        let rgba_buffer = rds_to_frame_buffer(&self.reaction_diffusion_system);
        let image = graphics::Image::from_rgba8(
            ctx,
            self.reaction_diffusion_system.width as u16,
            self.reaction_diffusion_system.height as u16,
            &rgba_buffer,
        )?;
        graphics::draw(ctx, &image, Point2::new(0.0, 0.0), 0.0)?;

        graphics::present(ctx);

        self.frames += 1;
        if (self.frames % 100) == 0 {
            self.frames = 1;
            println!("FPS: {}", ggez::timer::get_fps(ctx));
        }

        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _button: ggez::event::MouseButton,
        x: i32,
        y: i32,
    ) {
        self.reaction_diffusion_system
            .set(&ChemicalSpecies::V, x as isize, y as isize, 1.0)
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

fn rds_to_frame_buffer(rds: &ReactionDiffusionSystem) -> Vec<u8> {
    (0..rds.height)
        .flat_map(|y| (0..rds.width).map(move |x| (x, y)))
        .map(|(x, y)| {
            let u = rds.get(&ChemicalSpecies::U, x as isize, y as isize);
            let v = rds.get(&ChemicalSpecies::V, x as isize, y as isize);

            (0.5 + 0.5 * (20.0 * v + 10.0 * u).sin()).to_degrees()
        })
        .map(|hue| hsvf32_to_rgba8(hue, 1.0, 1.0).to_vec())
        .flatten()
        .collect()
}

fn hsvf32_to_rgba8(hue: f32, saturation: f32, value: f32) -> [u8; 4] {
    let i = (hue * 6.0).floor() as isize;
    let f = hue * 6.0 - i as f32;
    let p = value * (1.0 - saturation);
    let q = value * (1.0 - f * saturation);
    let t = value * (1.0 - (1.0 - f) * saturation);

    let (r, g, b) = match i % 6 {
        0 => (value, t, p),
        1 => (q, value, p),
        2 => (p, value, t),
        3 => (p, q, value),
        4 => (t, p, value),
        _ => (value, p, q),
    };

    [(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, 255]
}
