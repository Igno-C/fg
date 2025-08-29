use godot::prelude::*;
pub use rgdext_shared::{
    playerdata::playercontainer::PlayerContainer,
    basemap::BaseMap, genericevent::GenericEvent
};

// pub mod server;

struct FGExtensionClient;

#[gdextension]
unsafe impl ExtensionLibrary for FGExtensionClient {
    
}
