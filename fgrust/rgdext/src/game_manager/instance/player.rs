use std::{cell::RefCell, rc::Rc};

use rgdext_shared::playerdata::PlayerData;


pub struct Player {
    pub ticks_since_move: i32,
    pub data_just_updated: bool,
    pub private_data_just_updated: bool,
    speed: i32,
    // These two essentially make a 2-move buffer
    nextmove: Option<(i32, i32, i32)>,
    nextnextmove: Option<(i32, i32, i32)>,

    pub data: PlayerData,
    /// Counter that increments every time the non-positional data is edited
    /// 
    /// Used by clients to know whether they have the most up to date player data
    data_version: i32,
}

impl Player {
    pub fn insert_next_move(&mut self, x: i32, y: i32, speed: i32) {
        if self.nextmove.is_none() {
            self.nextmove = Some((x, y, speed));
        }
        else {
            self.nextnextmove = Some((x, y, speed));
        }   
    }

    pub fn peek_next_move(&mut self) -> &Option<(i32, i32, i32)> {
        if self.nextmove.is_some() {
            &self.nextmove
        }
        else {
            &self.nextnextmove
        }
    }

    pub fn eat_next_move(&mut self) {
        self.nextmove = self.nextnextmove.take();
    }

    pub fn get_full_pos(&self) -> (i32, i32, i32) {
        let b = &self.data;
        (b.x, b.y, self.speed)
    }

    pub fn get_pos(&self) -> (i32, i32) {
        let b = &self.data;
        (b.x, b.y)
    }

    pub fn set_full_pos(&mut self, x: i32, y: i32, speed: i32) {
        let b = &mut self.data;
        b.x = x; b.y = y; self.speed = speed; self.ticks_since_move = 0;
    }

    pub fn x(&self) -> i32 {self.data.x}

    pub fn y(&self) -> i32 {self.data.y}

    // pub fn speed(&self) -> i32 {self.speed}

    pub fn pid(&self) -> i32 {self.data.pid}

    pub fn data_version(&self) -> i32 {self.data_version}

    // pub fn data(&self) -> &PlayerData {&self.data}

    pub fn set_public_change(&mut self) {
        self.data_version += 1;
        self.data_just_updated = true;
    }

    pub fn set_private_change(&mut self) {
        self.private_data_just_updated = true;
    }

    pub fn into_data(self) -> PlayerData {
        self.data
    }

    pub fn new_rc(data: PlayerData) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(
            Self {
                ticks_since_move: 0,
                speed: 0,
                nextmove: None,
                nextnextmove: None,
    
                data,
                data_version: 0,
                data_just_updated: false,
                private_data_just_updated: false,
            }
        ))
    }

    pub fn set_location(&mut self, location: impl ToString) {
        self.data.location = location.to_string();
    }
}
