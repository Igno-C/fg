use bitcode::{Decode, Encode};
use godot::prelude::*;


#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
/// Generic event that can be created by the player at any moment.
/// 
/// Use the various constructors to create specific events, then send it to the server via pevent.
pub struct GenericEvent {
    event: GenericPlayerEvent,
    
    base: Base<RefCounted>
}

#[godot_api]
impl GenericEvent {
    #[func]
    fn interaction(x: i32, y: i32, entity_id: i32) -> Gd<Self> {
        Gd::from_init_fn(|base| {
            Self{event: GenericPlayerEvent::Interaction{x, y, entity_id}, base}
        })
    }

    #[func]
    fn swap_items(from_index: i32, to_index: i32) -> Gd<Self> {
        Gd::from_init_fn(|base| {
            Self{event: GenericPlayerEvent::SwapItems{from: from_index as usize, to: to_index as usize}, base}
        })
    }

    #[func]
    fn equip_item(from_index: i32) -> Gd<Self> {
        Gd::from_init_fn(|base| {
            Self{event: GenericPlayerEvent::EquipItem{from: from_index as usize}, base}
        })
    }

    // #[func]
    // fn interaction_with_item(x: i32, y: i32, item_index: i32) -> Gd<Self> {
    //     Gd::from_init_fn(|base| {
    //         Self{event: GenericPlayerEvent::Interaction{x, y, item_index: Some(item_index as usize)}, base}
    //     })
    // }

    #[func]
    pub fn to_bytearray(&self) -> PackedByteArray {
        self.event.to_bytearray()
    }
}

#[derive(Decode, Encode, Clone)]
pub enum GenericPlayerEvent {
    Interaction{x: i32, y: i32, entity_id: i32},
    SwapItems{from: usize, to: usize},
    EquipItem{from: usize},
    Err
}

impl GenericPlayerEvent {
    pub fn from_bytes(b: &[u8]) -> Self {
        match bitcode::decode(b) {
            Ok(e) => e,
            Err(_err) => GenericPlayerEvent::Err,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bitcode::encode(self)
    }

    pub fn to_bytearray(&self) -> PackedByteArray {
        PackedByteArray::from(self.to_bytes())
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
/// Generic response that the server may send to the client at any moment.
/// 
/// This Godot wrapper class is only meant to be used on client side to parse 
pub struct GenericResponse {
    response: GenericServerResponse,
    
    base: Base<RefCounted>
}

#[godot_api]
impl GenericResponse {
    #[func]
    pub fn from_bytearray(b: PackedByteArray) -> Gd<GenericResponse> {
        Gd::from_init_fn(|base| {
            GenericResponse {
                response: GenericServerResponse::from_bytes(b.as_slice()),
                base
            }
        })
    }

    #[constant]
    const RESPONSE_ERR: i32 = -1;
    #[constant]
    const RESPONSE_LOAD_MAP: i32 = 0;

    #[func]
    pub fn response_type(&self) -> i32 {
        match &self.response {
            GenericServerResponse::LoadMap{mapname: _} => Self::RESPONSE_LOAD_MAP,
            GenericServerResponse::Err => Self::RESPONSE_ERR,
        }
    }

    #[func]
    pub fn as_load_map(&self) -> String {
        match &self.response {
            GenericServerResponse::LoadMap{mapname} => mapname.clone(),
            _ => "".to_string(),
        }
    }
}

#[derive(Decode, Encode, Clone)]
pub enum GenericServerResponse {
    LoadMap{mapname: String},
    Err,
}

impl GenericServerResponse {
    pub fn from_bytes(b: &[u8]) -> Self {
        match bitcode::decode(b) {
            Ok(e) => e,
            Err(_err) => GenericServerResponse::Err,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bitcode::encode(self)
    }

    pub fn to_bytearray(&self) -> PackedByteArray {
        PackedByteArray::from(self.to_bytes())
    }
}
