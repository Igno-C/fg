use godot::prelude::*;
extern crate rgdext_shared;

struct FGExtensionClient;

#[gdextension]
unsafe impl ExtensionLibrary for FGExtensionClient {
    
}
