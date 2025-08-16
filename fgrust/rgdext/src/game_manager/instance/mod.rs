use std::{cell::RefCell, collections::HashMap, rc::Rc};

use godot::{prelude::*, classes::ResourceLoader};
use crate::eventqueue::{EQueue, ServerEvent, GameEvent};
use rgdext_shared::{basemap::{BaseMap, CollisionArray}, playerdata::PlayerData};
use player::Player; use entity::{Entities, GenericScriptedEntity, ResponseType, ScriptResponse};

pub mod player;
mod entity;

#[derive(GodotClass)]
#[class(no_init, base=Node)]
pub struct Instance {
    pub mapname: String,
    /// Events may only be pushed by instances in process().
    /// 
    /// Or in callbacks from children, like handle_entity_response.
    equeue: EQueue,
    
    map: Option<Gd<BaseMap>>,
    col_array: CollisionArray,

    /// net_id -> Player
    players: HashMap<i32, Player>,
    playercount: i32,
    /// (x, y, net_id)
    deferred_despawns: Vec<(i32, i32, i32)>,

    entities: Entities,
    // Needed because handling responses requires equeue access, so some need to be deferred to _process()
    deferred_responses: Vec<Gd<ScriptResponse>>,

    base: Base<Node>
}

#[godot_api]
impl INode for Instance {
    // fn init(base: Base<Node>) -> Self {
    //     Self {
    //         mapname: "".into(),

    //         equeue: None,

    //         map: None,
    //         col_array: CollisionArray::new(),

    //         players: HashMap::new(),
    //         playercount: 0,
    //         deferred_despawns: Vec::new(),

    //         entities: Entities::default(),
    //         deferred_responses: Vec::new(),

    //         base
    //     }
    // }

    fn ready(&mut self) {
        // let q = self.base().get_node_as::<EventQueue>("/root/QueueNode");
        // self.equeue = Some(q);

        self.load_map();

        godot_print!("Instance {} ready with map {}", self.base().get_name(), self.mapname);
    }

    fn process(&mut self, _delta: f64) {
        while let Some(response) = self.deferred_responses.pop() {
            self.handle_entity_response(response);
        }

        // let mut equeue = self.equeue.as_ref().unwrap().clone();
        // let mut e = equeue.bind_mut();

        // First, despawning players deferred for despawn
        // if !self.deferred_despawns.is_empty() {
        //     let packed_nulldata = std::rc::Rc::new(PlayerData::null().to_bytes());

        //     for (x, y, net_id) in &self.deferred_despawns {
        //         if self.players.remove(net_id).is_some() {
        //             for target_net_id in self.get_adjacent_players(*net_id) {
        //                 self.equeue.push_server(ServerEvent::UpdatePlayer{data: packed_nulldata.clone(), *net_id, *target_net_id};
        //             }
        //         }
        //     }

        //     self.deferred_despawns.clear();
        // }

        let col_array = &self.col_array;

        // Ticking players, starting with movement
        for p in self.players.values_mut() {
            // let mut p = p.borrow_mut();
            p.ticks_since_move += 1;

            if let Some((nextx, nexty, nextspeed)) = *p.peek_next_move() {
                // godot_print!("Trying move to {}, {} with speed {}", nextx, nexty, nextspeed);
                if p.ticks_since_move >= nextspeed {
                    let (x, y, speed) = p.get_pos();

                    if (x - nextx).abs() == 1 || (y - nexty).abs() == 1 {
                        if !col_array.get_at(nextx, nexty) {
                            p.set_pos(nextx, nexty, nextspeed);
                            // *x = nextx; *y = nexty; *speed = nextspeed;

                            // 0 ticks since last move indicates a move just happened
                            p.ticks_since_move = 0;
                        }
                        else {
                            godot_print!("Move into wall by {}: {}, {}, {}", &p.data.borrow().name, nextx, nexty, nextspeed);
                        }
                    }

                    p.eat_next_move();
                }
                else {
                    // godot_print!("Not yet - {}/{}", p.ticks_since_move, nextspeed);
                }
            }
        }


        for (net_id, p) in self.players.iter() {
            // let mut p = p.borrow_mut();
            // Pushing server events for updated player data
            // Players who just spawned are also just updated
            if p.data_just_updated {
                // Updating player of their own data
                // Check here to not send update to player that just disconnected / despawned
                // if !p.data.is_null() {
                //     // Sending full data
                //     let packed_data = std::rc::Rc::new(p.data.to_bytes());
                //     self.equeue.push_server(ServerEvent::UpdatePlayer(packed_data, *net_id, *net_id));
                // }

                // Updating player of other players on initial join
                // if p.just_spawned {
                //     for target_net_id in self.get_adjacent_players(*net_id) {
                //         let other_player = &self.players[target_net_id].borrow_mut();
                //         let packed_mindata = std::rc::Rc::new(other_player.data.get_minimal().to_bytes());
                //         self.equeue.push_server(ServerEvent::UpdatePlayer(packed_mindata, *target_net_id, *net_id));
                //         self.equeue.push_server(ServerEvent::PlayerMoveResponse(other_player.x(), other_player.y(), 0, *target_net_id, *net_id));
                //     }
                // }
                
                // // Updating other players of given player's data, only minimal data
                // let packed_mindata = std::rc::Rc::new(p.data.get_minimal().to_bytes());
                // for target_net_id in self.get_adjacent_players(*net_id) {
                //     self.equeue.push_server(ServerEvent::UpdatePlayer(packed_mindata.clone(), *net_id, *target_net_id));
                // }
            }

            // Pushing server events for every player movement (or initial position after spawning)
            if p.ticks_since_move == 0 {
                self.equeue.push_server(ServerEvent::PlayerMoveResponse(p.x(), p.y(), p.speed(), *net_id, *net_id));
                for target_net_id in self.get_adjacent_players(*net_id) {
                    self.equeue.push_server(ServerEvent::PlayerMoveResponse(p.x(), p.y(), p.speed(), *net_id, *target_net_id));
                }
            }

            // p.data_just_updated = false;
            // p.just_spawned = false;
        }

        // for p in self.players.values_mut() {
        //     p.just_spawned = false;
        // }

        // Player data is nulled when they're despawned
        // This cleans up the list after sending the null packets
        self.players.retain(|_, p| !p.data.borrow().is_null());

        // Resetting any flags
        // for p in self.players.values_mut() {
        //     let mut p = p.borrow_mut();
        //     p.data_just_updated = false;
        //     p.just_spawned = false;
        //     // Reset next move if just moved
        //     if p.ticks_since_move == 0 {p.nextmove = None;}
        // }
    }
}

#[godot_api]
impl Instance {
    pub fn new(mapname: impl ToString, equeue: EQueue) -> Gd<Instance> {
        Gd::from_init_fn(|base| {
            Instance {
                mapname: mapname.to_string(),
                equeue,
                map: None,
                col_array: CollisionArray::new(),
                players: HashMap::new(),
                playercount: 0,
                deferred_despawns: Vec::new(),

                entities: Entities::default(),
                deferred_responses: Vec::new(),

                base
            }
        })
    }

    /// Loads map of a given filename in the maps directory.
    /// 
    /// Assumes the root node of the map scene is a BaseMap.
    fn load_map(&mut self) {
        let m = ResourceLoader::singleton().load(&format!("res://maps/{}.tscn", &self.mapname)).unwrap();
        let map: Gd<PackedScene> = m.cast();
        let mut mapnode = map.instantiate_as::<BaseMap>();

        mapnode.bind_mut().drop_graphics = true;
        mapnode.bind_mut().base_mut().set_name(&self.mapname);
        let col_array = mapnode.bind_mut().extract_collisions();
        self.col_array = col_array;
        self.base_mut().add_child(&mapnode);

        self.map = Some(mapnode);
    }

    pub fn spawn_player(&mut self, data: Rc<RefCell<PlayerData>>, net_id: i32) {
        // let pnode = Player::new(data, x, y);
        // let player = Rc::new(RefCell::new(pnode));
        let player = Player::new(data, &self.mapname);
        // let mut p = player.borrow_mut();
        // p.just_spawned = true;
        // p.data_just_updated = true;
        // p.data.x = x; p.data.y = y; p.nextmove = None;
        // p.set_location(&self.mapname);
        // drop(p);

        self.players.insert(net_id, player);
        self.playercount += 1;
    }

    // Complete removal happens in process()
    // A nulled player first updates other clients, then gets deleted
    pub fn despawn_player(&mut self, net_id: i32) -> Rc<RefCell<PlayerData>> {
        let player = self.players.remove(&net_id).unwrap();
        self.deferred_despawns.push((player.x(), player.y(), net_id));
        player.data
        // if self.players.contains_key(&net_id) {
        //     self.deferred_despawns.push(net_id);
        //     // let mut p = p.borrow_mut();
        //     // // let data = p.data.clone();
        //     // p.data_just_updated = true;
        //     // p.data = PlayerData::null();
        //     // self.playercount -= 1;
        // }
        // else {
        //     godot_error!("Tried to despawn nonexsitent player with net_id {}", net_id);
        // }
    }

    pub fn player_move(&mut self, x: i32, y: i32, speed: i32, net_id: i32) {
        if let Some(player) = self.players.get_mut(&net_id) {
            player.set_next_move(x, y, speed);
        }
        
    }

    // pub fn handle_move(&mut self, x: i32, y: i32, speed: i32, net_id: i32) {
    //     if let Some(pnode) = self.players.get_mut(&net_id) {
    //         pnode.set_next_move(x, y, speed);
    //     }
    // }

    // // Exposed in order to attach signals to it
    #[func]
    fn handle_entity_response(&mut self, response: Gd<ScriptResponse>) {
        match &response.bind().response {
            ResponseType::MovePlayerToMap(inst_name, x, y, net_id) => {
                // let mut equeue = self.equeue.as_ref().unwrap().clone();
                
                self.equeue.push_game(GameEvent::PlayerJoinInstance{mapname: inst_name.to_string(), x: *x, y: *y, net_id: *net_id});
            },
            ResponseType::Null => {},
        }
    }

    pub fn handle_interaction(&mut self, x: i32, y: i32, net_id: i32) {
        if let Some(mut interactable) = self.entities.get_interactable_at(x, y) {
            let response = interactable.bind_mut().on_player_interaction(net_id);

            // self.handle_entity_response(response);

            // Needs to defer here because this is called from the manager and equeue is bound
            self.deferred_responses.push(response);
        }
    }

    /// Gets iterator of net_ids adjacent to given net_id
    /// 
    /// Currently just gives all net_ids minus the passed value
    pub fn get_adjacent_players(&self, adjacent_to: i32) -> impl Iterator<Item = &i32> {
        self.players.keys().filter(move |k| {**k!=adjacent_to})
    }

    #[func]
    pub fn register_interactable(&mut self, entity: Gd<GenericScriptedEntity>) {
        self.entities.register_interactable(entity);
    }
}
