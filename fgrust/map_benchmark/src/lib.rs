use godot::prelude::*;
use rgdext_shared::basemap::BaseMap;

extern crate rgdext_shared;

struct FGExtensionMapBenchmark;

#[gdextension]
unsafe impl ExtensionLibrary for FGExtensionMapBenchmark {
    
}


#[derive(GodotClass)]
#[class(base=Node)]
struct BenchmarkNode {
    base: Base<Node>
}

#[godot_api]
impl INode for BenchmarkNode {
    fn init(base: Base<Node>) -> Self {
        Self {
            base
        }
    }
}

#[godot_api]
impl BenchmarkNode {
    #[func]
    fn do_perf_bench(&self) {
        let map_scene = load::<PackedScene>("res://map1.tscn");

        let map1 = map_scene.instantiate().unwrap().cast::<BaseMap>();
        let map1 = map_scene.instantiate().unwrap().cast::<BaseMap>();
    }
}

