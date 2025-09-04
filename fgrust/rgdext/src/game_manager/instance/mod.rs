use std::{cell::RefCell, collections::HashMap, rc::Rc};

use godot::{classes::{FileAccess, ResourceLoader}, prelude::*};
use crate::eventqueue::{EQueue, ServerEvent, GameEvent};
use rgdext_shared::{basemap::{spatialhash::SpatialHash, CollisionArray}, genericevent::{GenericPlayerEvent, GenericServerResponse}, playerdata::{skills::Skill, MAX_ITEMS}};
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
    /// (x, y, net_id), used to send out packets about a player's despawning
    deferred_despawns: Vec<(i32, i32, i32)>,
    /// (x, y, entity_id), used to send out packets about an entity's despawning
    deferred_entity_despawns: Vec<(i32, i32, i32)>,

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
                self.entities.register_entity(entity);
            }
        }

        godot_print!("Instance {} ready with map {}", self.base().get_name(), self.mapname);
    }

    fn process(&mut self, _delta: f64) {
        for despawn in std::mem::take(&mut self.deferred_despawns) {
            let (x, y, pid) = despawn;

            self.spatial_hash.for_each_adjacent((x, y), |(net_id, _)| {
                let response = GenericServerResponse::DespawnPlayer{pid};
                self.equeue.push_server(ServerEvent::GenericResponse{response: response.to_bytearray(), net_id: *net_id});
            });
        }
        for edespawn in std::mem::take(&mut self.deferred_entity_despawns) {
            let (x, y, entity_id) = edespawn;

            self.spatial_hash.for_each_adjacent((x, y), |(net_id, _)| {
                self.equeue.push_server(ServerEvent::EntityDataResponse{
                    interactable: false,
                    walkable: false,
                    related_scene: "".into(),
                    data: Dictionary::new(), 
                    entity_id: entity_id,
                    net_id: *net_id
                });
            });

            // self.entities.despawn_entity();
        }

        for (entity, response) in std::mem::take(&mut self.deferred_responses) {
            self.handle_entity_response(entity, response);
        }

        // Sending out packets
        for (net_id, p) in self.players.iter() {
            let mut p = p.borrow_mut();
            // Broadcasting movement responses from just moved or spawned player to adjacent players
            // 0 ticks since last move means a move just happened
            if p.ticks_since_move == 0 {
                let (x, y, speed) = p.get_full_pos();

                self.spatial_hash.for_each_adjacent((x, y), |adjacent| {
                    self.equeue.push_server(
                        ServerEvent::PlayerMoveResponse{x, y, speed, pid: p.pid(), data_version: p.data_version(), net_id: adjacent.0}
                    );
                });
            }

            if p.data_just_updated {
                let (x, y) = p.get_pos();
                let response = GenericServerResponse::DataUpdate{pid: p.pid(), data_version: p.data_version()}.to_bytearray();
                
                self.spatial_hash.for_each_adjacent((x, y), |adjacent| {
                    self.equeue.push_server(
                        ServerEvent::GenericResponse{response: response.clone(), net_id: adjacent.0}
                    );
                });
                
                p.data_just_updated = false;
            }

            if p.private_data_just_updated {
                self.equeue.push_server(
                    ServerEvent::PlayerDataResponse{data: p.data.to_bytearray(), net_id: *net_id}
                );
                p.private_data_just_updated = false;
            }
        }

        // Ticking player movement
        let col_array = &self.col_array;
        for (net_id, p) in self.players.iter() {
            let mut p = p.borrow_mut();
            p.ticks_since_move += 1;
            p.data_just_updated = false;

            if let Some((nextx, nexty, nextspeed)) = *p.peek_next_move() {
                // godot_print!("Trying move to {}, {} with speed {}", nextx, nexty, nextspeed);
                if p.ticks_since_move >= nextspeed {
                    let (x, y, _speed) = p.get_full_pos();

                    if (x - nextx).abs() == 1 || (y - nexty).abs() == 1 {
                        if !col_array.get_at(nextx, nexty) {
                            p.set_full_pos(nextx, nexty, nextspeed);

                            // Updating player on all players that just entered their spatial hash adjacency
                            let delta = self.spatial_hash.update_pos(*net_id, (x, y), (nextx, nexty));
                            delta.for_each_with(&self.spatial_hash, |(_other_net_id, pdata)| {
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
                            // And on all entities that just entered their spatial hash adjacency
                            delta.for_each_with(self.entities.get_visible_hash(), |(entity_id, entity)| {
                                let b = entity.bind();
                                self.equeue.push_server(ServerEvent::EntityMoveResponse{
                                    x: b.pos.x,
                                    y: b.pos.y,
                                    speed: 0,
                                    entity_id: *entity_id,
                                    data_version: b.public_data_version,
                                    net_id: *net_id
                                });
                            });

                            // Handling walkable entity
                            if let Some(entity) = self.entities.get_walkable_at(nextx, nexty) {
                                let res = GenericScriptedEntity::on_player_walk(entity.clone(), *net_id);
                                self.deferred_responses.push((entity.clone(), res));
                            }
                        }
                        else {
                            godot_print!("Move into wall attempt by {}: {}, {}, {}", p.data.name, nextx, nexty, nextspeed);
                        }
                    }

                    p.eat_next_move();
                }
            }
        }

        for entity in self.entities.iter_visibles_mut() {
            let mut e = entity.bind_mut();
            if e.ticks_since_last_move == 0 || e.data_just_updated {
                let pos = e.pos;

                self.spatial_hash.for_each_adjacent((pos.x, pos.y), |(net_id, _p)| {
                    self.equeue.push_server(
                        ServerEvent::EntityMoveResponse{
                            x: pos.x,
                            y: pos.y,
                            speed: e.last_speed,
                            entity_id: e.entity_id,
                            data_version: e.public_data_version,
                            net_id: *net_id
                        }
                    );
                });
            }
            e.ticks_since_last_move += 1;
            e.data_just_updated = false;
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
                deferred_entity_despawns: Vec::new(),

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
        self.entities.set_spatial_hash(self.col_array.get_default_spatialhash());

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
        // And initial update about nearby entities
        self.entities.get_visible_hash().for_each_adjacent(pos, |adjacent| {
            let entity_id = adjacent.0;
            let b = adjacent.1.bind();

            self.equeue.push_server(
                ServerEvent::EntityMoveResponse{x: b.pos.x, y: b.pos.y, speed: 0, entity_id, data_version: b.public_data_version, net_id}
            );
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
    fn handle_entity_response(&mut self, mut entity: Gd<GenericScriptedEntity>, response: Gd<ScriptResponse>) {
        match &response.bind().response {
            ResponseType::MovePlayerToMap{mapname, x, y, net_id} => {
                self.equeue.push_game(GameEvent::PlayerJoinInstance{mapname: mapname.to_string(), x: *x, y: *y, net_id: *net_id});
            },
            ResponseType::MovePlayer{x, y, speed, net_id} => {
                if let Some(player) = self.players.get(net_id) {
                    player.borrow_mut().set_full_pos(*x, *y, *speed);
                }
            },
            ResponseType::MoveSelf{x, y, speed} => {
                let mut b = entity.bind_mut();
                let oldpos = (b.pos.x, b.pos.y);
                let newpos = (*x, *y);
                b.pos = Vector2i::new(*x, *y);
                b.last_speed = *speed;
                b.ticks_since_last_move = 0;
                self.entities.get_visible_hash_mut().update_pos(b.entity_id, oldpos, newpos);
            },
            ResponseType::GiveItem{item, net_id} => {
                if let Some(player) = self.players.get(net_id) {
                    let mut b = player.borrow_mut();
                    if b.data.insert_item(item.bind().to_item()) {
                        b.set_private_change();
                    }
                    else {
                        godot_print!("Player had full inventory while trying to insert item");
                    }
                }
            },
            ResponseType::GiveXp{skill, amount, net_id} => {
                if let Some(player) = self.players.get(net_id) {
                    let skillstr = String::from(skill);
                    if let Some(skill) = Skill::try_from_str(&skillstr) {
                        let mut b = player.borrow_mut();
                        let level_delta = b.data.add_xp(skill, *amount);
                        if level_delta > 0 {
                            b.set_public_change();
                        }
                        else {
                            b.set_private_change();
                        }
                    }
                }
            },
            ResponseType::TakeItem{id_string, amount, net_id} => {
                if let Some(player) = self.players.get(net_id) {
                    let id_str = String::from(id_string);
                    let mut b = player.borrow_mut();
                    
                    if b.data.remove_item(&id_str, *amount) {
                        b.set_private_change();
                    }
                    else {
                        godot_print!("Couldn't take item");
                    }
                }
            },
            ResponseType::ChangeGold{amount, net_id} => {
                if let Some(player) = self.players.get(net_id) {
                    let mut b = player.borrow_mut();
                    
                    b.data.gold += *amount;
                    b.set_private_change();
                }
            },
            ResponseType::DespawnSelf{} => {
                let b = entity.bind_mut();
                let (x, y) = (b.pos.x, b.pos.y);
                let entity_id = b.entity_id;
                drop(b);
                entity.queue_free();
                self.deferred_entity_despawns.push((x, y, entity_id));
                self.entities.remove_entity((x, y), entity_id);
            },
            ResponseType::RegisterEntity{entity} => {self.entities.register_entity(entity.clone());}
            ResponseType::SystemChatMessage{text, net_id} => {
                self.equeue.push_server(ServerEvent::PlayerChat{text: text.clone(), from: "".into(), from_pid: -1, is_dm: false, net_id: *net_id});
            }
            ResponseType::Null => {},
        }
    }

    fn handle_interaction(&mut self, x: i32, y: i32, entity_id: i32, net_id: i32) {
        if let Some(interactable) = self.entities.get_interactable_at(x, y) {
            if let Some(player) = self.players.get(&net_id) {
                let b = interactable.bind();
                let pb = player.borrow();
                if b.entity_id != entity_id {
                    godot_print!("Player {} attempted to interact with invalid entity_id {}", player.borrow().data.name, entity_id);
                    return
                }
                let (px, py) = pb.get_pos();
                drop(b); 
                let dist = x.abs_diff(px).max(y.abs_diff(py)) as i32;
    
                if dist <= 1 {
                    let item = pb.data.equipped_item.as_ref().map(|i| i.to_resource());
                    let response = GenericScriptedEntity::on_player_interaction(interactable.clone(), item, net_id);
                    drop(pb);
                    let interactable = interactable.clone();
        
                    self.handle_entity_response(interactable, response);
                }
                else {
                    self.equeue.push_server(
                        ServerEvent::PlayerChat{text: "Too far!".into(), from: "".into(), from_pid: -1, is_dm: false, net_id}
                    );
                }
            }
        }
    }

    pub fn handle_generic_event(&mut self, event: GenericPlayerEvent, net_id: i32) {
        match event {
            GenericPlayerEvent::Interaction{x, y, entity_id} => {
                self.handle_interaction(x, y, entity_id, net_id);
            },
            GenericPlayerEvent::SwapItems{from, to} => {
                if from < MAX_ITEMS && to < MAX_ITEMS {
                    if let Some(player) = self.players.get(&net_id) {
                        let mut b = player.borrow_mut();
                        b.data.items.swap(from, to);
                        b.set_private_change();
                    }   
                }
            }
            GenericPlayerEvent::EquipItem{from} => {
                if from < MAX_ITEMS {
                    if let Some(player) = self.players.get(&net_id) {
                        let mut b = player.borrow_mut();
                        let d = &mut b.data;
                        std::mem::swap(&mut d.items[from], &mut d.equipped_item);
                        b.set_public_change(); // Because equipped items are public
                    }   
                }
            },
            _ => {

            },
        }
    }

    /// Pushes response with empty dictionary if invalid entity targeted
    pub fn get_entity_data(&self, x: i32, y: i32, entity_id: i32, net_id: i32) {
        if let Some(entity) = self.entities.get_visible_hash().get((x, y), entity_id) {
            let (interactable, walkable, related_scene, data) = entity.bind().get_data();

            self.equeue.push_server(
                ServerEvent::EntityDataResponse{interactable, walkable, related_scene, data, entity_id, net_id}
            );
        }
    }

    pub fn broadcast_chat(&mut self, text: GString, target_pid: i32, net_id: i32) {
        if let Some(player) = self.players.get(&net_id) {
            let b = player.borrow();
            let from = GString::from(&b.data.name);

            // Target pid -1 means broadcast to all
            if target_pid == -1 {
                for target_net_id in self.players.keys() {
                    self.equeue.push_server(
                        ServerEvent::PlayerChat{text: text.clone(), from: from.clone(), from_pid: b.pid(),is_dm: false, net_id: *target_net_id}
                    );
                }
            }
            else {
                if player.borrow().pid() == target_pid {
                    return;
                }
                self.equeue.push_game(
                    GameEvent::PlayerDm{text: text.clone(), from, from_pid: b.pid(), target_pid}
                );
            }
        }
    }

    pub fn get_net_id_playerdata(&self, net_id: i32) -> Option<Rc<RefCell<Player>>> {
        self.players.get(&net_id).map(|p| p.clone())
    }
}
