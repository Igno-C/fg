use bitcode::{Decode, Encode};
use godot::prelude::*;
use item::Item;
use skills::Skill;

pub mod item;
pub mod skills;
pub mod playercontainer;


/// Corresponding to an 5x8 grid on the client
pub const MAX_ITEMS: usize = 40;

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
    pub gold: i32,
    pub equipped_item: Option<Item>,
    pub items: [Option<Item>; MAX_ITEMS],
    pub friends: Vec<i32>,
}

impl PlayerData {
    pub fn from_name(name: String, pid: i32) -> Self {
        Self {
            name,
            pid,
            friends: Vec::new(),
            ..Default::default()
        }
    }

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

    pub fn get_item(&self, index: usize) -> Option<&Item> {
        match &self.items.get(index) {
            Some(i) => i.as_ref(),
            None => None,
        }
    }

    /// Returns true if item successfully inserted
    pub fn insert_item(&mut self, item: Item) -> bool {
        for item_slot in self.items.iter_mut() {
            match item_slot {
                Some(existing_item) => {
                    if existing_item.stackable() && existing_item.id_string() == item.id_string() {
                        existing_item.count += item.count;
                        return true;
                    }
                },
                None => {
                    *item_slot = Some(item);
                    return true;
                },
            }
        }
        return false;
    }

    /// Gets all player data except for skill progress, gold and inventory
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
            gold: 0,
            equipped_item: self.equipped_item.clone(),
            items: [const {None}; MAX_ITEMS],
            friends: Vec::new(),
        }
    }

    /// Returns amount of levels gained
    pub fn add_xp(&mut self, skill: Skill, amount: i32) -> i32 {
        if self.skills[skill] >= 100 {return 0;}
        self.skill_progress[skill] += amount;
        let mut level = self.skills[skill] as i32;
        while self.skill_progress[skill] > level * level * 100 {
            level += 1;
        }
        let level_delta = level - self.skills[skill] as i32;
        if level >= 100 {
            self.skills[skill] = 100;
            self.skill_progress[skill] = 0;
            return level_delta;
        }
        self.skills[skill] = level as u8;
        return level_delta;
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
            gold: 0,
            equipped_item: None,
            items: [const {None}; MAX_ITEMS],
            friends: Vec::new(),
        }
    }
}
