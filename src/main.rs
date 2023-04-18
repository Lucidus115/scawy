use crate::prelude::*;
use assets_manager::AssetCache;
use glam::{vec2, Vec2};
use graphics::Texture;
use line_drawing::Bresenham;
use state::AppState;
use std::time::Instant;
use winit_input_helper::WinitInputHelper;

pub mod components;

mod graphics;
mod physics;
mod state;

use game_loop::{
    game_loop,
    winit::{
        dpi::LogicalSize, event::VirtualKeyCode, event_loop::EventLoop, window::WindowBuilder,
    },
};
use pixels::{
    wgpu::{Color, RequestAdapterOptions},
    Error, Pixels, PixelsBuilder, SurfaceTexture,
};

const WIDTH: usize = 256;
const HEIGHT: usize = 144;
const TITLE: &str = "Scawy";

pub mod prelude {
    pub use crate::components;
    pub use glam::*;
    pub use log::*;

    pub const FPS: u32 = 144;
    pub const PPU: f32 = 16.;
}

#[derive(Default)]
pub struct Controls {
    pub x: f32,
    pub y: f32,
    pub left: f32,
    pub right: f32,
    pub action: bool,
    pub interact: bool,
    pub pause: bool,
}

pub struct Context {
    assets: AssetCache,
    controls: Controls,
    input: WinitInputHelper,
}

struct Game {
    ctx: Context,
    state: AppState,
    pixels: Pixels,
}

impl Game {
    fn new(pixels: Pixels) -> Self {
        let assets = AssetCache::new("assets").expect("Path is not a valid directory");

        let mut ctx = Context {
            assets,
            controls: Controls::default(),
            input: WinitInputHelper::new(),
        };
        let default_state = Box::new(state::game::InGame::new(&mut ctx));

        Self {
            ctx,
            state: AppState::new(default_state),
            pixels,
        }
    }

    fn update(&mut self) {
        self.ctx.controls = {
            let x = self.ctx.input.key_held(VirtualKeyCode::D) as i8
                - self.ctx.input.key_held(VirtualKeyCode::A) as i8;
            let y = self.ctx.input.key_held(VirtualKeyCode::S) as i8
                - self.ctx.input.key_held(VirtualKeyCode::W) as i8;
            let (left, right) = (
                self.ctx.input.key_held(VirtualKeyCode::Left) as i8 as f32,
                self.ctx.input.key_held(VirtualKeyCode::Right) as i8 as f32,
            );
            // let (left, right) = self.input.mouse_diff();

            Controls {
                x: x as f32,
                y: y as f32,
                left,
                right,
                ..Default::default()
            }
        };

        let active_state = self.state.peek();
        active_state.update(&mut self.ctx);
    }

    fn draw(&mut self) {
        let screen = self.pixels.frame_mut();

        // Clear screen
        for (i, byte) in screen.iter_mut().enumerate() {
            *byte = if i % 4 == 3 { 255 } else { 8 };
        }

        let active_state = self.state.peek();
        active_state.draw(&mut self.ctx, screen);
    }
}

pub fn draw_line(screen: &mut [u8], p1: &Vec2, p2: &Vec2, color: [u8; 4]) {
    let p1 = (p1.x as i64, p1.y as i64);
    let p2 = (p2.x as i64, p2.y as i64);

    for (x, y) in Bresenham::new(p1, p2) {
        // Don't render if outside of rendering view
        if !in_frustum(x as f32, y as f32, 1., 1.) {
            continue;
        }

        let x = std::cmp::min(x as usize, WIDTH - 1);
        let y = std::cmp::min(y as usize, HEIGHT - 1);

        let i = x * 4 + y * WIDTH * 4;

        screen[i..i + 4].copy_from_slice(&color);
    }
}

pub fn draw_rect(screen: &mut [u8], p1: &Vec2, p2: &Vec2, color: [u8; 4]) {
    let p2 = vec2(p2.x - 1., p2.y - 1.);
    let p3 = vec2(p1.x, p2.y);
    let p4 = vec2(p2.x, p1.y);

    draw_line(screen, p1, &p3, color);
    draw_line(screen, &p3, &p2, color);
    draw_line(screen, &p2, &p4, color);
    draw_line(screen, &p4, p1, color);
}

pub fn in_frustum(x: f32, y: f32, width: f32, height: f32) -> bool {
    physics::collide(
        vec2(x, y),
        vec2(width, height),
        Vec2::ZERO,
        vec2(WIDTH as f32, HEIGHT as f32),
    )
}

fn main() -> Result<(), Error> {
    env_logger::Builder::new()
        .filter(None, LevelFilter::Warn)
        .init();

    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            //.with_fullscreen(Some(game_loop::winit::window::Fullscreen::Borderless(None)))
            .with_title(TITLE)
            .with_inner_size(LogicalSize::new(size.width * 4., size.height * 4.))
            .with_min_inner_size(size)
            .build(&event_loop)
            .expect("bruh, window could not be built")
    };

    let win_size = window.inner_size();
    let surface_texture = SurfaceTexture::new(win_size.width, win_size.height, &window);

    let pixels = PixelsBuilder::new(WIDTH as u32, HEIGHT as u32, surface_texture)
        .request_adapter_options(RequestAdapterOptions {
            power_preference: pixels::wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        })
        .clear_color(Color {
            r: 0.0025,
            g: 0.0025,
            b: 0.0025,
            a: 1.,
        })
        .present_mode(pixels::wgpu::PresentMode::AutoNoVsync)
        .build()?;

    let game = Game::new(pixels);

    let mut frames_drawn = 0;
    let mut start = Instant::now();

    game_loop(
        event_loop,
        window,
        game,
        FPS,
        0.1,
        move |g| {
            g.game.update();
        },
        move |g| {
            g.game.draw();

            if let Err(err) = g.game.pixels.render() {
                error!("bruh, rendering failed: {err}");
                g.exit()
            }

            if start.elapsed().as_secs() >= 1 {
                let fps = frames_drawn as f64 / start.elapsed().as_millis() as f64 * 1000.0;
                g.window
                    .set_title(format!("{} - FPS: {:.0}", TITLE, fps).as_str());

                start = Instant::now();
                frames_drawn = 0;
            }
            frames_drawn += 1;
        },
        |g, event| {
            if !g.game.ctx.input.update(event) {
                return;
            }

            if g.game.ctx.input.key_pressed(VirtualKeyCode::Escape)
                || g.game.ctx.input.close_requested()
                || g.game.ctx.input.destroyed()
            {
                g.exit();
                return;
            }

            if let Some(size) = g.game.ctx.input.window_resized() {
                if let Err(err) = g.game.pixels.resize_surface(size.width, size.height) {
                    error!("uh oh! window resize failed: {err}");
                    g.exit();
                }
            }
        },
    )
}
