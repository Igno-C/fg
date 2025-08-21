use godot::prelude::*;
use rgdext_shared::genericevent::{GenericPlayerEvent, GenericServerResponse};
use std::{rc::Rc, cell::RefCell};


#[derive(GodotClass)]
#[class(base=Node)]
pub struct EQueueInitializer {
    // pub shared_queue: EQueue,
    base: Base<Node>
}


#[godot_api]
impl INode for EQueueInitializer {
    fn init(base: Base<Node>) -> EQueueInitializer {
        EQueueInitializer {
            // shared_queue: EQueue::default(),
            base
        }
    }

    fn ready(&mut self) {
        self.set_equeue();
        self.base_mut().queue_free();
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

pub enum ServerEvent {
    PlayerMoveResponse{x: i32, y: i32, speed: i32, pid: i32, data_version: i32, net_id: i32},
    PlayerDataResponse{data: PackedByteArray, net_id: i32},
    PlayerForceDisconnect{net_id: i32},

    GenericResponse{response: GenericServerResponse, net_id: i32}
}

pub enum GameEvent {
    PlayerMove{x: i32, y: i32, speed: i32, net_id: i32},
    PlayerJoined{net_id: i32, pid: i32},
    PlayerDisconnected{net_id: i32},
    /// Joins player to an instance by the given map name
    PlayerJoinInstance{mapname: String, x: i32, y: i32, net_id: i32},
    PlayerChat{text: GString, target_pid: i32, net_id: i32},
    GenericEvent{event: GenericPlayerEvent, net_id: i32},
    /// net_id of the user requesting, pid of the user whose data is requested
    PDataRequest{pid: i32, net_id: i32}
}
