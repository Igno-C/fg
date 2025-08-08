use godot::prelude::*;
use rgdext_shared::playerdata::PlayerData;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct EventQueue {
    server_events: Vec<ServerEvent>,
    game_events: Vec<GameEvent>,

    base: Base<Node>
}

#[godot_api]
impl INode for EventQueue {
    fn init(base: Base<Node>) -> Self {
        Self {
            server_events: Vec::new(),
            game_events: Vec::new(),
            base
        }
    }

    fn ready(&mut self) {
        godot_print!("Queue node ready.\n");
    }
}

#[godot_api]
impl EventQueue {
    pub fn push_server(&mut self, e: ServerEvent) {
        self.server_events.push(e);
    }

    pub fn push_game(&mut self, e: GameEvent) {
        self.game_events.push(e);
    }

    pub fn iter_server(&mut self) -> ConsumingIterator<ServerEvent> {
        ConsumingIterator(&mut self.server_events)
    }

    pub fn iter_game(&mut self) -> ConsumingIterator<GameEvent> {
        ConsumingIterator(&mut self.game_events)
    }
}

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
