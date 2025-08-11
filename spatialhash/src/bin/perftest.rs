use rand::{Rng, SeedableRng, rngs::StdRng};
use spatialhash::{*, benchmark::benchmark};


struct TestObject {
    x: i32,
    y: i32,
    dummy_data: [u8; 1000],
}

impl TestObject {
    fn new(x: i32, y: i32) -> Box<TestObject> {
        Box::new(
            TestObject {
                x, y, dummy_data: [0; 1000]
            }
        )
    }
}

impl Positioned for Box<TestObject> {
    fn get_pos(&self) -> (i32, i32) {
        (self.x, self.y)
    }
}

impl Positionable for Box<TestObject> {
    fn set_pos(&mut self, x: i32, y: i32) {
        self.x = x; self.y = y;
    }
}

fn main() {
    lazy_vs_hash_uniform_benchmark(
        100,
        100,
        200,
        50,
        1
    );

    lazy_vs_hash_uniform_benchmark(
        1000,
        1000,
        200,
        50,
        2
    );
}


fn lazy_vs_hash_uniform_benchmark(num_objects: usize, num_checks: usize, map_size: i32, max_distance: i32, seed: u64) {
    println!("# Max distance {}, {} objects, seeded random uniform distribution, {}x{} map:", max_distance, num_objects, map_size, map_size);
    let positions = generate_uniform(num_objects, map_size, seed);
    let check_positions = generate_uniform(num_checks, map_size, seed+1000000000);
    let mut lazy = LazyChecker::new(max_distance);
    for pos in positions.iter() {
        lazy.insert(TestObject::new(pos.0, pos.1));
    }
    benchmark(format!("Lazy distance checking"), || {
        for check_pos in check_positions.iter() {
            for adjacent in lazy.get_adjacent(*check_pos) {
                std::hint::black_box(adjacent.dummy_data);
            }
        }
    });
}

/// Generates randomly with seed, in range -range..range
fn generate_uniform(length: usize, range: i32, seed: u64) -> Vec<(i32, i32)> {
    let mut rng = StdRng::seed_from_u64(seed);
    (0..length)
        .map(|_| (rng.random_range(-range/2..range/2), rng.random_range(-range/2..range/2)))
        .collect()
}
