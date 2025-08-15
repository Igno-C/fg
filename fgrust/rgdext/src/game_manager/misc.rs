use std::{cell::RefCell, rc::Rc};

use rgdext_shared::playerdata::PlayerData;



const PLAYER_DATA_TIMEOUT: f64 = 60.;
const PLAYER_SAVE_TIMEOUT: f64 = 60.;

pub struct PlayerDataEntry {
    pub data: Rc<RefCell<PlayerData>>,
    age: f64,
    net_id: Option<i32>,
}

impl PlayerDataEntry {
    pub fn new_with_id(data: Rc<RefCell<PlayerData>>, net_id: i32) -> Self {
        Self {
            data,
            age: 0.,
            net_id: Some(net_id)
        }
    }

    pub fn new(data: Rc<RefCell<PlayerData>>) -> Self {
        Self {
            data,
            age: 0.,
            net_id: None
        }
    }

    /// Returns true on timeout
    pub fn tick(&mut self, delta: f64) -> DataTickResult {
        self.age += delta;
        if self.net_id.is_none() {
            if self.age > PLAYER_DATA_TIMEOUT {
                return DataTickResult::Timeout;
            }
        }
        else {
            if self.age > PLAYER_SAVE_TIMEOUT {
                self.age = 0.;
                return DataTickResult::Save;
            }
        }
        return DataTickResult::Idle;
    }

    pub fn net_id(&self) -> &Option<i32> {
        &self.net_id
    }
}

pub enum DataTickResult {
    Idle,
    Save,
    Timeout,
}
