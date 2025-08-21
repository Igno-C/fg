use std::{cell::RefCell, rc::Rc};

use rgdext_shared::playerdata::PlayerData;

use super::instance::player::Player;



const PLAYER_DATA_TIMEOUT: f64 = 60.;
const PLAYER_SAVE_TIMEOUT: f64 = 60.;

pub enum PlayerDataEntry {
    RawData{data: PlayerData, age: f64},
    ActivePlayer{player: Rc<RefCell<Player>>, age: f64}
    // pub data: Rc<RefCell<PlayerData>>,
    // age: f64,
    // net_id: Option<i32>,
}

impl PlayerDataEntry {
    pub fn new_active(player: Rc<RefCell<Player>>) -> Self {
        PlayerDataEntry::ActivePlayer{player, age: 0.}
    }

    pub fn new_inactive(data: PlayerData) -> Self {
        PlayerDataEntry::RawData{data, age: 0.}
    }

    /// Returns true on timeout
    pub fn tick(&mut self, delta: f64) -> DataTickResult {
        match self {
            PlayerDataEntry::RawData{data: _, age} => {
                *age += delta;
                if *age > PLAYER_DATA_TIMEOUT {
                    return DataTickResult::Timeout;
                }
            },
            PlayerDataEntry::ActivePlayer{player: _, age} => {
                *age += delta;
                if *age > PLAYER_SAVE_TIMEOUT {
                    *age = 0.;
                    return DataTickResult::Save;
                }
            },
        }
        return DataTickResult::Idle;
    }

    pub fn is_active(&self) -> bool {
        match self {
            PlayerDataEntry::ActivePlayer{player: _, age: _} => true,
            _ => false,
        }
    }
}

pub enum DataTickResult {
    Idle,
    Save,
    Timeout,
}
