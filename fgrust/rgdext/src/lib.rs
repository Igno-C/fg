use godot::prelude::*;
pub use rgdext_shared::{basemap::*, playerdata::*, serverconnector::*};
// pub use rgdext_shared;

mod server;
mod eventqueue;
mod game_manager;

struct FGExtension;

#[gdextension]
unsafe impl ExtensionLibrary for FGExtension {
    
}
