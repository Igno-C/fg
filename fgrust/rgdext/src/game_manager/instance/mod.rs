use std::{cell::RefCell, collections::HashMap, rc::Rc};

use godot::{prelude::*, classes::ResourceLoader};
use crate::eventqueue::{EQueue, ServerEvent, GameEvent};
use rgdext_shared::{basemap::{spatialhash::SpatialHash, BaseMap, CollisionArray}, genericevent::GenericServerResponse, playerdata::PlayerData};
use player::Player; use entity::{Entities, GenericScriptedEntity, ResponseType, ScriptResponse};

pub mod player;
mod entity;
// mod spatialhash;

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
    spatial_hash: SpatialHash,

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
        for (net_id, p) in self.players.iter_mut() {
            // let mut p = p.borrow_mut();
            p.ticks_since_move += 1;

            if let Some((nextx, nexty, nextspeed)) = *p.peek_next_move() {
                // godot_print!("Trying move to {}, {} with speed {}", nextx, nexty, nextspeed);
                if p.ticks_since_move >= nextspeed {
                    let (x, y, _speed) = p.get_full_pos();

                    if (x - nextx).abs() == 1 || (y - nexty).abs() == 1 {
                        if !col_array.get_at(nextx, nexty) {
                            p.set_full_pos(nextx, nexty, nextspeed);
                            self.spatial_hash.update_pos(*net_id, (x, y), (nextx, nexty));

                            // 0 ticks since last move indicates a move just happened
                            p.ticks_since_move = 0;
                        }
                        else {
                            godot_print!("Move into wall attempt by {}: {}, {}, {}", &p.data.borrow().name, nextx, nexty, nextspeed);
                        }
                    }

                    p.eat_next_move();
                }
                // else {
                //     // godot_print!("Not yet - {}/{}", p.ticks_since_move, nextspeed);
                // }
            }
        }


        for (net_id, p) in self.players.iter_mut() {
            // let mut p = p.borrow_mut();
            // Pushing server events for updated player data
            // Players who just spawned are also just updated
            // if p.data_just_updated {
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
            // }

            // Broadcasting movement responses from just moved or spawned player to adjacent players
            if p.ticks_since_move == 0 || p.just_spawned {
                let (x, y, speed) = p.get_full_pos();
                
                self.spatial_hash.for_each_adjacent((x, y), |adjacent| {
                    self.equeue.push_server(ServerEvent::PlayerMoveResponse{x, y , speed, pid: p.pid(), net_id: adjacent.0});
                });
                // for target_net_id in self.get_adjacent_players(*net_id) {
                //     self.equeue.push_server(ServerEvent::PlayerMoveResponse{x, y, speed, pid: p.pid(), target_net_id: *target_net_id});
                // }
            }

            // Broadcasting positions from adjacent players to just spawned player
            // Getting pdata will be done by the client
            if p.just_spawned {
                self.spatial_hash.for_each_adjacent(p.get_pos(), |adjacent| {
                    let b = adjacent.1.borrow();

                    self.equeue.push_server(ServerEvent::PlayerMoveResponse{x: b.x, y: b.y, speed: 0, pid: b.pid, net_id: *net_id});
                });

                p.just_spawned = false;
            }

            // p.data_just_updated = false;
        }

        for despawn in std::mem::take(&mut self.deferred_despawns) {
            let (x, y, pid) = despawn;
            let bytearray = PlayerData::null(pid).to_bytearray();

            self.spatial_hash.for_each_adjacent((x, y), |adjacent| {
                self.equeue.push_server(ServerEvent::PlayerDataResponse{data: bytearray.clone(), net_id: adjacent.0});
            });
        }
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
                spatial_hash: SpatialHash::default(),
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

        mapnode.bind_mut().on_server();
        mapnode.set_name(&self.mapname);
        let (col_array, spatial_hash) = mapnode.bind_mut().extract_collisions();
        self.col_array = col_array;
        self.spatial_hash = spatial_hash;
        self.base_mut().add_child(&mapnode);

        self.map = Some(mapnode);
    }

    pub fn spawn_player(&mut self, data: Rc<RefCell<PlayerData>>, net_id: i32) {
        data.borrow_mut().location = self.mapname.clone();

        let player = Player::new(data.clone());

        self.spatial_hash.insert(net_id, data, player.get_pos());
        self.players.insert(net_id, player);
        self.playercount += 1;

        self.equeue.push_server(
            ServerEvent::GenericResponse{response: GenericServerResponse::LoadMap{mapname: self.mapname.clone()}, net_id}
        );
    }

    // Complete removal happens in process()
    // A nulled player first updates other clients, then gets deleted
    pub fn despawn_player(&mut self, net_id: i32) -> Rc<RefCell<PlayerData>> {
        let player = self.players.remove(&net_id).unwrap();
        self.spatial_hash.remove(net_id, player.get_pos());
        self.deferred_despawns.push((player.x(), player.y(), player.pid()));
        self.playercount -= 1;
        player.data
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
