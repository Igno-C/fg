use std::{cell::RefCell, rc::Rc};

use rgdext_shared::playerdata::PlayerData;

use super::instance::player::Player;


/// After how many seconds does an inactive player's data get removed
const PLAYER_DATA_TIMEOUT: f64 = 45.;
/// Every how many seconds does the player data get saved to the database
const PLAYER_SAVE_TIMEOUT: f64 = 90.;

pub enum PlayerDataEntry {
    RawData{data: PlayerData, age: f64},
    ActivePlayer{player: Rc<RefCell<Player>>, net_id: i32, age: f64}
}

impl PlayerDataEntry {
    pub fn new_active(player: Rc<RefCell<Player>>, net_id: i32) -> Self {
        PlayerDataEntry::ActivePlayer{player, net_id, age: 0.}
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
            PlayerDataEntry::ActivePlayer{player: _, net_id: _, age} => {
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
            PlayerDataEntry::ActivePlayer{player: _, net_id: _, age: _} => true,
            _ => false,
        }
    }

    /// Returns none if player not online on this server
    pub fn get_net_id(&self) -> Option<i32> {
        match self {
            PlayerDataEntry::RawData{data: _, age: _} => None,
            PlayerDataEntry::ActivePlayer{player: _, net_id, age: _} => Some(*net_id),
        }
    }
}

pub enum DataTickResult {
    Idle,
    Save,
    Timeout,
}
