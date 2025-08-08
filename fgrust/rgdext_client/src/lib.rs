use godot::prelude::*;
pub use rgdext_shared::{playerdata::*, basemap::*};

// pub mod server;

struct FGExtensionClient;

#[gdextension]
unsafe impl ExtensionLibrary for FGExtensionClient {
    
}
