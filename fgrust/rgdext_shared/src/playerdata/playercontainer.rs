use godot::prelude::*;
use super::*;

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
    fn get_location(&self) -> GString {
        self.data.location.clone().into()
    }

    #[func]
    fn set_location(&mut self, location: String) {
        self.data.location = location;
    }

    #[func]
    fn set_server_name(&mut self, name: String) {
        self.data.server_name = name;
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
}