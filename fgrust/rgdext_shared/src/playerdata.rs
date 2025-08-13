use bitcode::{Decode, Encode};
use godot::prelude::*;
// use serde::{Deserialize, Serialize};


#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct PlayerContainer {
    data: PlayerData,
    
    base: Base<RefCounted>
}

// #[godot_api]
// impl IRefCounted for PlayerContainer {
//     fn init(base: Base<RefCounted>) -> Self {
//         PlayerContainer {
//             data: PlayerData::default(),

//             base
//         }
//     }
// }

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
    fn from_name(name: String) -> Gd<PlayerContainer> {
        let mut data = PlayerData::default();
        data.name = name;
        Gd::from_init_fn(|base| {
            PlayerContainer {
                data,
                base
            }
        })
    }

    #[func]
    fn null() -> Gd<PlayerContainer> {
        Gd::from_init_fn(|base| {
            PlayerContainer {
                data: PlayerData::null(),
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
    /// Allocates a new Godot String, try to call only once if needed
    fn get_location(&self) -> GString {
        self.data.location.clone().into()
    }

    #[func]
    fn is_null(&self) -> bool {
        self.data.is_null()
    }
}

#[derive(Decode, Encode, Clone)]
pub struct PlayerData {
    pub name: String,
    pub location: String,
    pub x: i32,
    pub y: i32,
    pub speed: i32,

    placeholder: Option<i32>,
}

impl PlayerData {
    pub fn from_bytes(b: &[u8]) -> Result<Self, bitcode::Error> {
        bitcode::decode(b)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bitcode::encode(self)
    }

    pub fn to_bytearray(&self) -> PackedByteArray {
        PackedByteArray::from(self.to_bytes())
    }

    pub fn null() -> Self {
        Self {
            name: "".to_string(),
            location: "".to_string(),
            x: 0,
            y: 0,
            speed: 0,

            placeholder: None
        }
    }

    pub fn is_null(&self) -> bool {
        return self.name.is_empty();
    }

    /// Clones name and location
    pub fn get_minimal(&self) -> Self {
        Self {
            name: self.name.clone(),
            location: self.location.clone(),

            x: self.x,
            y: self.y,
            speed: self.speed,

            placeholder: None,
        }
    }
}

impl Default for PlayerData {
    fn default() -> Self {
        Self {
            name: Default::default(),
            location: "map1".to_string(),

            x: 0,
            y: 0,
            speed: 0,

            placeholder: Default::default()
        }
    }
}
