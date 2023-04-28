use crate::prelude::*;
use assets_manager::AssetCache;
use kira::manager::{backend::cpal::CpalBackend, AudioManager, AudioManagerSettings};
use state::AppState;
use std::time::Instant;
use winit_input_helper::WinitInputHelper;

pub mod astar;
pub mod components;
pub mod physics;

mod ai;
mod graphics;
mod map;
mod math;
mod player;
mod sound;
mod spawner;
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

const WIDTH: usize = 384;
const HEIGHT: usize = 216;
const TITLE: &str = "Scawy";
const DEBUG: bool = cfg!(debug_assertions);

const ASSETS_FOLDER: &str = "assets";
pub mod prelude {
    pub use crate::components;
    pub use crate::math::*;
    pub use crate::physics;
    pub use log::*;

    pub const FPS: u32 = 144;
    pub const PPU: f32 = 16.;
    pub const TIMESTEP: f32 = 1. / FPS as f32;
}

#[derive(Default)]
pub struct Controls {
    pub x: f32,
    pub y: f32,
    pub left: f32,
    pub right: f32,
    pub interact: bool,
    pub pause: bool,
    pub debug_view: bool,
    pub debug_zoom: f32,
}

pub struct Context {
    pub assets: AssetCache,
    pub controls: Controls,
    pub input: WinitInputHelper,
    pub snd: AudioManager,
}

struct Game {
    ctx: Context,
    state: AppState,
    pixels: Pixels,
    frame_count: u32,
}

impl Game {
    fn new(pixels: Pixels) -> Self {
        let assets = AssetCache::new(ASSETS_FOLDER).expect("Path is not a valid directory");
        let snd = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())
            .expect("failed to init Audio Manager");

        let mut ctx = Context {
            snd,
            assets,
            controls: Controls::default(),
            input: WinitInputHelper::new(),
        };
        let default_state = Box::new(state::game::InGame::new(&mut ctx));

        Self {
            ctx,
            state: AppState::new(default_state),
            pixels,
            frame_count: 0,
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

            let debug_zoom = (self.ctx.controls.debug_zoom
                + (self.ctx.input.key_held(VirtualKeyCode::Key2) as i8
                    - self.ctx.input.key_held(VirtualKeyCode::Key1) as i8)
                    as f32)
                .max(1.);

            Controls {
                x: x as f32,
                y: y as f32,
                left,
                right,
                interact: self.ctx.input.key_pressed(VirtualKeyCode::E),
                debug_view: self.ctx.input.key_held(VirtualKeyCode::Tab),
                debug_zoom,
                ..Default::default()
            }
        };

        let active_state = self.state.peek();
        active_state.update(&mut self.ctx);
        self.frame_count += 1;
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

pub fn idx(x: u32, y: u32, width: u32) -> usize {
    (y * width + x) as usize
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

            if !DEBUG {
                return;
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
