use bitcode::{Decode, Encode};
// use serde::{Serialize, Deserialize};
use godot::prelude::*;

pub mod item;
pub mod skills;

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct PlayerContainer {
    data: PlayerData,
    
    base: Base<RefCounted>
}

#[godot_api]
impl PlayerContainer {
    #[func]
    fn from_bytearray(b: PackedByteArray) -> Gd<PlayerContainer> {
        Gd::from_init_fn(|base| {
            PlayerContainer {
                data: 
                    match PlayerData::from_bytes(b.as_slice()) {
                        Ok(d) => d,
                        Err(message) => {
                            godot_error!("{}", message.to_string());
                            PlayerData::default()
                        }
                    },
                base
            }
        })
    }

    #[func]
    fn from_name(name: String, pid: i32) -> Gd<PlayerContainer> {
        let mut data = PlayerData::default();
        data.name = name;
        data.pid = pid;
        Gd::from_init_fn(|base| {
            PlayerContainer {
                data,
                base
            }
        })
    }

    #[func]
    fn null(pid: i32) -> Gd<PlayerContainer> {
        Gd::from_init_fn(|base| {
            PlayerContainer {
                data: PlayerData::null(pid),
                base
            }
        })
    }

    #[func]
    pub fn to_bytearray(&self) -> PackedByteArray {
        self.data.to_bytearray()
    }

    #[func]
    /// Allocates a new Godot String, try to call only once if needed
    fn get_name(&self) -> GString {
        self.data.name.clone().into()
    }

    #[func]
    fn get_pid(&self) -> i32 {
        self.data.pid
    }

    #[func]
    /// Allocates a new Godot String, try to call only once if needed
    fn get_location(&self) -> GString {
        self.data.location.clone().into()
    }

    #[func]
    fn get_pos(&self) -> Vector2i {
        Vector2i{x: self.data.x, y: self.data.y}
    }

    #[func]
    fn set_pos(&mut self, pos: Vector2i) {
        self.data.x = pos.x;
        self.data.y = pos.y;
    }

    #[func]
    fn is_null(&self) -> bool {
        self.data.is_null()
    }
}

#[derive(Clone, Encode, Decode, Debug)]
pub struct PlayerData {
    pub name: String,
    pub pid: i32,
    pub location: String,
    pub x: i32,
    pub y: i32,

    skills: skills::Skills,
    items: Vec<item::Item>,
}

impl PlayerData {
    pub fn from_bytes(b: &[u8]) -> Result<Self, bitcode::Error> {
        // bitcode::deserialize(b)
        bitcode::decode(b)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // bitcode::serialize(self).unwrap()
        bitcode::encode(self)
    }

    pub fn to_bytearray(&self) -> PackedByteArray {
        PackedByteArray::from(self.to_bytes())
    }

    pub fn null(pid: i32) -> Self {
        Self {
            name: "".to_string(),
            pid,
            location: "".to_string(),
            x: 0,
            y: 0,

            skills: skills::Skills::default(),
            items: Vec::new(),
        }
    }

    pub fn is_null(&self) -> bool {
        return self.location.is_empty();
    }

    /// Clones name and location
    pub fn get_minimal(&self) -> Self {
        Self {
            name: self.name.clone(),
            pid: self.pid,
            location: self.location.clone(),

            x: self.x,
            y: self.y,

            skills: self.skills.clone(),
            items: Vec::new(),
        }
    }
}

// The default PlayerData value is what players are initialized with
// sans fields like name, pid or whatever else may get overwritten
impl Default for PlayerData {
    fn default() -> Self {
        Self {
            name: Default::default(),
            pid: -1,
            location: "map1".to_string(),

            x: 0,
            y: 0,

            skills: skills::Skills::default(),
            items: Vec::new(),
        }
    }
}
