use godot::prelude::*;

pub use rgdext_shared::{serverconnector::ServerConnector, playerdata::playercontainer::PlayerContainer};

struct FGExtensionServerUtil;

#[gdextension]
unsafe impl ExtensionLibrary for FGExtensionServerUtil {
    
}
