// use godot::prelude::*;

use std::{cell::RefCell, rc::Rc};

use rgdext_shared::playerdata::PlayerData;


// #[derive(GodotClass)]
// #[class(base=Node)]
pub struct Player {
    pub ticks_since_move: i32,
    speed: i32,
    // These two essentially make a 2-move buffer
    nextmove: Option<(i32, i32, i32)>,
    nextnextmove: Option<(i32, i32, i32)>,

    data: PlayerData,
    /// Counter that increments every time the non-positional data is edited
    /// 
    /// Used by clients to know whether they have the most up to date player data
    data_version: i32,
    // pub location: String,
    // Flag that signals that players should be sent updated player data
    // pub data_just_updated: bool,

    // Ugly flag just used to update players on their positions once
    // pub just_spawned: bool,
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

    pub fn speed(&self) -> i32 {self.speed}

    pub fn pid(&self) -> i32 {self.data.pid}

    pub fn data_version(&self) -> i32 {self.data_version}

    pub fn data(&self) -> &PlayerData {&self.data}

    /// Increments data_version, so you better change something
    pub fn data_mut(&mut self) -> &mut PlayerData {
        self.data_version += 1;
        &mut self.data
    }

    pub fn into_data(self) -> PlayerData {
        self.data
    }

    pub fn new_rc(mut data: PlayerData, server_name: impl ToString) -> Rc<RefCell<Self>> {
        data.server_name = server_name.to_string();
        Rc::new(RefCell::new(
            Self {
                ticks_since_move: 0,
                speed: 0,
                nextmove: None,
                nextnextmove: None,
    
                data,
                data_version: 0,
            }
        ))
    }

    /// Has data_just_updated = true and just_spawned = true
    // pub fn new(data: Rc<RefCell<PlayerData>>) -> Self {
    //     // data.borrow_mut().location = location.to_string();
    //     // let startx = 0;
    //     // let starty = 0;
    //     Self {
    //         ticks_since_move: 0,
    //         speed: 0,
    //         nextmove: None,
    //         nextnextmove: None,
    //         // location: location.to_string(),

    //         data,
    //         data_version: 0,

    //         // just_spawned: true,
    //     }
    // }

    pub fn set_location(&mut self, location: impl ToString) {
        self.data.location = location.to_string();
    }
}

// pub struct PeekMove<'a>(&'a mut Option<(i32, i32, i32)>, &'a mut Option<(i32, i32, i32)>);

// impl<'a> PeekMove<'a> {
//     pub fn eat(&mut self) {
//         *self.0 = self.1.take();
//         // *self.1 = None;
//     }
// }

// impl<'a> std::ops::Deref for PeekMove<'a> {
//     type Target = Option<(i32, i32, i32)>;

//     fn deref(&self) -> &Self::Target {
//         self.0
//     }
// }
