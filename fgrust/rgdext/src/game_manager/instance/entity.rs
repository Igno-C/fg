use std::collections::HashMap;

use godot::prelude::*;
use rgdext_shared::{basemap::spatialhash::SpatialHash, playerdata::item::ItemResource};

/// Only overrwite on_* virtual methods.
/// 
/// If a null response is returned (as by default), no signal will be sent.
#[derive(GodotClass)]
#[class(base=Node)]
pub struct GenericScriptedEntity {
    #[export]
    /// Position of the entity in the world.
    pub pos: Vector2i,
    #[export]
    /// Whether the entity can be interacted with.
    /// 
    /// Interaction response is based on the [method _on_player_interaction] method.
    pub interactable: bool,
    // #[export]
    // /// If interactable, what's the max Chebyshev distance the entity can be interacted with. 0 means just the position.
    // pub interactable_distance: i32,
    #[export]
    /// Whether the entity should trigger on being walked on.
    /// 
    /// Walked on response is based on the [method _on_player_walk] method.
    pub walkable: bool,
    #[export]
    /// Whether the entity has a client scene that should be shown to the clients.
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
            // interactable_distance: 0,
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
        if response.bind().response.is_null() {
            return;
        }

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
    pub fn on_player_walk(this: Gd<Self>, net_id: i32) -> Gd<ScriptResponse> {
        ScriptResponse::null_response()
    }

    #[func(gd_self, virtual)]
    pub fn on_player_interaction(this: Gd<Self>, with_item: Option<Gd<ItemResource>>, net_id: i32) -> Gd<ScriptResponse> {
        ScriptResponse::null_response()
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
    DespawnSelf{},
    RegisterEntity{entity: Gd<GenericScriptedEntity>},
    SystemChatMessage{text: GString, net_id: i32},
    Null
}

impl ResponseType {
    pub fn is_null(&self) -> bool {
        match self {
            ResponseType::Null => {true},
            _ => {false},
        }
    }
}

// Struct that holds a list of interactable entities
#[derive(Default)]
pub struct Entities {
    // Used to give registered visible entities unique ids
    last_id: i32,

    interactables: HashMap<(i32, i32), Gd<GenericScriptedEntity>>,
    walkables: HashMap<(i32, i32), Gd<GenericScriptedEntity>>,
    visible_hash: SpatialHash<i32, Gd<GenericScriptedEntity>>,
    visibles: Vec<Gd<GenericScriptedEntity>>
}

impl Entities {
    pub fn set_spatial_hash(&mut self, hash: SpatialHash<i32, Gd<GenericScriptedEntity>>) {
        self.visible_hash = hash;
    }

    pub fn get_interactable_at(&mut self, x: i32, y: i32) -> Option<&mut Gd<GenericScriptedEntity>> {
        return self.interactables.get_mut(&(x, y));
    }

    pub fn get_walkable_at(&mut self, x: i32, y: i32) -> Option<&mut Gd<GenericScriptedEntity>> {
        return self.walkables.get_mut(&(x, y));
    }

    pub fn get_visible_hash(&self) -> &SpatialHash<i32, Gd<GenericScriptedEntity>> {
        &self.visible_hash
    }

    pub fn get_visible_hash_mut(&mut self) -> &mut SpatialHash<i32, Gd<GenericScriptedEntity>> {
        &mut self.visible_hash
    }

    pub fn register_entity(&mut self, entity: Gd<GenericScriptedEntity>) {
        if entity.bind().interactable {
            // godot_print!("Registering interactable");
            self.register_interactable(entity.clone());
        }
        if entity.bind().walkable {
            // godot_print!("Registering walkable");
            self.register_walkable(entity.clone());
        }
        if !entity.bind().related_scene.is_empty() {
            // godot_print!("Registering visible");
            self.register_visible(entity);
        }
    }

    pub fn iter_visibles_mut(&mut self) -> impl Iterator<Item = &mut Gd<GenericScriptedEntity>> {
        self.visibles.iter_mut()
    }

    fn register_interactable(&mut self, entity: Gd<GenericScriptedEntity>) {
        let e = entity.bind();
        let x = e.pos.x; let y = e.pos.y;
        std::mem::drop(e);

        self.interactables.insert((x, y), entity);
    }

    fn register_walkable(&mut self, entity: Gd<GenericScriptedEntity>) {
        let e = entity.bind();
        let x = e.pos.x; let y = e.pos.y;
        std::mem::drop(e);

        self.walkables.insert((x, y), entity);
    }

    fn register_visible(&mut self, mut entity: Gd<GenericScriptedEntity>) {
        let e = entity.bind();
        let x = e.pos.x; let y = e.pos.y;
        std::mem::drop(e);

        self.last_id += 1;
        entity.bind_mut().entity_id = self.last_id;
        self.visibles.push(entity.clone());
        self.visible_hash.insert(self.last_id, entity, (x, y));
    }
}
