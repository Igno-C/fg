use std::collections::{HashSet};

use godot::{classes::{FileAccess, TileMapLayer}, prelude::*};
use rgdext_shared::basemap::CollisionArray;

// use malloc_size_of::MallocSizeOf;

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
        let cmap_scene = load::<PackedScene>("res://cmap_scene.tscn");

        let cmap = cmap_scene.instantiate().unwrap().cast::<TileMapLayer>();

        let col_array_data = FileAccess::get_file_as_bytes("res://col_array.col");
        let col_array = CollisionArray::from_bytes(col_array_data.as_slice()).unwrap();
        let (topleft, bottomright) = col_array.get_dimensions();

        godot_print!("Checking map with an area from {:?} to {:?}", topleft, bottomright);

        let mut small_positions = Vec::new();
        let mut col_hash = HashSet::new();
        for x in topleft.0..=bottomright.0 {
            for y in topleft.1..=bottomright.1 {
                small_positions.push((x, y));
                if col_array.get_at(x, y) {
                    col_hash.insert((x, y));
                }
            }
        }
        let mut big_positions1 = Vec::with_capacity(40000);
        for _ in 0..20 {
            big_positions1.append(&mut small_positions.clone());
        }

        let big_positions2 = big_positions1.clone();
        let big_positions3 = big_positions1.clone();

        godot_print!("This many checks in total: {}\n", big_positions1.len());
        godot_print!("Tilemap check:");
        let tilemap_time = std::time::Instant::now();
        for (x, y) in big_positions2.into_iter() {
            std::hint::black_box(
                // If there is a collision tile there, it returns 0, 0
                // Otherwise, this returns -1, -1
                cmap.get_cell_atlas_coords(Vector2i{x, y}).x == 0
            );
        }
        let tilemap_elapsed = tilemap_time.elapsed();
        godot_print!("Took {} us", tilemap_elapsed.as_micros());

        godot_print!("\nCollision array check:");
        let colmap_time = std::time::Instant::now();
        for (x, y) in big_positions1.into_iter() {
            std::hint::black_box(
                col_array.get_at(x, y)  
            );
        }
        let colmap_elapsed = colmap_time.elapsed();
        godot_print!("Took {} us", colmap_elapsed.as_micros());

        godot_print!("\nHash map check:");
        let hash_time = std::time::Instant::now();
        for (x, y) in big_positions3.into_iter() {
            std::hint::black_box(
                col_hash.contains(&(x, y))
            );
        }
        let hash_elapsed = hash_time.elapsed();
        godot_print!("Took {} us", hash_elapsed.as_micros());
    }
}

