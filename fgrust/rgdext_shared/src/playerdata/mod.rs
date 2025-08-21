use bitcode::{Decode, Encode};
use godot::prelude::*;

pub mod item;
pub mod skills;
pub mod playercontainer;


#[derive(Clone, Encode, Decode, Debug)]
pub struct PlayerData {
    pub name: String,
    pub pid: i32,

    pub server_name: String,
    pub location: String,
    pub x: i32,
    pub y: i32,

    pub skills: skills::Skills,
    pub skill_progress: skills::SkillProgress,
    pub items: Vec<item::Item>,
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

            server_name: "".to_string(),
            location: "".to_string(),

            x: 0,
            y: 0,

            skills: skills::Skills::default(),
            skill_progress: skills::SkillProgress::default(),
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

            server_name: self.server_name.clone(),
            location: self.location.clone(),

            x: self.x,
            y: self.y,

            skills: self.skills.clone(),
            skill_progress: skills::SkillProgress::default(),
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

            server_name: "".to_string(),
            location: "map1".to_string(),

            x: 0,
            y: 0,

            skills: skills::Skills::default(),
            skill_progress: skills::SkillProgress::default(),
            items: Vec::new(),
        }
    }
}
