use godot::prelude::*;
extern crate rgdext_shared;

mod server;
mod eventqueue;
mod game_manager;

struct FGExtension;

#[gdextension]
unsafe impl ExtensionLibrary for FGExtension {
    
}
