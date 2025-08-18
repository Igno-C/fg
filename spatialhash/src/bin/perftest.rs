use std::{cell::RefCell, rc::Rc};

use rand::{Rng, SeedableRng, rngs::StdRng};
use spatialhash::{*, benchmark::benchmark};


struct TestObject {
    pos: (i32, i32),
    id: i32,
}

impl TestObject {
    fn new(x: i32, y: i32, id: i32) -> TestObject {
        TestObject {
            pos: (x, y),
            id
        }
    }
}

impl Positioned for TestObject {
    fn get_pos(&self) -> (i32, i32) {
        self.pos
    }
}

impl Positionable for TestObject {
    fn set_pos(&mut self, x: i32, y: i32) {
        self.pos = (x, y)
    }
}

fn main() {
    full_spatial_benchmark(
        100,
        100,
        200,
        30,
        1
    );

    full_spatial_benchmark(
        1000,
        1000,
        200,
        30,
        2
    );

    full_spatial_benchmark(
        500,
        500,
        50,
        30,
        3
    );
}


fn full_spatial_benchmark(num_objects: usize, num_checks: usize, check_size: i32, max_distance: i32, seed: u64) {
    println!("\n# Max distance {}, {} objects, seeded random uniform distribution, {}x{} map:", max_distance, num_objects, check_size, check_size);
    let spawn_positions = generate_uniform_id(num_objects, check_size, seed);
    let swap_positions = generate_uniform(num_objects, check_size, seed+50000);
    let check_positions = generate_uniform(num_checks, check_size, seed+1000000000);
    
    let mut lazy = LazyChecker::new(max_distance);
    let mut field1 = SpatialHash::new(max_distance, (-check_size/2, -check_size/2), (check_size/2 - 1, check_size/2 - 1), 1);
    let mut field2 = SpatialHash::new(max_distance/2, (-check_size/2, -check_size/2), (check_size/2 - 1, check_size/2 - 1), 2);
    let mut field3 = SpatialHash::new(max_distance/3, (-check_size/2, -check_size/2), (check_size/2 - 1, check_size/2 - 1), 3);

    benchmark("Lazy object insertion", || {
        for pos in spawn_positions.iter() {
            lazy.insert(TestObject::new(pos.0, pos.1, pos.2));
        }
    });
    benchmark("Field object insertion", || {
        for pos in spawn_positions.iter() {
            field1.insert(pos.2, (pos.0, pos.1));
        }
    });
    for pos in spawn_positions.iter() {
        field2.insert(pos.2, (pos.0, pos.1));
        field3.insert(pos.2, (pos.0, pos.1));
    }

    benchmark(format!("Lazy distance checking"), || {
        for check_pos in check_positions.iter() {
            for adjacent in lazy.get_adjacent(*check_pos) {
                std::hint::black_box(adjacent);
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

    benchmark(format!("Field distance checking"), || {
        for check_pos in check_positions.iter() {
            field1.for_each_adjacent(*check_pos, |adjacent| {
                std::hint::black_box(adjacent);
            });
        }
    });
    let mut checked_num = 0;
    for check_pos in check_positions.iter() {
        field1.for_each_adjacent(*check_pos, |_adjacent| {
            checked_num += 1;
        });
    }
    println!("Checked {} positions", checked_num);

    benchmark(format!("Field with 2-wide distance checking"), || {
        for check_pos in check_positions.iter() {
            field2.for_each_adjacent(*check_pos, |adjacent| {
                std::hint::black_box(adjacent);
            });
        }
    });
    let mut checked_num = 0;
    for check_pos in check_positions.iter() {
        field2.for_each_adjacent(*check_pos, |_adjacent| {
            checked_num += 1;
        });
    }
    println!("Checked {} positions", checked_num);

    benchmark(format!("Field with 3-wide distance checking"), || {
        for check_pos in check_positions.iter() {
            field3.for_each_adjacent(*check_pos, |adjacent| {
                std::hint::black_box(adjacent);
            });
        }
    });
    let mut checked_num = 0;
    for check_pos in check_positions.iter() {
        field3.for_each_adjacent(*check_pos, |_adjacent| {
            checked_num += 1;
        });
    }
    println!("Checked {} positions", checked_num);

    benchmark("Lazy object moving", || {
        for (o, pos) in lazy.iter_mut().zip(swap_positions.iter()) {
            std::hint::black_box(o.set_pos(pos.0, pos.1));
        }
    });
    benchmark("Field object moving", || {
        for (pos, newpos) in spawn_positions.iter().zip(swap_positions.iter()) {
            field1.update_pos(pos.2, (pos.0, pos.1), *newpos);
        }
    });
}

// fn field_benchmark()

fn generate_uniform(length: usize, range: i32, seed: u64) -> Vec<(i32, i32)> {
    let mut rng = StdRng::seed_from_u64(seed);
    (0..length)
        .map(|_| (rng.random_range(-range/2..range/2), rng.random_range(-range/2..range/2)))
        .collect()
}

/// Generates randomly with seed, in range -range..range
/// 
/// (x, y, id)
fn generate_uniform_id(length: usize, range: i32, seed: u64) -> Vec<(i32, i32, i32)> {
    let mut rng = StdRng::seed_from_u64(seed);
    (0..length)
        .map(|_| (rng.random_range(-range/2..range/2), rng.random_range(-range/2..range/2), rng.random()))
        .collect()
}
