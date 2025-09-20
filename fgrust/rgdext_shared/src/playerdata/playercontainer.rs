use godot::prelude::*;
use super::{item::ItemResource, skills::Skill, *};

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
        let data = PlayerData::from_name(name, pid);
        Gd::from_init_fn(|base| {
            PlayerContainer {
                data,
                base
            }
        })
    }

    pub fn from_data(data: PlayerData) -> Gd<PlayerContainer> {
        Gd::from_init_fn(|base| {
            PlayerContainer {
                data,
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
    fn get_location(&self) -> GString {
        self.data.location.clone().into()
    }

    #[func]
    fn get_friends(&self) -> Array<i32> {
        Array::from(self.data.friends.as_slice())
    }

    #[func]
    fn get_server_name(&self) -> GString {
        GString::from(&self.data.server_name)
    }

    #[func]
    fn get_pos(&self) -> Vector2i {
        Vector2i{x: self.data.x, y: self.data.y}
    }

    #[func]
    fn get_gold(&self) -> i32 {
        self.data.gold
    }

    #[func]
    fn get_equipped_item(&self) -> Option<Gd<ItemResource>> {
        self.data.equipped_item.as_ref().map(|i| i.to_resource())
    }

    #[func]
    fn get_items(&self) -> Array<Option<Gd<ItemResource>>> {
        let mut array = Array::new();
        for item in &self.data.items {
            array.push(
                &item.as_ref().map(|i| i.to_resource())
            );
        }
        array
    }

    #[func]
    fn get_stat(&self, stat: String) -> i32 {
        if let Some(skill) = skills::Skill::try_from_str(stat.as_str()) {
            self.data.skills[skill] as i32
        }
        else {
            -1
        }
    }

    #[func]
    fn get_stat_progress(&self, stat: String) -> i32 {
        if let Some(skill) = skills::Skill::try_from_str(stat.as_str()) {
            self.data.skill_progress[skill]
        }
        else {
            -1
        }
    }

    #[func]
    fn skill_array() -> Array<GString> {
        let mut arr = Array::new();
        for skill in Skill::skill_strs() {
            arr.push(skill);
        }
        arr
    }
}