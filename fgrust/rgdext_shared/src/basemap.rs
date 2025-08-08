use godot::{prelude::*, classes::TileMapLayer};

#[derive(GodotClass)]
#[class(base=Node)]
pub struct BaseMap {
    pub drop_graphics: bool,

    /// Impacts how entities load
    on_server: bool,

    col_array: Option<CollisionArray>,
    
    base: Base<Node>
}

#[godot_api]
impl INode for BaseMap {
    fn init(base: Base<Node>) -> Self {
        Self {
            drop_graphics: false,
            on_server: false,

            col_array: None,
            
            base
        }
    }


    fn ready(&mut self) {
        self.bake_collisions();

        if self.drop_graphics {
            self.drop_graphics();
        }
    }
}

#[godot_api]
impl BaseMap {
    #[func]
    pub fn get_at(&self, x: i32, y: i32) -> bool {
        self.col_array.as_ref().unwrap().get_at(x, y)
    }

    #[func]
    pub fn set_at(&mut self, x: i32, y: i32, to: bool) {
        self.col_array.as_mut().unwrap().set_at(x, y, to);
    }

    /// If the collisions were already extracted, they're set to null here
    fn bake_collisions(&mut self) {
        self.col_array = Some(self.extract_collisions());
    }

    /// Also drops the collision tilemap.
    pub fn extract_collisions(&mut self) -> CollisionArray {
        // CMap is the expected name of the collision tilemap node
        let mut tmap_c: Gd<TileMapLayer> = match self.base().try_get_node_as::<TileMapLayer>("CMap") {
            Some(map) => map,
            None => return CollisionArray::new(),
        };

        let rect: Rect2i = tmap_c.get_used_rect();

        let mut col_array = CollisionArray::from_used_rect(&rect);

        let cells = tmap_c.get_used_cells();
        // let col_array = self.col_array.as_mut().unwrap();
        for cell in cells.iter_shared() {
            col_array.set_at(cell.x, cell.y, true);
        }

        tmap_c.queue_free();

        col_array
    }

    // Drops all child nodes except for one named 'Entities'
    fn drop_graphics(&mut self) {
        for mut child in self.base().get_children().iter_shared() {
            if child.get_name() != "Entities".into() {
                child.queue_free();
            }
        }
        // Since the graphics get dropped, must be on a server
        self.on_server = true;
    }
}

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

    pub fn new_with_dimensions(other: &Self) -> Self {
        Self {
            map: Vec::with_capacity(other.map.len()),
            topleftx: other.topleftx,
            toplefty: other.toplefty,
            width: other.width,
            height: other.height,
            mapsize: other.mapsize,
        }
    }

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

    pub fn set_from_used_rect(&mut self, rect: &Rect2i) {
        self.topleftx = rect.position.x;
        self.toplefty = rect.position.y;
        self.width = rect.size.x + 1;
        self.height = rect.size.y + 1;
        self.mapsize = self.width * self.height;

        self.map = Vec::with_capacity(self.mapsize as usize);
        for _ in 0..self.mapsize as usize {self.map.push(false);}
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
