use godot::prelude::*;
use rgdext_shared::playerdata::PlayerData;

// #[derive(GodotClass)]
// #[class(base=Node)]
// pub struct EventQueue {
//     server_events: Vec<ServerEvent>,
//     game_events: Vec<GameEvent>,

//     base: Base<Node>
// }

// #[godot_api]
// impl INode for EventQueue {
//     fn init(base: Base<Node>) -> Self {
//         Self {
//             server_events: Vec::new(),
//             game_events: Vec::new(),
//             base
//         }
//     }

//     fn ready(&mut self) {
//         godot_print!("Queue node ready.\n");
//     }
// }

// #[godot_api]
// impl EventQueue {
//     pub fn push_server(&mut self, e: ServerEvent) {
//         self.server_events.push(e);
//     }

//     pub fn push_game(&mut self, e: GameEvent) {
//         self.game_events.push(e);
//     }

//     pub fn iter_server(&mut self) -> ConsumingIterator<ServerEvent> {
//         ConsumingIterator(&mut self.server_events)
//     }

//     pub fn iter_game(&mut self) -> ConsumingIterator<GameEvent> {
//         ConsumingIterator(&mut self.game_events)
//     }
// }

pub struct ConsumingIterator<'a, T>(&'a mut Vec<T>);

impl<'a, T> Iterator for ConsumingIterator<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

pub enum ServerEvent {
    /// x, y, speed, net_id, target_net_id
    PlayerMoveResponse(i32, i32, i32, i32, i32),
    /// pdata, net_id, target_net_id
    /// 
    /// Uses a reference counted pointer to only store one copy of the data for each event
    UpdatePlayer(std::rc::Rc<Vec<u8>>, i32, i32),
}

pub enum GameEvent {
    /// deltax, deltay, speed, net_id
    PlayerMove(i32, i32, i32, i32),
    /// net_id
    PlayerJoined{net_id: i32, pid: i32},
    /// net_id
    PlayerDisconnected(i32),
    /// name, x, y, net_id
    /// 
    /// Joins player to an instance by the given map name
    PlayerJoinInstance(String, i32, i32, i32),
    /// x, y, net_id
    PlayerInteract(i32, i32, i32),
    NewPlayerData{pid: i32, data: PlayerData},
}

#[derive(GodotClass)]
#[class(base=Node)]
pub struct EQueueInitializer {
    pub shared_queue: EQueue,
    base: Base<Node>
}


#[godot_api]
impl INode for EQueueInitializer {
    fn init(base: Base<Node>) -> EQueueInitializer {
        EQueueInitializer {
            shared_queue: EQueue::default(),
            base
        }
    }
}

#[godot_api]
impl EQueueInitializer {
    #[func]
    fn set_equeue(&mut self) {
        let mut server: Gd<crate::server::Server> = self.base().get_node_as("/root/ServerNode");
        let mut game_manager: Gd<crate::game_manager::GameManager> = self.base().get_node_as("/root/ManagerNode");

        let equeue = EQueue::default();
        server.bind_mut().set_equeue(equeue.clone());
        game_manager.bind_mut().set_equeue(equeue);
    }
}

use std::rc::Rc; use std::cell::RefCell;
#[derive(Clone, Default)]
pub struct EQueue {
    game_events: Rc<RefCell<Vec<GameEvent>>>,
    server_events: Rc<RefCell<Vec<ServerEvent>>>
}

impl EQueue {
    pub fn push_server(&self, e: ServerEvent) {
        self.server_events.borrow_mut().push(e);
    }

    pub fn push_game(&self, e: GameEvent) {
        self.game_events.borrow_mut().push(e);
    }

    pub fn iter_server(&self) -> std::vec::IntoIter<ServerEvent> {
        let vec = std::mem::take(&mut *self.server_events.borrow_mut());
        vec.into_iter()
    }

    pub fn iter_game(&self) -> std::vec::IntoIter<GameEvent> {
        let vec = std::mem::take(&mut *self.game_events.borrow_mut());
        let iter = vec.into_iter();
        iter
    }

    pub fn to_string(&self) -> String {
        format!("<EQueue with pointers {:?} and {:?}>", self.game_events.as_ptr(), self.server_events.as_ptr())
    }
}

pub struct ConsumingRefIterator<'a, T>(std::cell::RefMut<'a, Vec<T>>);

impl<'a, T> Iterator for ConsumingRefIterator<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}
