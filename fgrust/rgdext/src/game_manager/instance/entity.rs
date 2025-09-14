use std::collections::HashMap;

use godot::prelude::*;
use rgdext_shared::{basemap::spatialhash::SpatialHash, playerdata::{item::ItemResource, playercontainer::PlayerContainer}};

/// Only overwite on_* virtual methods.
/// 
/// If a null response is returned (as by default), no signal will be sent.
#[derive(GodotClass)]
#[class(base=Node)]
pub struct GenericScriptedEntity {
    #[export]
    /// Position of the entity in the world.
    pub pos: Vector2i,
    #[var]
    /// Whether the entity can be interacted with. Should be set in _ready().
    /// 
    /// Interaction response is based on the [method _on_player_interaction] method.
    pub interactable: bool,
    #[var]
    /// If interactable, what's the max Chebyshev distance the entity can be interacted with. Zero means just the position of the entity. Should be set in _ready().
    pub interactable_distance: i32,
    #[var]
    /// Whether the entity should trigger on being walked on. Should be set in _ready().
    /// 
    /// Walked on response is based on the [method _on_player_walk] method.
    pub walkable: bool,
    #[var]
    /// Whether the entity has a client scene that should be shown to the clients. Should be set in _ready().
    /// 
    /// Set to the name of the scene minus .tscn suffix. Leave empty to leave the entity invisible to clients.
    related_scene: GString,

    #[export]
    /// Custom entity data that gets synchronised between client and server.
    /// The client scene may use this to show visual effects, or whatever else the client scene has implemented.
    /// 
    /// Change either using [method set_public_value], or by calling [method increment_data_version]
    /// to update the data version for client synchronization.
    public_data: Dictionary,
    pub public_data_version: i32,
    pub data_just_updated: bool,
    pub last_speed: i32,
    pub ticks_since_last_move: i32,
    pub entity_id: i32,

    base: Base<Node>
}

#[godot_api]
impl INode for GenericScriptedEntity {
    fn init(base: Base<Node>) -> Self {
        Self {
            pos: Vector2i::ZERO,
            interactable: false,
            interactable_distance: 1,
            walkable: false,
            related_scene: "".to_godot(),
            public_data: Dictionary::new(),
            public_data_version: 0,
            data_just_updated: false,
            last_speed: 0,
            ticks_since_last_move: 0,
            entity_id: -1,
            
            base
        }
    }
}

#[godot_api]
impl GenericScriptedEntity {
    #[signal]
    fn entity_response(object: Gd<GenericScriptedEntity>, response: Gd<ScriptResponse>);

    #[func]
    fn emit_response(&mut self, response: Gd<ScriptResponse>) {
        // if response.bind().response.is_null() {
        //     return;
        // }

        let this = self.to_gd();

        self.base_mut().emit_signal("entity_response", vslice![this, response]);
    }

    #[func]
    fn set_public_value(&mut self, key: Variant, value: Variant) {
        self.public_data_version += 1;
        self.data_just_updated = true;
        self.public_data.set(key, value);
    }

    #[func]
    fn increment_data_version(&mut self) {
        self.public_data_version += 1;
        self.data_just_updated = true;
    }

    #[func(gd_self, virtual)]
    pub fn on_player_walk(this: Gd<Self>, player: Gd<PlayerContainer>, net_id: i32) -> Array<Gd<ScriptResponse>> {
        Array::from(&[ScriptResponse::null_response()])
    }

    #[func(gd_self, virtual)]
    pub fn on_player_interaction(this: Gd<Self>, player: Gd<PlayerContainer>, net_id: i32) -> Array<Gd<ScriptResponse>> {
        Array::from(&[ScriptResponse::null_response()])
    }

    #[func]
    fn ticks_since_last_move(&self) -> i32 {
        self.ticks_since_last_move
    }

    pub fn get_data(&self) -> (bool, bool, GString, Dictionary) {
        (self.interactable, self.walkable, self.related_scene.clone(), self.public_data.clone())
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct ScriptResponse {
    pub response: ResponseType,

    base: Base<RefCounted>
}

#[godot_api]
impl ScriptResponse {
    #[func]
    fn move_player_to_map(mapname: GString, x: i32, y: i32, net_id: i32) -> Gd<ScriptResponse> {
        let response = ResponseType::MovePlayerToMap{mapname, x, y, net_id};

        Gd::from_init_fn(|base| ScriptResponse {response, base})
    }

    #[func]
    fn move_player(x: i32, y: i32, speed: i32, net_id: i32) -> Gd<ScriptResponse> {
        let response = ResponseType::MovePlayer{x, y, speed, net_id};
        
        Gd::from_init_fn(|base| ScriptResponse {response, base})
    }

    #[func]
    fn move_self(x: i32, y: i32, speed: i32) -> Gd<ScriptResponse> {
        let response = ResponseType::MoveSelf{x, y, speed};
        
        Gd::from_init_fn(|base| ScriptResponse {response, base})
    }

    #[func]
    fn give_item(item: Gd<ItemResource>, net_id: i32) -> Gd<ScriptResponse> {
        let response = ResponseType::GiveItem{item, net_id};
        
        Gd::from_init_fn(|base| ScriptResponse {response, base})
    }

    #[func]
    fn take_item(id_string: GString, amount: i32, net_id: i32) -> Gd<ScriptResponse> {
        let response = ResponseType::TakeItem{id_string, amount, net_id};
        
        Gd::from_init_fn(|base| ScriptResponse {response, base})
    }

    #[func]
    fn change_gold(amount: i32, net_id: i32) -> Gd<ScriptResponse> {
        let response = ResponseType::ChangeGold{amount, net_id};
        
        Gd::from_init_fn(|base| ScriptResponse {response, base})
    }

    #[func]
    fn give_xp(skill: GString, amount: i32, net_id: i32) -> Gd<ScriptResponse> {
        let response = ResponseType::GiveXp{skill, amount, net_id};
        
        Gd::from_init_fn(|base| ScriptResponse {response, base})
    }

    #[func]
    fn despawn_self() -> Gd<ScriptResponse> {
        let response = ResponseType::DespawnSelf{};
        
        Gd::from_init_fn(|base| ScriptResponse {response, base})
    }

    #[func]
    fn register_entity(entity: Gd<GenericScriptedEntity>) -> Gd<ScriptResponse> {
        let response = ResponseType::RegisterEntity{entity};
        
        Gd::from_init_fn(|base| ScriptResponse {response, base})
    }

    #[func]
    fn chat_message(text: GString, net_id: i32) -> Gd<ScriptResponse> {
        let response = ResponseType::SystemChatMessage{text, net_id};
        
        Gd::from_init_fn(|base| ScriptResponse {response, base})
    }

    #[func]
    fn null_response() -> Gd<ScriptResponse> {
        Gd::from_init_fn(|base| ScriptResponse {response: ResponseType::Null, base})
    }
}

#[derive(Clone)]
pub enum ResponseType {
    /// map name, x, y, net_id
    MovePlayerToMap{mapname: GString, x: i32, y: i32, net_id: i32},
    MovePlayer{x: i32, y: i32, speed: i32, net_id: i32},
    MoveSelf{x: i32, y: i32, speed: i32},
    GiveItem{item: Gd<ItemResource>, net_id: i32},
    TakeItem{id_string: GString, amount: i32, net_id: i32},
    ChangeGold{amount: i32, net_id: i32},
    GiveXp{skill: GString, amount: i32, net_id: i32},
    DespawnSelf{},
    RegisterEntity{entity: Gd<GenericScriptedEntity>},
    SystemChatMessage{text: GString, net_id: i32},
    Null
}

// impl ResponseType {
//     pub fn is_null(&self) -> bool {
//         match self {
//             ResponseType::Null => {true},
//             _ => {false},
//         }
//     }
// }

// Struct that holds a list of interactable entities
#[derive(Default)]
pub struct Entities {
    // Used to give registered visible entities unique ids
    last_id: i32,

    // interactables: HashMap<(i32, i32), Gd<GenericScriptedEntity>>,
    walkable_hash: HashMap<(i32, i32), Gd<GenericScriptedEntity>>,
    entity_hash: SpatialHash<i32, Gd<GenericScriptedEntity>>,
    entity_list: Vec<Gd<GenericScriptedEntity>>
}

impl Entities {
    pub fn move_entity(&mut self, oldpos: (i32, i32), newpos: (i32, i32), entity_id: i32) {
        self.entity_hash.update_pos(entity_id, oldpos, newpos);
    }

    pub fn move_walkable(&mut self, oldpos: (i32, i32), newpos: (i32, i32)) {
        if let Some(entity) = self.walkable_hash.remove(&oldpos) {
            self.walkable_hash.insert(newpos, entity);
        }
    }

    pub fn remove_entity(&mut self, pos: (i32, i32), entity_id: i32) {
        // self.interactables.remove(&pos);
        // self.walkables.remove(&pos);
        self.entity_hash.remove(entity_id, pos);
        if let Some(index) = self.entity_list.iter().position(|v| v.bind().entity_id == entity_id) {
            self.entity_list.remove(index);
        }
    }

    pub fn set_spatial_hash(&mut self, hash: SpatialHash<i32, Gd<GenericScriptedEntity>>) {
        self.entity_hash = hash;
    }

    pub fn get_at(&mut self, x: i32, y: i32, entity_id: i32) -> Option<&mut Gd<GenericScriptedEntity>> {
        self.entity_hash.get_mut((x, y), entity_id)
    }

    pub fn get_walkable_at(&mut self, x: i32, y: i32) -> Option<&mut Gd<GenericScriptedEntity>> {
        return self.walkable_hash.get_mut(&(x, y));
    }

    pub fn get_hash(&self) -> &SpatialHash<i32, Gd<GenericScriptedEntity>> {
        &self.entity_hash
    }

    pub fn register_entity(&mut self, mut entity: Gd<GenericScriptedEntity>) {
        self.last_id += 1;
        
        let mut e = entity.bind_mut();
        let x = e.pos.x; let y = e.pos.y;
        e.entity_id = self.last_id;
        std::mem::drop(e);

        if entity.bind().walkable {
            self.walkable_hash.insert((x, y), entity.clone());
        }
        self.entity_list.push(entity.clone());
        self.entity_hash.insert(self.last_id, entity, (x, y));
    }

    pub fn iter_visibles_mut(&mut self) -> impl Iterator<Item = &mut Gd<GenericScriptedEntity>> {
        self.entity_list.iter_mut()
    }
}
