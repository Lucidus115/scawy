use crate::{astar, idx, prelude::*};
use bevy_ecs::system::Resource;
use rand::{rngs::StdRng, Rng, SeedableRng};

const NUM_ROOMS: u32 = 20;
const SIZE: u32 = 50;

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
        let mut tile_map = vec![1; (SIZE * SIZE) as usize];
        let mut rng = StdRng::seed_from_u64(seed);
        let rooms = build_rooms(&mut rng);
        for i in 0..rooms.len() {
            let room = &rooms[i];
            // Carve
            for y in room.y..room.y + room.height {
                for x in room.x..room.x + room.width {
                    tile_map[idx(x, y, SIZE)] = 0;
                }
            }

            // Connect
            let neighbor =
                (i as i32 + rng.gen_range(-1..2).min(rooms.len() as i32 - 1 - i as i32).max(0)) as usize;
            let neighbor = &rooms[neighbor];
            let path = astar::navigate(
                vec2(room.x as f32, room.y as f32),
                vec2(neighbor.x as f32, neighbor.y as f32),
            );
            path.iter()
                .for_each(|point| tile_map[idx(point.x as u32, point.y as u32, SIZE)] = 0);
        }

        let mut map = Map::new(SIZE, SIZE);

        for y in 0..SIZE {
            for x in 0..SIZE {
                let idx = idx(x, y, SIZE);
                map.set_tile(x, y, tile_map[idx]);
            }
        }

        let spawn = &rooms[0];

        Self {
            map,
            spawn: vec2(
                spawn.x as f32 + (spawn.width as f32 / 2.),
                spawn.y as f32 + (spawn.height as f32 / 2.),
            ),
        }
    }
}

fn build_rooms(rng: &mut StdRng) -> Vec<Room> {
    let mut rooms = Vec::with_capacity(NUM_ROOMS as usize);
    let bounds = Room {
        x: 0,
        y: 0,
        width: SIZE,
        height: SIZE,
    };
    'outer: while rooms.len() < NUM_ROOMS as usize {
        let room = Room {
            x: rng.gen_range(1..SIZE - 10),
            y: rng.gen_range(1..SIZE - 10),
            width: rng.gen_range(2..10),
            height: rng.gen_range(2..10),
        };
        for other in &rooms {
            if room.intersect(other) || !room.intersect(&bounds) {
                continue 'outer;
            }
        }
        rooms.push(room);
    }
    rooms
}

struct Room {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl Room {
    fn intersect(&self, room: &Room) -> bool {
        physics::collide(
            vec2(self.x as f32, self.y as f32),
            vec2(self.width as f32, self.height as f32),
            vec2(room.x as f32, room.y as f32),
            vec2(room.width as f32, room.height as f32),
        )
    }
}
