use godot::{classes::TileMapLayer, prelude::*};
use bitcode::{Encode, Decode};

pub mod spatialhash;

#[derive(GodotClass)]
#[class(tool, base=Node2D)]
pub struct BaseMap {
    col_array: CollisionArray,
    
    base: Base<Node2D>
}

#[godot_api]
impl INode2D for BaseMap {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            col_array: CollisionArray::new(),
            
            base
        }
    }


    fn ready(&mut self) {
        if !godot::classes::Engine::singleton().is_editor_hint() {
            self.col_array = self.extract_collisions(true);
            self.base_mut().set_z_index(-1);
        }
    }
}

#[godot_api]
impl BaseMap {
    #[func]
    fn get_collision_bytes(&mut self) -> PackedByteArray {
        let col_array = self.extract_collisions(false);

        PackedByteArray::from(col_array.to_bytes())
    }

    #[func]
    pub fn get_at(&self, x: i32, y: i32) -> bool {
        self.col_array.get_at(x, y)
    }

    #[func]
    pub fn set_at(&mut self, x: i32, y: i32, to: bool) {
        self.col_array.set_at(x, y, to);
    }

    /// Also drops the collision tilemap.
    pub fn extract_collisions(&mut self, drop_node: bool) -> CollisionArray {
        // CMap is the expected name of the collision tilemap node
        let mut collision_tilemap: Gd<TileMapLayer> = match self.base().try_get_node_as::<TileMapLayer>("CMap") {
            Some(map) => map,
            None => return CollisionArray::new(),
        };

        let rect: Rect2i = collision_tilemap.get_used_rect();

        let mut col_array = CollisionArray::from_used_rect(&rect);

        let cells = collision_tilemap.get_used_cells();
        for cell in cells.iter_shared() {
            col_array.set_at(cell.x, cell.y, true);
        }

        if drop_node {
            collision_tilemap.queue_free();
        }

        col_array
    }
}

#[derive(Encode, Decode)]
pub struct CollisionArray {
    map: Vec<bool>,
    topleftx: i32,
    toplefty: i32,
    width: i32,
    height: i32,
    mapsize: i32,
}

impl CollisionArray {
    pub fn new() -> Self {
        Self {
            map: Vec::new(),
            topleftx: 0,
            toplefty: 0,
            width: 0,
            height: 0,
            mapsize: 0,
        }
    }

    // pub fn new_with_dimensions(other: &Self) -> Self {
    //     Self {
    //         map: Vec::with_capacity(other.map.len()),
    //         topleftx: other.topleftx,
    //         toplefty: other.toplefty,
    //         width: other.width,
    //         height: other.height,
    //         mapsize: other.mapsize,
    //     }
    // }

    pub fn from_used_rect(rect: &Rect2i) -> Self {
        let topleftx = rect.position.x;
        let toplefty = rect.position.y;
        let width = rect.size.x + 1;
        let height = rect.size.y + 1;
        let mapsize = width * height;

        let mut map = Vec::with_capacity(mapsize as usize);
        for _ in 0..mapsize as usize {map.push(false);}

        Self {
            map,
            topleftx,
            toplefty,
            width,
            height,
            mapsize,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<CollisionArray, bitcode::Error> {
        bitcode::decode(bytes)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bitcode::encode(self)
    }

    pub fn get_default_spatialhash<I: Eq, T>(&self) -> spatialhash::SpatialHash<I ,T> {
        let topleft = (self.topleftx, self.toplefty);
        let bottomright = (topleft.0 + self.width - 1, topleft.1 + self.height -1);

        spatialhash::SpatialHash::new(spatialhash::GRID_SIZE, topleft, bottomright, spatialhash::CHECK_RADIUS)
    }

    fn to_index(&self, mut x: i32, mut y: i32) -> i32 {
        x -= self.topleftx; y -= self.toplefty;
        return x + y*self.width;
    }

    pub fn get_at(&self, x: i32, y: i32) -> bool {
        let at = self.to_index(x, y);
        if at < 0 || at >= self.mapsize {
            true
        }
        else {self.map[at as usize]}
    }

    pub fn set_at(&mut self, x: i32, y: i32, to: bool) {
        let at = self.to_index(x, y);
        if at < 0 || at >= self.mapsize {
            return;
        }
        else {self.map[at as usize] = to;}
    }
}
