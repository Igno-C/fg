use std::{cell::RefCell, collections::HashMap, rc::Rc};

use godot::{classes::{FileAccess, ResourceLoader}, prelude::*};
use crate::eventqueue::{EQueue, ServerEvent, GameEvent};
use rgdext_shared::{basemap::{spatialhash::SpatialHash, CollisionArray}, genericevent::GenericServerResponse, playerdata::PlayerData};
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
    
    entities_node: Option<Gd<Node>>,
    col_array: CollisionArray,
    /// Uses net_id as identifier
    spatial_hash: SpatialHash<i32, Rc<RefCell<Player>>>,

    /// net_id -> Player
    players: HashMap<i32, Rc<RefCell<Player>>>,
    playercount: i32,
    /// (x, y, net_id)
    deferred_despawns: Vec<(i32, i32, i32)>,

    entities: Entities,
    deferred_responses: Vec<(Gd<GenericScriptedEntity>, Gd<ScriptResponse>)>,

    base: Base<Node>
}

#[godot_api]
impl INode for Instance {
    fn ready(&mut self) {
        self.load_map();

        // Registering registerable entities
        for child in self.entities_node.as_ref().unwrap().get_children().iter_shared() {
            if let Ok(entity) = child.try_cast::<GenericScriptedEntity>() {
                let e = entity.clone();
                let b = entity.bind();

                if b.interactable {
                    self.entities.register_interactable(e.clone());
                }
                if b.walkable {
                    self.entities.register_walkable(e.clone());
                }
                if !b.related_scene.is_empty() {
                    self.entities.register_visible(e);
                }
            }
        }

        godot_print!("Instance {} ready with map {}", self.base().get_name(), self.mapname);
    }

    fn process(&mut self, _delta: f64) {
        for despawn in std::mem::take(&mut self.deferred_despawns) {
            let (x, y, pid) = despawn;
            let bytearray = PlayerData::null(pid).to_bytearray();

            self.spatial_hash.for_each_adjacent((x, y), |adjacent| {
                self.equeue.push_server(ServerEvent::PlayerDataResponse{data: bytearray.clone(), net_id: adjacent.0});
            });
        }

        for (entity, response) in std::mem::take(&mut self.deferred_responses) {
            self.handle_entity_response(entity, response);
        }

        // Sending out packets
        for (_net_id, p) in self.players.iter() {
            let p = p.borrow();
            // Broadcasting movement responses from just moved or spawned player to adjacent players
            // 0 ticks since last move means a move just happened
            if p.ticks_since_move == 0 {
                let (x, y, speed) = p.get_full_pos();
                
                self.spatial_hash.for_each_adjacent((x, y), |adjacent| {
                    self.equeue.push_server(ServerEvent::PlayerMoveResponse{x, y , speed, pid: p.pid(), data_version: p.data_version(), net_id: adjacent.0});
                });
            }

            // Broadcasting positions from adjacent players to just spawned player
            // Getting pdata will be done by the client
            // if p.just_spawned {
            //     self.spatial_hash.for_each_adjacent(p.get_pos(), |adjacent| {
            //         let b = adjacent.1.borrow();

            //         self.equeue.push_server(ServerEvent::PlayerMoveResponse{x: b.x, y: b.y, speed: 0, pid: b.pid, net_id: *net_id});
            //     });

            //     p.just_spawned = false;
            // }
        }

        // Ticking player movement
        let col_array = &self.col_array;
        for (net_id, p) in self.players.iter_mut() {
            let mut p = p.borrow_mut();
            p.ticks_since_move += 1;

            if let Some((nextx, nexty, nextspeed)) = *p.peek_next_move() {
                // godot_print!("Trying move to {}, {} with speed {}", nextx, nexty, nextspeed);
                if p.ticks_since_move >= nextspeed {
                    let (x, y, _speed) = p.get_full_pos();

                    if (x - nextx).abs() == 1 || (y - nexty).abs() == 1 {
                        if !col_array.get_at(nextx, nexty) {
                            p.set_full_pos(nextx, nexty, nextspeed);
                            // Updating player on all players that just entered their spatial hash adjacency
                            let delta = self.spatial_hash.update_pos(*net_id, (x, y), (nextx, nexty));
                            delta.for_each_with(&self.spatial_hash, |(other_net_id, pdata)| {
                                let b = pdata.borrow();
                                self.equeue.push_server(ServerEvent::PlayerMoveResponse{
                                    x: b.x(),
                                    y: b.y(),
                                    speed: 0,
                                    pid: b.pid(),
                                    data_version: b.data_version(),
                                    net_id: *net_id
                                });
                            });
                            delta.for_each_with(self.entities.get_visible_hash(), |(entity_id, entity)| {
                                
                            });

                            if let Some(entity) = self.entities.get_walkable_at(nextx, nexty) {
                                let res = GenericScriptedEntity::on_player_walk(entity.clone(), *net_id);
                                self.deferred_responses.push((entity.clone(), res));
                            }
                        }
                        else {
                            godot_print!("Move into wall attempt by {}: {}, {}, {}", p.data().name, nextx, nexty, nextspeed);
                        }
                    }

                    p.eat_next_move();
                }
            }
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
                entities_node: None,
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
        let entities_resource = ResourceLoader::singleton().load(&format!("res://maps/{}.tscn", &self.mapname)).unwrap();
        let entities_scene: Gd<PackedScene> = entities_resource.cast();
        let entities_node = entities_scene.instantiate().unwrap();

        let col_array_data = FileAccess::get_file_as_bytes(&format!("res://maps/{}.col", &self.mapname));
        godot_print!("{:?}" ,col_array_data);
        let col_array = CollisionArray::from_bytes(col_array_data.as_slice()).unwrap();
        self.col_array = col_array;
        self.spatial_hash = self.col_array.get_default_spatialhash();

        self.base_mut().add_child(&entities_node);
        self.entities_node = Some(entities_node);
    }

    pub fn spawn_player(&mut self, player: Rc<RefCell<Player>>, net_id: i32) {
        let mut p = player.borrow_mut();
        p.set_location(&self.mapname);
        let pos = p.get_pos();
        drop(p);

        self.spatial_hash.insert(net_id, player.clone(), pos);
        self.players.insert(net_id, player);
        self.playercount += 1;

        self.equeue.push_server(
            ServerEvent::GenericResponse{response: GenericServerResponse::LoadMap{mapname: self.mapname.clone()}.to_bytearray(), net_id}
        );

        // Initial update about nearby player on spawn
        self.spatial_hash.for_each_adjacent(pos, |adjacent| {
            let b = adjacent.1.borrow();

            self.equeue.push_server(ServerEvent::PlayerMoveResponse{
                x: b.x(),
                y: b.y(),
                speed: 0,
                pid: b.pid(),
                data_version: b.data_version(),
                net_id
            });
        });
    }

    /// Removes all Player refcounts from the instance
    pub fn despawn_player(&mut self, net_id: i32) -> Rc<RefCell<Player>> {
        let player = self.players.remove(&net_id).unwrap();

        let p = player.borrow();
        self.spatial_hash.remove(net_id, p.get_pos());
        self.deferred_despawns.push((p.x(), p.y(), p.pid()));
        self.playercount -= 1;
        drop(p);
        
        player   
    }

    pub fn player_move(&mut self, x: i32, y: i32, speed: i32, net_id: i32) {
        if let Some(player) = self.players.get_mut(&net_id) {
            player.borrow_mut().insert_next_move(x, y, speed);
        }
    }

    // // Exposed in order to attach signals to it
    #[func]
    fn handle_entity_response(&mut self, entity: Gd<GenericScriptedEntity>, response: Gd<ScriptResponse>) {
        match &response.bind().response {
            ResponseType::MovePlayerToMap{mapname, x, y, net_id} => {
                self.equeue.push_game(GameEvent::PlayerJoinInstance{mapname: mapname.to_string(), x: *x, y: *y, net_id: *net_id});
            },
            ResponseType::MovePlayer{x, y, speed, net_id} => {
                if let Some(player) = self.players.get_mut(net_id) {
                    player.borrow_mut().set_full_pos(*x, *y, *speed);
                }
            },
            ResponseType::MoveSelf{x, y, speed} => {
                
            },
            ResponseType::Null => {},
        }
    }

    pub fn handle_interaction(&mut self, x: i32, y: i32, net_id: i32) {
        if let Some(interactable) = self.entities.get_interactable_at(x, y) {
            let b = interactable.bind_mut();
            let (ex, ey) = (b.pos.x, b.pos.y);
            let dist = x.abs_diff(ex).max(y.abs_diff(ey)) as i32;

            if dist <= b.interactable_distance {
                drop(b);
                let response = GenericScriptedEntity::on_player_interaction(interactable.clone(), net_id);
                // let response = b.on_player_interaction(net_id);
                // drop(b);
                let interactable = interactable.clone();
    
                self.handle_entity_response(interactable, response);
            }
        }
    }

    /// Pushes response with empty dictionary if invalid entity targeted
    pub fn get_entity_data(&self, x: i32, y: i32, entity_id: i32, net_id: i32) {
        let data = self.entities.get_visible_hash().get((x, y), entity_id)
            .map_or(Dictionary::new(), |entity| {entity.bind().get_data()});
        self.equeue.push_server(
            ServerEvent::EntityDataResponse{data, entity_id, net_id}
        );
    }

    pub fn broadcast_chat(&mut self, text: GString, target_pid: i32, net_id: i32) {
        if let Some(player) = self.players.get(&net_id) {
            let from = GString::from(&player.borrow().data().name);

            // Target pid -1 means broadcast to all
            if target_pid == -1 {
                for target_net_id in self.players.keys() {
                    self.equeue.push_server(
                        ServerEvent::PlayerChat{from: from.clone(), text: text.clone(), is_dm: false, net_id: *target_net_id}
                    );
                }
            }
            else {
                self.equeue.push_game(GameEvent::PlayerDm{from, text, target_pid});
            }
        }
    }
}
