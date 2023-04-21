use bevy_ecs::system::Resource;
    use log::warn;

    const ROOM_AMOUNT: u32 = 20;

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
        map: Map
    }