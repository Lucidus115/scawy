use std::{default, fs::File};

use crate::{astar, idx, prelude::*};
use bevy_ecs::system::Resource;
use rand::{rngs::StdRng, Rng, SeedableRng};

const SIZE: u32 = 128;

pub type Tile = u32;

#[derive(Resource)]
pub struct Map {
    tiles: Vec<Tile>,
    width: u32,
    height: u32,
}

impl Map {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            tiles: vec![0; (width * height) as usize],
            width,
            height,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn set_tile(&mut self, x: u32, y: u32, tile: Tile) -> bool {
        let idx = crate::idx(x, y, self.width);
        if idx >= self.tiles.len() {
            warn!("Attempted to set a nonexistent tile value");
            return false;
        }

        self.tiles[idx] = tile;
        true
    }

    pub fn get_tile(&self, x: u32, y: u32) -> Option<&u32> {
        let idx = crate::idx(x, y, self.width);
        self.tiles.get(idx)
    }
}

pub struct MapGenerator {
    pub map: Map,
    pub spawn: Vec2,
}

impl MapGenerator {
    pub fn new(seed: u64) -> Self {
        let map = Map::new(SIZE, SIZE);

        let mut rng = StdRng::seed_from_u64(seed);
        let mut gen = Self {
            map,
            spawn: Vec2::ZERO,
        };
        gen.map.tiles.iter_mut().for_each(|tile| *tile = 1);
        gen.build_rooms(&mut rng);

        gen
    }

    fn build_rooms(&mut self, rng: &mut StdRng) {
        const MIN_TUNNEL_LEN: u32 = 3;
        const MAX_TUNNEL_LEN: u32 = 7;

        let room_defs = RoomDefs::load();
        let possible_starts: Vec<&Room> = room_defs
            .rooms
            .iter()
            .filter(|room| room.prefab.contains('@'))
            .collect();
        let start_room = possible_starts
            .get(rng.gen_range(0..possible_starts.len()))
            .expect("Uh oh. There are no rooms to start in");
        let pos = UVec2::splat(SIZE / 2);

        // Place selected room
        self.place_room(start_room, pos);

        // Grab indicies of possible connectors
        let connectors: Vec<usize> = start_room
            .prefab
            .chars()
            .enumerate()
            .filter(|(_, c)| *c == '+')
            .map(|(idx, _)| idx)
            .collect();

        // Room has no connectors so stop generating
        if connectors.is_empty() {
            return;
        }

        let connector = connectors[rng.gen_range(0..connectors.len())];
        // let connector_pos_a = uvec2(
        //     (connector as u32 % start_room.width) + pos.x,
        //     (connector as u32 / start_room.height) + pos.y
        // );

        //TODO Carve tunnel from connector a to connector b using astar
        let tunnel_len = rng.gen_range(MIN_TUNNEL_LEN..MAX_TUNNEL_LEN);

        //let new_room = ROOM_SMALL;
    }

    fn place_room(&mut self, room: &Room, pos: UVec2) {
        let mut height = 0;
        let mut width = 0;
        let chars: Vec<char> = room
            .prefab
            .chars()
            .filter(|c| {
                if !c.is_whitespace() {
                    width += 1;
                    return true;
                }
                if *c == '\n' {
                    height += 1;
                }
                false
            })
            .collect();

        let mut i = 0;
        for y in 0..height - 1 {
            for x in 0..width / (height - 1) {
                let tile = match chars[i] {
                    '-' => 0,
                    '@' => {
                        self.spawn = vec2((pos.x + x) as f32 + 0.5, (pos.y + y) as f32 + 0.5);
                        0
                    }
                    'D' => 0,
                    _ => 1,
                };

                self.map.tiles[idx(pos.x + x, pos.y + y, SIZE)] = tile;
                i += 1;
            }
        }
    }
}

#[derive(Clone, serde::Deserialize, PartialEq)]
struct Room {
    prefab: String,
}

#[derive(Clone, serde::Deserialize, PartialEq)]
struct RoomDefs {
    rooms: Vec<Room>,
}

impl RoomDefs {
    fn load() -> Self {
        let file = File::open("assets/rooms.ron").expect("Failed to open or could not find file");
        ron::de::from_reader(file).expect("Could not load rooms")
    }
}
