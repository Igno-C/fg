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
                x,
                y,
                dummy_data: [0; 1000]
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
    full_spatial_benchmark(
        100,
        100,
        200,
        200,
        30,
        0,
        1
    );

    full_spatial_benchmark(
        1000,
        1000,
        200,
        200,
        30,
        0,
        2
    );

    full_spatial_benchmark(
        1000,
        1000,
        200,
        200,
        30,
        10,
        2
    );
}


fn full_spatial_benchmark(num_objects: usize, num_checks: usize, check_size: i32, spawn_size: i32, max_distance: i32, sleep_time: u64, seed: u64) {
    println!("\n# Max distance {}, {} objects, seeded random uniform distribution, {}x{} map:", max_distance, num_objects, check_size, check_size);
    let positions = generate_uniform(num_objects, spawn_size, seed);
    let check_positions = generate_uniform(num_checks, check_size, seed+1000000000);
    
    let mut lazy = LazyChecker::new(max_distance);
    let mut hashed = SpatialHashChecker::new(max_distance);
    let mut field = SpatialFieldChecker::new(max_distance, (-check_size/2, -check_size/2), (check_size/2 - 1, check_size/2 - 1));
    for pos in positions.iter() {
        lazy.insert(TestObject::new(pos.0, pos.1));
        hashed.insert(TestObject::new(pos.0, pos.1));
        field.insert(TestObject::new(pos.0, pos.1));
    }


    benchmark(format!("Lazy distance checking"), || {
        for check_pos in check_positions.iter() {
            for adjacent in lazy.get_adjacent(*check_pos) {
                std::hint::black_box(adjacent.dummy_data);
                std::thread::sleep(std::time::Duration::from_micros(sleep_time));
            }
        }
    });
    let mut checked_num = 0;
    for check_pos in check_positions.iter() {
        for _adjacent in lazy.get_adjacent(*check_pos) {
            checked_num+=1;
        }
    }
    println!("Checked {} positions", checked_num);


    benchmark(format!("Hashed distance checking"), || {
        for check_pos in check_positions.iter() {
            hashed.for_each_adjacent(*check_pos, |adjacent| {
                std::hint::black_box(adjacent);
                std::thread::sleep(std::time::Duration::from_micros(sleep_time));
            });
        }
    });
    let mut checked_num = 0;
    for check_pos in check_positions.iter() {
        hashed.for_each_adjacent(*check_pos, |_adjacent| {
            checked_num += 1;
            // println!("Base pos {check_pos:?} is adjacent to {:?}", _adjacent.get_pos());
        });
    }
    println!("Checked {} positions", checked_num);


    benchmark(format!("Field distance checking"), || {
        for check_pos in check_positions.iter() {
            field.for_each_adjacent(*check_pos, |adjacent| {
                std::hint::black_box(adjacent);
                std::thread::sleep(std::time::Duration::from_micros(sleep_time));
            });
        }
    });
    let mut checked_num = 0;
    for check_pos in check_positions.iter() {
        field.for_each_adjacent(*check_pos, |_adjacent| {
            checked_num += 1;
            // println!("Base pos {check_pos:?} is adjacent to {:?}", _adjacent.get_pos());
        });
    }
    println!("Checked {} positions", checked_num);
}

/// Generates randomly with seed, in range -range..range
fn generate_uniform(length: usize, range: i32, seed: u64) -> Vec<(i32, i32)> {
    let mut rng = StdRng::seed_from_u64(seed);
    (0..length)
        .map(|_| (rng.random_range(-range/2..range/2), rng.random_range(-range/2..range/2)))
        .collect()
}
