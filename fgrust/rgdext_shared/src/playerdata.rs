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
    // pub speed: i32,

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

            placeholder: Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test() {
        let pdata = PlayerData {
            name: "123123".into(),
            location: "map1".into(),
            x: 0,
            y: 0,
            placeholder: None,
        };
        println!("{:?}", pdata.to_bytes());
    }

    #[test]
    fn test_null_function() {
        let null_data = PlayerData::null();
        let bytes = null_data.to_bytes();
        let decoded = PlayerData::from_bytes(&bytes).unwrap();
        
        assert!(decoded.is_null());
    }

    #[test]
    fn test_round_trip_consistency() {
        let test_cases = vec![
            PlayerData {
                name: "Player1".to_string(),
                location: "map1".to_string(),
                x: 10,
                y: 20,
                placeholder: Some(100),
            },
            PlayerData {
                name: "Player2".to_string(),
                location: "map2".to_string(),
                x: -10,
                y: -20,
                placeholder: None,
            },
            PlayerData {
                name: "Player3".to_string(),
                location: "map3".to_string(),
                x: 0,
                y: 0,
                placeholder: Some(0),
            },
        ];

        for original in test_cases {
            let bytes = original.to_bytes();
            let decoded = PlayerData::from_bytes(&bytes).unwrap();
            
            assert_eq!(original.name, decoded.name);
            assert_eq!(original.location, decoded.location);
            assert_eq!(original.x, decoded.x);
            assert_eq!(original.y, decoded.y);
            assert_eq!(original.placeholder, decoded.placeholder);
        }
    }
}
