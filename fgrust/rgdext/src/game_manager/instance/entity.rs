use std::collections::HashMap;

use godot::prelude::*;
use rgdext_shared::basemap::spatialhash::SpatialHash;

/// Only overrwite on_* virtual methods.
/// 
/// If a null response is returned (as by default), no signal will be sent.
#[derive(GodotClass)]
#[class(base=Node)]
pub struct GenericScriptedEntity {
    #[export]
    pub pos: Vector2i,
    #[export]
    pub interactable: bool,
    #[export]
    pub interactable_distance: i32,
    #[export]
    pub walkable: bool,
    #[export]
    pub related_scene: GString,


    // instance: Option<Gd<super::Instance>>,
    // entities: Option<std::rc::Rc<Entities>>,

    base: Base<Node>
}

#[godot_api]
impl INode for GenericScriptedEntity {
    fn init(base: Base<Node>) -> Self {
        Self {
            pos: Vector2i::ZERO,
            interactable: false,
            interactable_distance: 0,
            walkable: false,
            related_scene: "".to_godot(),
            
            base
        }
    }

    // fn ready(&mut self) {
    //     // Entity <- Entities Node <- BaseMap <- Instance
    //     // let instance = self.base()
    //     //     .get_parent().unwrap()
    //     //     .get_parent().unwrap()
    //     //     .get_parent().unwrap()
    //     //     .cast::<super::Instance>();

    //     // // self.instance = Some(instance);

    //     // self.base_mut().connect("entity_response", &Callable::from_object_method(&instance, "handle_entity_response"));
    //     // self.base_mut().connect("register_interactable", &Callable::from_object_method(&instance, "register_interactable"));
       
    //     // Freeing stuff like sprites and the like
    //     // for mut child in self.base().get_children().iter_shared() {
    //     //     child.queue_free();
    //     // }

    //     if self.interactable {
    //         let this = self.to_gd().clone();
    //         self.base_mut().emit_signal("register_interactable", vslice![this]);
    //         // self.register_interactable();
    //     }

    //     let response = self.on_ready();

    //     self.emit_response(response);
    // }

    // fn process(&mut self, delta: f64) {
    //     let response = self.on_process(delta);

    //     self.emit_response(response);
    // }
}

#[godot_api]
impl GenericScriptedEntity {
    #[signal]
    fn entity_response(object: Gd<GenericScriptedEntity>, response: Gd<ScriptResponse>);

    // #[signal]
    // fn register_interactable(this: Gd<GenericScriptedEntity>);

    #[func]
    fn emit_response(&mut self, response: Gd<ScriptResponse>) {
        if response.bind().response.is_null() {
            return;
        }

        let this = self.to_gd();

        self.base_mut().emit_signal("entity_response", vslice![this, response]);
    }

    // #[func]
    // fn register_interactable(&mut self) {
    //     self.interactable = true;
    // }


    // #[func]
    // fn get_pos(&self) -> Vector2i {
    //     Vector2i {x: self.x, y: self.y}
    // }

    // #[func]
    // fn register_interactable(&mut self) {
    //     let e = self.to_gd().clone();
    //     let mut i = self.instance.as_ref().unwrap().clone();
    //     godot_print!("This: {:?}, That: {:?}", e, i);

    //     // i.bind_mut().entities.register_interactable(e);
    //     // self.instance.as_mut().unwrap().bind_mut().entities.register_interactable(e);
    // }

    // #[func]
    // fn move_to(&mut self, x: i32, y: i32, check_collisions: bool) {

    // }

    #[func(virtual)]
    pub fn on_player_walk(&mut self, net_id: i32) -> Gd<ScriptResponse> {
        ScriptResponse::null_response()
    }

    #[func(virtual)]
    pub fn on_player_interaction(&mut self, net_id: i32) -> Gd<ScriptResponse> {
        ScriptResponse::null_response()
    }

    // #[func(virtual)]
    // fn on_ready(&mut self) -> Gd<ScriptResponse> {
    //     ScriptResponse::null_response()
    // }

    // #[func(virtual)]
    // fn on_process(&mut self, _delta: f64) -> Gd<ScriptResponse>  {
    //     ScriptResponse::null_response()
    // }
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
    last_id: i32,

    interactables: HashMap<(i32, i32), Gd<GenericScriptedEntity>>,
    walkables: HashMap<(i32, i32), Gd<GenericScriptedEntity>>,
    visibles: SpatialHash<i32, Gd<GenericScriptedEntity>>,
}

impl Entities {
    pub fn get_interactable_at(&mut self, x: i32, y: i32) -> Option<&mut Gd<GenericScriptedEntity>> {
        return self.interactables.get_mut(&(x, y));
    }

    pub fn get_walkable_at(&mut self, x: i32, y: i32) -> Option<&mut Gd<GenericScriptedEntity>> {
        return self.walkables.get_mut(&(x, y));
    }

    pub fn register_interactable(&mut self, entity: Gd<GenericScriptedEntity>) {
        let e = entity.bind();
        let x = e.pos.x; let y = e.pos.y;
        std::mem::drop(e);

        self.interactables.insert((x, y), entity);
    }

    pub fn register_walkable(&mut self, entity: Gd<GenericScriptedEntity>) {
        let e = entity.bind();
        let x = e.pos.x; let y = e.pos.y;
        std::mem::drop(e);

        self.walkables.insert((x, y), entity);
    }

    pub fn register_visible(&mut self, entity: Gd<GenericScriptedEntity>) {
        let e = entity.bind();
        let x = e.pos.x; let y = e.pos.y;
        std::mem::drop(e);

        self.last_id += 1;
        self.visibles.insert(self.last_id, entity, (x, y));
    }
}
