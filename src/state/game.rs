use crate::{
    graphics::{Color, Texture},
    prelude::*,
    state::State,
    Context, HEIGHT, WIDTH, map, idx,
};

use assets_manager::BoxedError;
use bevy_ecs::prelude::*;
use glam::{vec2, Vec2};

#[derive(Debug)]
struct Camera {
    pos: Vec2,
    dir: Vec2,
    plane: Vec2,
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

pub struct InGame {
    cam: Camera,
    world: World,
    schedule: Schedule,
    z_buffer: Vec<f32>,
}

impl InGame {
    pub fn new(ctx: &mut Context) -> Self {
        use components::*;
        let mut world = World::default();

        // Spawn player
        world.spawn((
            Transform {
                pos: vec2(14., 5.),
                ..Default::default()
            },
            Movement::with_speed(0.2),
            Player,
        ));

        // Spawn test sprite
        world.spawn((
            Transform {
                pos: vec2(11., 5.),
                ..Default::default()
            },
            Sprite {
                texture: String::from("owo"),
                ..Default::default()
            },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems((
            crate::physics::detect_collision.before(crate::physics::apply_movement),
            crate::physics::apply_movement,
        ));

        setup_map(&mut world);

        let load_assets = || -> Result<(), BoxedError> {
            ctx.assets.load::<Texture>("textures.wall")?;
            ctx.assets.load::<Texture>("textures.floor")?;
            ctx.assets.load::<Texture>("textures.ceil")?;

            Ok(())
        };

        if load_assets().is_err() {
            warn!("Uh oh! sorry guys. No preloaded assets for you.")
        }

        Self {
            world,
            schedule,
            cam: Camera::default(),
            z_buffer: vec![0.; WIDTH],
        }
    }
}

const SENSITIVITY: f32 = 1. / FPS as f32 * 3.;
impl State for InGame {
    fn update(&mut self, ctx: &mut Context) {
        // Input
        let mut query = self
            .world
            .query_filtered::<&mut components::Movement, With<components::Player>>();
        for mut movement in query.iter_mut(&mut self.world) {
            let mut vel = Vec2::ZERO;

            if ctx.controls.y < 0. {
                vel += vec2(self.cam.dir.x, self.cam.dir.y);
            }
            if ctx.controls.y > 0. {
                vel += vec2(-self.cam.dir.x, -self.cam.dir.y);
            }
            if ctx.controls.x > 0. {
                vel += vec2(self.cam.dir.y, -self.cam.dir.x);
            }
            if ctx.controls.x < 0. {
                vel += vec2(-self.cam.dir.y, self.cam.dir.x);
            }

            movement.set_velocity(vel);

            if ctx.controls.right != 0. {
                let rot = -SENSITIVITY;
                let prev_dir_x = self.cam.dir.x;
                let prev_plane_x = self.cam.plane.x;

                self.cam.dir.x = self.cam.dir.x * rot.cos() - self.cam.dir.y * rot.sin();
                self.cam.dir.y = prev_dir_x * rot.sin() + self.cam.dir.y * rot.cos();
                self.cam.plane.x = self.cam.plane.x * rot.cos() - self.cam.plane.y * rot.sin();
                self.cam.plane.y = prev_plane_x * rot.sin() + self.cam.plane.y * rot.cos();
            }
            if ctx.controls.left != 0. {
                let rot = SENSITIVITY;
                let prev_dir_x = self.cam.dir.x;
                let prev_plane_x = self.cam.plane.x;

                self.cam.dir.x = self.cam.dir.x * rot.cos() - self.cam.dir.y * rot.sin();
                self.cam.dir.y = prev_dir_x * rot.sin() + self.cam.dir.y * rot.cos();
                self.cam.plane.x = self.cam.plane.x * rot.cos() - self.cam.plane.y * rot.sin();
                self.cam.plane.y = prev_plane_x * rot.sin() + self.cam.plane.y * rot.cos();
            }
        }

        self.schedule.run(&mut self.world);

        // Camera follow player
        let mut query = self
            .world
            .query_filtered::<&components::Transform, With<components::Player>>();
        let Ok(player_loc) = query.get_single(&self.world) else {
                return;
            };

        self.cam.pos = player_loc.pos;
    }

    fn draw(&mut self, ctx: &mut Context, screen: &mut [u8]) {
        let floor = ctx.assets.load::<Texture>("textures.floor").unwrap().read();
        let ceil = ctx.assets.load::<Texture>("textures.ceil").unwrap().read();

        let cam_pos_x = self.cam.pos.x;
        let cam_pos_y = self.cam.pos.y;

        // draw map first
        self.world.resource_scope(|world, map: Mut<map::Map>| {

            //TODO: DEBUG REMOVE ME
            if ctx.controls.debug_view {
                let player_pos;
                let zoom = 1. / ctx.controls.debug_zoom;

                {
                    let trans = world.query_filtered::<&components::Transform, With<components::Player>>().single(world);
                    player_pos = trans.pos * PPU - vec2(WIDTH as f32 / 2., HEIGHT as f32 / 2.);
                }

                for y in 0..map.height() {
                    for x in 0..map.width() {
                        let tile = map.get_tile(x, y).expect("tilemap should never be empty");

                        let color = match *tile {
                            1 => Color::from_rgb(255, 0, 0),
                            2 => Color::from_rgb(0, 255, 0),
                            3 => Color::from_rgb(0, 0, 255),
                            _ => Color::from_rgb(0, 0, 0),
                        };

                        let pos = vec2((x as f32 * PPU) - player_pos.x, (y as f32 * PPU) - player_pos.y) * zoom;
                        let size = PPU * zoom;

                        if !crate::in_frustum(pos.x, pos.y, size, size) {
                            continue;
                        }

                        crate::draw_rect(screen, &pos, &(pos + size), color)
                    }
                }

                let mut query = world
                    .query::<(&components::Transform, &components::Sprite)>();

                for (trans, sprite) in query.iter(world) {
                    let pos = vec2(
                        (trans.pos.x * PPU - player_pos.x) * zoom,
                        (trans.pos.y * PPU - player_pos.y) * zoom,
                    );

                    if !crate::in_frustum(pos.x, pos.y, PPU * zoom, PPU * zoom) {
                        continue;
                    }
                    crate::draw_rect(
                        screen,
                        &pos,
                        &(pos + (PPU * zoom)),
                        sprite.color,
                    )
                }
                return;
            }
            // raycast
            // followed this tutorial lmao https://lodev.org/cgtutor/raycasting.html

            // floor + ceiling
            for y in 0..HEIGHT {
                let ray_0 = self.cam.dir - self.cam.plane;
                let ray_1 = self.cam.dir + self.cam.plane;

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

                    // floor
                    {
                        let idx = idx(tex_coords.x * 4, tex_coords.y * 4, floor.width());
                        let mut rgba = floor.pixel(idx).slice();
                        rgba.iter_mut().take(3).for_each(|val| *val /= 2);

                        let i = x * 4 + y * WIDTH * 4;
                        screen[i..i + 4].copy_from_slice(&rgba);
                    }

                    // ceiling
                    {
                        let idx = idx(tex_coords.x * 4, tex_coords.y * 4, ceil.width());
                        let mut rgba = ceil.pixel(idx).slice();
                        rgba.iter_mut().take(3).for_each(|val| *val /= 2);

                        let i = x * 4 + (HEIGHT - y - 1) * WIDTH * 4;
                        screen[i..i + 4].copy_from_slice(&rgba);
                    }
                }
            }

            // wall
            for x in 0..WIDTH {
                let mut tile_pos = self.cam.pos.as_ivec2();

                // cam coordinates in range of -1 to 1
                let cam_x = 2. * x as f32 / WIDTH as f32 - 1.;
                let ray_x = self.cam.dir.x + self.cam.plane.x * cam_x;
                let ray_y = self.cam.dir.y + self.cam.plane.y * cam_x;

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
                        hit = *tile > 0;
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

                    if side {
                        rgba.iter_mut().take(3).for_each(|val| *val /= 2);
                    }

                    let i = x * 4 + y as usize * WIDTH * 4;
                    screen[i..i + 4].copy_from_slice(&rgba);
                }

                // sprites
                self.z_buffer[x] = perp_wall_dist;
            }

            let mut query = world.query::<(&components::Transform, &components::Sprite)>();
                query
                    .iter(world)
                    .max_by(|(trans_a, _), (trans_b, _)| {
                        // Sort based on the sprite's distance to camera (far to close)
                        let dist_a = trans_a.pos.distance_squared(self.cam.pos);
                        dist_a.total_cmp(&trans_b.pos.distance_squared(self.cam.pos))
                    })
                    .iter()
                    .for_each(|(trans, sprite)| {
                        let Ok(tex) = ctx.assets.load::<Texture>(&format!("textures.{}", sprite.texture)) else {
                            warn!("Could not load sprite with texture {}. Path does not exist", sprite.texture);
                            return;
                        };
                        let tex = tex.read();

                        // sprite position relative to camera
                        let pos = trans.pos - self.cam.pos;
                        let inverse = 1.
                            / (self.cam.plane.x * self.cam.dir.y
                                - self.cam.dir.x * self.cam.plane.y);
                        let trans_x = inverse * (self.cam.dir.y * pos.x - self.cam.dir.x * pos.y);
                        let trans_y =
                            inverse * (-self.cam.plane.y * pos.x + self.cam.plane.x * pos.y);

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
                                screen[i..i + 4].copy_from_slice(&prev_color.slice());
                            }
                        }
                    });
        });
    }
}

fn setup_map(world: &mut World) {
    // #[rustfmt::skip]
    // let (tiles, width, height) = (
    //     &[
    //         1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    //         1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    //         1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    //         1, 0, 0, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    //         1, 0, 0, 2, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    //         1, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    //         1, 0, 0, 2, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    //         1, 0, 0, 2, 2, 2, 0, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    //         1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    //         1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    //         1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    //         1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    //         1, 0, 0, 0, 0, 0, 0, 0, 3, 0, 3, 3, 3, 3, 3, 3, 3, 3, 0, 0, 0, 0, 0, 1,
    //         1, 0, 0, 0, 0, 0, 0, 0, 3, 0, 3, 0, 0, 0, 3, 3, 3, 3, 0, 0, 0, 0, 0, 1,
    //         1, 0, 0, 0, 0, 0, 0, 0, 3, 0, 3, 3, 3, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1,
    //         1, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 3, 0, 3, 3, 3, 3, 0, 0, 0, 0, 0, 1,
    //         1, 0, 0, 0, 0, 0, 0, 0, 3, 0, 3, 0, 3, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1,
    //         1, 0, 0, 0, 0, 0, 0, 0, 3, 0, 3, 3, 3, 0, 3, 3, 0, 3, 0, 0, 0, 0, 0, 1,
    //         1, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 3, 0, 3, 0, 0, 0, 0, 0, 1,
    //         1, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3, 3, 3, 3, 3, 0, 3, 0, 0, 0, 0, 0, 1,
    //         1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    //         1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    //         1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    //         1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    //     ], 
    //     24, 
    //     24
    // );
    // let mut map = Map::new(width, height);

    // for y in 0..height {
    //     for x in 0..width {
    //         let tile = tiles[idx(x, y, width)];

    //         map.set_tile(x, y, tile);
    //     }
    // }

    //world.insert_resource(map);
}

fn tile_to_texture(tile: map::Tile) -> &'static str {
    match tile {
        1 => "ceil",
        _ => "wall",
    }
}