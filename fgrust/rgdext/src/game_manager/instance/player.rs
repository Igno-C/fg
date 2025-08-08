// use godot::prelude::*;

use std::{cell::RefCell, rc::Rc};

use rgdext_shared::playerdata::PlayerData;


// #[derive(GodotClass)]
// #[class(base=Node)]
pub struct Player {
    // pub x: i32,
    // pub y: i32,
    // pub speed: i32,

    pub ticks_since_move: i32,
    pub nextmove: Option<(i32, i32, i32)>,
    pub nextnextmove: Option<(i32, i32, i32)>,

    pub data: PlayerData,
    // pub location: String,
    /// Flag that signals that players should be sent updated player data
    pub data_just_updated: bool,

    /// Ugly flag just used to update players on their positions once
    pub just_spawned: bool,
}

impl Player {
    pub fn set_next_move(&mut self, x: i32, y: i32, speed: i32) {
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

    pub fn get_pos_mut(&mut self) -> (&mut i32, &mut i32, &mut i32) {
        (&mut self.data.x, &mut self.data.y, &mut self.data.speed)
    }

    pub fn x(&self) -> i32 {self.data.x}

    pub fn y(&self) -> i32 {self.data.y}

    pub fn speed(&self) -> i32 {self.data.speed}

    /// Has data_just_updated = true and just_spawned = true
    pub fn new(mut data: PlayerData, startx: i32, starty: i32, location: impl ToString) -> Self {
        data.location = location.to_string();
        // let startx = 0;
        // let starty = 0;
        Self {
            // x: startx,
            // y: starty,
            // speed: 0,

            ticks_since_move: 0,
            nextmove: None,
            nextnextmove: None,
            // location: location.to_string(),

            data,
            data_just_updated: true,

            just_spawned: true,
        }
    }

    pub fn new_rc(data: PlayerData, startx: i32, starty: i32, location: impl ToString) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Player::new(data, startx, starty, location)))
    }

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
