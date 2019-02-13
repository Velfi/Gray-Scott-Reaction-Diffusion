use gfx::gfx_constant_struct_meta;
use gfx::gfx_defines;
use gfx::gfx_impl_struct_meta;
use ggez::conf;
use ggez::event;
use ggez::graphics;
use ggez::graphics::DrawMode;
use ggez::graphics::Point2;
use ggez::{Context, GameResult};
use std::env;
use std::path;

gfx_defines! {
    constant ReactionDiffusion {
        rate: f32 = "u_Rate",
    }
}

struct MainState {
    frames: usize,
    dim: ReactionDiffusion,
    rd_shader: graphics::Shader<ReactionDiffusion>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let dim = ReactionDiffusion { rate: 1.0 };
        let rd_shader = graphics::Shader::new(
            ctx,
            "/basic_150.glslv",
            "/reaction_diffusion_150.glslf",
            dim,
            "ReactionDiffusion",
            None,
        )?;
        let s = MainState {
            frames: 0,
            rd_shader,
            dim,
        };

        println!("{}", graphics::get_renderer_info(ctx)?);
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        graphics::circle(ctx, DrawMode::Fill, Point2::new(100.0, 300.0), 100.0, 2.0)?;

        {
            let _lock = graphics::use_shader(ctx, &self.rd_shader);
            self.rd_shader.send(ctx, self.dim)?;
            graphics::circle(ctx, DrawMode::Fill, Point2::new(400.0, 300.0), 100.0, 2.0)?;
        }

        graphics::present(ctx);

        self.frames += 1;
        if (self.frames % 100) == 0 {
            println!("FPS: {}", ggez::timer::get_fps(ctx));
        }

        Ok(())
    }
}

pub fn main() {
    let c = conf::Conf::new();
    let ctx = &mut Context::load_from_conf("game_template", "zelda", c).unwrap();

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
