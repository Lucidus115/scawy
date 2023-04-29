use crate::{
    graphics::{Color, Texture},
    idx,
    input::KeyCode,
    map, player,
    prelude::*,
    sound, spawner,
    state::State,
    Context, HEIGHT, WIDTH,
};

use assets_manager::{asset::Wav, BoxedError};
use bevy_ecs::{prelude::*, system::SystemState};
use kira::{
    manager::error::AddSubTrackError,
    sound::static_sound::StaticSoundData,
    track::{effect::reverb::ReverbBuilder, TrackBuilder, TrackHandle, TrackRoutes},
};
use rand::Rng;

const DARKNESS: f32 = 3.5;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
#[system_set(base)]
pub enum CoreSet {
    First,
    Update,
}

impl CoreSet {
    pub fn schedule() -> Schedule {
        let mut schedule = Schedule::new();
        schedule.set_default_base_set(CoreSet::Update);
        schedule.configure_set(CoreSet::First.before(CoreSet::Update));
        schedule
    }
}

#[derive(Resource, Debug)]
pub struct GameData {
    pub generators_required: u32,
    pub notes_taken: u32,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            generators_required: 3,
            notes_taken: 0,
        }
    }
}

#[derive(Resource, Debug)]
pub struct Camera {
    pub pos: Vec2,
    pub dir: Vec2,
    pub plane: Vec2,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            pos: Vec2::ZERO,
            dir: Vec2::NEG_X,
            plane: Vec2::new(0., 0.66),
        }
    }
}

#[derive(Default)]
pub struct Controls {
    pub x: f32,
    pub y: f32,
    pub left: f32,
    pub right: f32,
    pub interact: bool,
    pub attack: bool,
    pub pause: bool,
}

pub struct InGame {
    audio_tracks: Vec<TrackHandle>,
    world: World,
    schedule: Schedule,
    z_buffer: Vec<f32>,
    controls: Controls,
    light_intensity: f32,
    light_duration: u32,
}

impl InGame {
    pub fn new(ctx: &mut Context) -> Self {
        let mut world = World::default();
        world.insert_resource(Camera::default());
        world.insert_resource(sound::SoundQueue::default());
        world.insert_resource(GameData::default());

        let mut schedule = CoreSet::schedule();

        crate::physics::add_to_world(&mut schedule, &mut world);
        crate::ai::add_to_world(&mut schedule, &mut world);
        crate::player::add_to_world(&mut schedule, &mut world);

        setup_map(&mut world);

        let load_assets = || -> Result<(), BoxedError> {
            ctx.assets.load::<Texture>("textures.wall")?;
            ctx.assets.load::<Texture>("textures.floor")?;
            ctx.assets.load::<Texture>("textures.ceil")?;

            ctx.assets.load::<Wav>("sounds.step")?;
            Ok(())
        };

        if load_assets().is_err() {
            warn!("Uh oh! sorry guys. No preloaded assets for you.")
        }

        let mut tracks = Vec::with_capacity(2);
        let mut setup_audio_tracks = || -> Result<(), AddSubTrackError> {
            let ambience = ctx.snd.add_sub_track(TrackBuilder::new())?;
            let sfx = ctx.snd.add_sub_track(TrackBuilder::new())?;

            tracks.push(ambience);
            tracks.push(sfx);

            Ok(())
        };

        if setup_audio_tracks().is_err() {
            warn!("Bruh, audio tracks couldn't be set up properly. There goes the sound.");
        }

        Self {
            audio_tracks: tracks,
            world,
            schedule,
            z_buffer: vec![0.; WIDTH],
            controls: Default::default(),
            light_intensity: 1.,
            light_duration: 0,
        }
    }
}

const SENSITIVITY: f32 = 1. / FPS as f32 * 2.5;
impl State for InGame {
    fn update(&mut self, ctx: &mut Context) {
        self.controls = {
            let x = ctx.input.held(KeyCode::D) as i8 - ctx.input.held(KeyCode::A) as i8;
            let y = ctx.input.held(KeyCode::S) as i8 - ctx.input.held(KeyCode::W) as i8;
            let (left, right) = (
                ctx.input.held(KeyCode::Left) as i8 as f32,
                ctx.input.held(KeyCode::Right) as i8 as f32,
            );
            // let (left, right) = self.input.mouse_diff();

            Controls {
                x: x as f32,
                y: y as f32,
                left,
                right,
                interact: ctx.input.pressed(KeyCode::E),
                attack: ctx.input.pressed(KeyCode::Space),
                ..Default::default()
            }
        };

        #[allow(clippy::type_complexity)]
        let mut system_state: SystemState<(
            EventWriter<player::SendAction>,
            ResMut<Camera>,
            Query<
                (
                    Entity,
                    &mut components::Transform,
                    &mut components::Movement,
                ),
                With<components::Player>,
            >,
        )> = SystemState::new(&mut self.world);

        let (mut writer, mut cam, mut player_query) = system_state.get_mut(&mut self.world);

        // Input
        for (ent, mut trans, mut movement) in player_query.iter_mut() {
            let mut vel = Vec2::ZERO;

            if self.controls.y < 0. {
                vel += vec2(cam.dir.x, cam.dir.y);
            }
            if self.controls.y > 0. {
                vel += vec2(-cam.dir.x, -cam.dir.y);
            }
            if self.controls.x > 0. {
                vel += vec2(cam.dir.y, -cam.dir.x);
            }
            if self.controls.x < 0. {
                vel += vec2(-cam.dir.y, cam.dir.x);
            }

            movement.set_velocity(vel);

            if self.controls.right != 0. {
                let rot = -SENSITIVITY;
                let prev_dir_x = cam.dir.x;
                let prev_plane_x = cam.plane.x;

                cam.dir.x = cam.dir.x * rot.cos() - cam.dir.y * rot.sin();
                cam.dir.y = prev_dir_x * rot.sin() + cam.dir.y * rot.cos();
                cam.plane.x = cam.plane.x * rot.cos() - cam.plane.y * rot.sin();
                cam.plane.y = prev_plane_x * rot.sin() + cam.plane.y * rot.cos();
            }
            if self.controls.left != 0. {
                let rot = SENSITIVITY;
                let prev_dir_x = cam.dir.x;
                let prev_plane_x = cam.plane.x;

                cam.dir.x = cam.dir.x * rot.cos() - cam.dir.y * rot.sin();
                cam.dir.y = prev_dir_x * rot.sin() + cam.dir.y * rot.cos();
                cam.plane.x = cam.plane.x * rot.cos() - cam.plane.y * rot.sin();
                cam.plane.y = prev_plane_x * rot.sin() + cam.plane.y * rot.cos();
            }

            if self.controls.interact {
                writer.send(player::SendAction {
                    entity: ent,
                    action: player::Action::Interact,
                });
            }
            if self.controls.attack {
                writer.send(player::SendAction {
                    entity: ent,
                    action: player::Action::Attack,
                });
            }

            trans.dir = cam.dir;
        }

        self.schedule.run(&mut self.world);

        // Play sounds
        let Some(mut sounds) = self.world.get_resource_mut::<sound::SoundQueue>() else {
            return;
        };

        let mut play_sounds = |track| {
            sounds.0.get_mut(&track).unwrap().retain(|snd| {
                let path = format!("{}/sounds/{}", crate::ASSETS_FOLDER, snd.path);
                let path = path.as_str();

                let Ok(snd) = StaticSoundData::from_file(path, snd.settings) else {
                    warn!("Failed to play sound from path: {path}. Path does not exist");
                    return false;
                };

                snd.settings.track(&self.audio_tracks[track as usize]);

                if ctx.snd.play(snd).is_err() {
                    warn!("An error occured attempting to play sound from path: {path}");
                }

                false
            });
        };

        play_sounds(sound::Track::Sfx);
    }

    #[allow(clippy::type_complexity)]
    fn draw(&mut self, ctx: &mut Context, screen: &mut [u8]) {
        let mut system_state: SystemState<(
            EventReader<player::FlashLight>,
            Res<Camera>,
            Res<map::Map>,
            Query<(&components::Transform, &components::Sprite)>,
        )> = SystemState::new(&mut self.world);

        let (mut event_reader, cam, map, sprite_query) = system_state.get_mut(&mut self.world);

        let floor = ctx.assets.load::<Texture>("textures.floor").unwrap().read();
        let ceil = ctx.assets.load::<Texture>("textures.ceil").unwrap().read();

        let cam_pos_x = cam.pos.x;
        let cam_pos_y = cam.pos.y;

        for event in event_reader.iter() {
            self.light_intensity = event.intesity;
            self.light_duration = event.duration;
        }

        if self.light_duration != 0 {
            self.light_intensity = lerp(self.light_intensity, 1., 1. / self.light_duration as f32);
            self.light_duration -= 1;
        }

        // draw map first

        // raycast
        // followed this tutorial lmao https://lodev.org/cgtutor/raycasting.html

        // floor + ceiling
        for y in 0..HEIGHT {
            let ray_0 = cam.dir - cam.plane;
            let ray_1 = cam.dir + cam.plane;

            let cur_y_pos = y as i32 - HEIGHT as i32 / 2;
            let vertical_pos = 0.5 * HEIGHT as f32;
            let row_dist = vertical_pos / cur_y_pos as f32;

            let step = row_dist * (ray_1 - ray_0) / WIDTH as f32;
            let mut floor_pos = vec2(
                cam_pos_x + row_dist * ray_0.x,
                cam_pos_y + row_dist * ray_0.y,
            );

            for x in 0..WIDTH {
                let cell = floor_pos.as_uvec2();

                let tex_coords = uvec2(
                    (floor.width() as f32 * (floor_pos.x - cell.x as f32)) as u32
                        & (floor.width() as f32 - 1.) as u32,
                    (floor.height() as f32 * (floor_pos.y - cell.y as f32)) as u32
                        & (floor.height() as f32 - 1.) as u32,
                );
                floor_pos += step;

                let idx = idx(tex_coords.x * 4, tex_coords.y * 4, floor.width());
                let dist = (row_dist * DARKNESS / self.light_intensity / 0.5).max(1.) as u8;

                // floor
                {
                    let mut rgba = floor.pixel(idx).slice();

                    rgba.iter_mut().take(3).for_each(|val| {
                        *val = *val / 2 / dist;
                    });

                    let i = x * 4 + y * WIDTH * 4;
                    screen[i..i + 4].copy_from_slice(&rgba);
                }

                // ceiling
                {
                    let mut rgba = ceil.pixel(idx).slice();
                    rgba.iter_mut().take(3).for_each(|val| {
                        *val = *val / 2 / dist;
                    });

                    let i = x * 4 + (HEIGHT - y - 1) * WIDTH * 4;
                    screen[i..i + 4].copy_from_slice(&rgba);
                }
            }
        }

        // wall
        for x in 0..WIDTH {
            let mut tile_pos = cam.pos.as_ivec2();

            // cam coordinates in range of -1 to 1
            let cam_x = 2. * x as f32 / WIDTH as f32 - 1.;
            let ray_x = cam.dir.x + cam.plane.x * cam_x;
            let ray_y = cam.dir.y + cam.plane.y * cam_x;

            let mut side_dist_x;
            let mut side_dist_y;

            let delta_dist_x = (1. / ray_x).abs();
            let delta_dist_y = (1. / ray_y).abs();

            let step_dir_x;
            let step_dir_y;
            let mut side = false;

            // calculate step and side distances
            if ray_x < 0. {
                step_dir_x = -1;
                side_dist_x = (cam_pos_x - tile_pos.x as f32) * delta_dist_x;
            } else {
                step_dir_x = 1;
                side_dist_x = (tile_pos.x as f32 + 1. - cam_pos_x) * delta_dist_x;
            }
            if ray_y < 0. {
                step_dir_y = -1;
                side_dist_y = (cam_pos_y - tile_pos.y as f32) * delta_dist_y;
            } else {
                step_dir_y = 1;
                side_dist_y = (tile_pos.y as f32 + 1. - cam_pos_y) * delta_dist_y;
            }

            let mut hit = false;

            // DDA
            while !hit {
                if side_dist_x < side_dist_y {
                    side_dist_x += delta_dist_x;
                    tile_pos.x += step_dir_x;
                    side = false;
                } else {
                    side_dist_y += delta_dist_y;
                    tile_pos.y += step_dir_y;
                    side = true;
                }

                if tile_pos.x.is_negative() || tile_pos.y.is_negative() {
                    return;
                }
                if let Some(tile) = map.get_tile(tile_pos.x as u32, tile_pos.y as u32) {
                    hit = *tile != map::Tile::Empty;
                }
            }

            let perp_wall_dist = if !side {
                side_dist_x - delta_dist_x
            } else {
                side_dist_y - delta_dist_y
            };
            let wall_height = (HEIGHT as f32 / perp_wall_dist) as i32;
            let draw_start = (-wall_height / 2 + HEIGHT as i32 / 2).max(0);
            let draw_end = (wall_height / 2 + HEIGHT as i32 / 2).min(HEIGHT as i32);

            let dist = (HEIGHT as f32 * DARKNESS / self.light_intensity / wall_height as f32)
                .max(1.) as u8;

            let tile = map
                .get_tile(tile_pos.x as u32, tile_pos.y as u32)
                .expect("tile should have been found already");
            let tex_path = format!("textures.{}", tile_to_texture(*tile));
            let tex = ctx
                .assets
                .load::<Texture>(tex_path.as_str())
                .expect("Failed to find texture from path")
                .read();

            // texture stuff
            let mut wall_x = if !side {
                cam_pos_y + perp_wall_dist * ray_y
            } else {
                cam_pos_x + perp_wall_dist * ray_x
            };
            wall_x -= wall_x.floor();

            // texture x coordinate
            let mut tex_x = (wall_x * tex.width() as f32) as u32;
            if (!side && ray_x > 0.) || (side && ray_y < 0.) {
                tex_x = tex.width() - tex_x - 1;
            }

            let step = tex.height() as f32 / wall_height as f32;
            let mut tex_pos = (draw_start - HEIGHT as i32 / 2 + wall_height / 2) as f32 * step;

            for y in draw_start..draw_end {
                let tex_y = tex_pos as u32 & (tex.height() - 1);
                tex_pos += step;

                // Multiply tex coordinates by 4 to ensure index rgba is in correct order
                let idx = idx(tex_x * 4, tex_y * 4, tex.width());
                let mut rgba = tex.pixel(idx).slice();

                rgba.iter_mut().take(3).for_each(|val| {
                    if side {
                        *val /= 2;
                    }

                    *val /= dist;
                });

                let i = x * 4 + y as usize * WIDTH * 4;
                screen[i..i + 4].copy_from_slice(&rgba);
            }

            // sprites
            self.z_buffer[x] = perp_wall_dist;
        }

        let mut sprites: Vec<(&components::Transform, &components::Sprite)> =
            sprite_query.iter().collect();
        sprites.sort_by(|(trans_a, _), (trans_b, _)| {
            // Sort based on the sprite's distance to camera (far to close)
            let dist_a = trans_a.pos.distance_squared(cam.pos);
            dist_a
                .total_cmp(&trans_b.pos.distance_squared(cam.pos))
                .reverse()
        });

        sprites
                    .iter()
                    .for_each(|(trans, sprite)| {
                        let Ok(tex) = ctx.assets.load::<Texture>(&format!("textures.{}", sprite.texture)) else {
                            warn!("Could not load sprite with texture {}. Path does not exist", sprite.texture);
                            return;
                        };
                        let tex = tex.read();

                        // sprite position relative to camera
                        let pos = trans.pos - cam.pos;
                        let inverse = 1.
                            / (cam.plane.x * cam.dir.y
                                - cam.dir.x * cam.plane.y);
                        let trans_x = inverse * (cam.dir.y * pos.x - cam.dir.x * pos.y);
                        let trans_y =
                            inverse * (-cam.plane.y * pos.x + cam.plane.x * pos.y);

                        // Prevent number from being too low
                        if trans_y.abs() < 0.001 {
                            return;
                        }

                        let move_screen = (-sprite.height / trans_y) as i32;

                        let screen_x = ((WIDTH as f32 / 2.) * (1. + trans_x / trans_y)) as i32;
                        let sprite_height = (HEIGHT as f32 / trans_y * trans.scale.y).abs() as i32;
                        let sprite_width = (HEIGHT as f32 / trans_y * trans.scale.x).abs() as i32;

                        let draw_start = uvec2(
                            (-sprite_width / 2 + screen_x).max(0) as u32,
                            (-sprite_height / 2 + HEIGHT as i32 / 2 + move_screen).max(0) as u32,
                        );
                        let draw_end = uvec2(
                            (((sprite_width / 2) + screen_x).max(0) as u32).min(WIDTH as u32),
                            (sprite_height / 2 + HEIGHT as i32 / 2 + move_screen).min(HEIGHT as i32) as u32);

                        let dist = (trans.pos.distance(cam.pos) * DARKNESS / self.light_intensity / 2.).max(1.) as u8;

                        for x in draw_start.x..draw_end.x {
                            let tex_x = (256 * (x as i32 - (-sprite_width / 2 + screen_x)) as u32 * tex.width() / sprite_width as u32) / 256;
                            if !(trans_y > 0. && x < WIDTH as u32 && trans_y < self.z_buffer[x as usize]) {
                                continue;
                            }
                            for y in draw_start.y..draw_end.y {
                                let d = ((y as i32 - move_screen) * 256 - HEIGHT as i32 * 128 + sprite_height * 128) as u32;
                                let tex_y = (d * tex.height()) / sprite_height as u32 / 256;
                                let idx = idx(tex_x * 4, tex_y * 4, tex.width());
                                let color = tex.pixel(idx);

                                if color.a == 0 {
                                    continue;
                                }

                                let i = x as usize * 4 + y as usize * WIDTH * 4;

                                let mut prev_color = Color::from(&screen[i..i + 4]);
                                prev_color.blend(color);

                                let mut slice = prev_color.slice();

                                slice.iter_mut().take(3).for_each(|val| {
                                    *val /= dist;
                                });
                                screen[i..i + 4].copy_from_slice(&slice);
                            }
                        }
                    });
    }
}

pub fn add_event<T>(world: &mut World, schedule: &mut Schedule)
where
    T: bevy_ecs::event::Event,
{
    world.init_resource::<Events<T>>();
    schedule.add_system(Events::<T>::update_system.in_base_set(CoreSet::First));
}

fn setup_map(world: &mut World) {
    let gen = map::MapGenerator::new(0);

    let mut system_state: SystemState<Commands> = SystemState::new(world);
    let mut cmd = system_state.get(world);

    spawner::spawn_player(
        &mut cmd,
        components::Transform {
            pos: gen.spawn,
            ..Default::default()
        },
    );

    for (ent, spawn) in gen.entities {
        ent.spawn(&mut cmd, spawn.as_vec2() + 0.5);
    }

    let mut monster_spawn = Vec2::NEG_ONE;
    while monster_spawn == Vec2::NEG_ONE {
        let x = rand::thread_rng().gen_range(0..gen.map.width());
        let y = rand::thread_rng().gen_range(0..gen.map.height());

        if gen.map.get_tile(x, y) == Some(&map::Tile::Empty) {
            monster_spawn = vec2(x as f32, y as f32) + 0.5;
        }
    }

    spawner::spawn_monster(
        &mut cmd,
        components::Transform {
            pos: monster_spawn,
            ..Default::default()
        },
    );

    // Since we used commands, we need to apply them to the world
    system_state.apply(world);
    world.insert_resource(gen.map);
}

fn tile_to_texture(tile: map::Tile) -> &'static str {
    use map::Tile;

    match tile {
        Tile::Empty => "",
        Tile::Exit => "exit",
        _ => "wall",
    }
}
