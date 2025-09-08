use rand::{random_range, seq::SliceRandom, Rng};
use spatialhash::{benchmark::*, spatialhash::SpatialHash, LazyChecker, TestObject};
use std::{cell::RefCell, rc::Rc};

const NUMBER_OF_REPEATS_FOR_AVG: usize = 1000;


fn main() {
    let comp1 = Comparison {
        lazy_check_distance: 25,
        hash_grid_size: 25,
        hash_check_radius: 1,
        num_objects: 200,
        num_checks: 1000,
        check_area: 80,
    };
    comp1.run_full();

    let comp2 = Comparison {
        lazy_check_distance: 25,
        hash_grid_size: 25,
        hash_check_radius: 1,
        num_objects: 200,
        num_checks: 1000,
        check_area: 30,
    };
    comp2.run_full();

    let comp3 = Comparison {
        lazy_check_distance: 25,
        hash_grid_size: 25,
        hash_check_radius: 1,
        num_objects: 500,
        num_checks: 1000,
        check_area: 200,
    };
    comp3.run_full();

    println!("\n\nHash only tests:\n");

    Comparison::run_hash_tests(
        500,
        1000,
        200,
        2,
        12
    );

    Comparison::run_hash_tests(
        500,
        1000,
        200,
        3,
        8
    );
}

struct Comparison {
    pub lazy_check_distance: i32,
    pub hash_grid_size: i32,
    pub hash_check_radius: i32,

    pub num_objects: usize,
    pub num_checks: usize,
    pub check_area: i32
}

impl Comparison {
    fn run_full(self) {
        let Comparison {
            lazy_check_distance,
            hash_grid_size,
            hash_check_radius,
        
            num_objects,
            num_checks,
            check_area
        } = self;

        println!("\n\n\nFull benchmark with a check distance of {lazy_check_distance} (or {hash_grid_size}x{hash_check_radius})");
        println!("{num_objects} objects, {num_checks} checks in square area of (-{check_area}, -{check_area}) to ({check_area}, {check_area}):\n\n");

        Comparison::run_lazy_tests(num_objects, num_checks, check_area, lazy_check_distance);
        
        Comparison::run_hash_tests(num_objects, num_checks, check_area, hash_check_radius, hash_grid_size);
    }

    fn run_lazy_tests(num_objects: usize, num_checks: usize, check_area: i32, lazy_check_distance: i32) {
        println!("\nLazy checker insertion:");
        benchmark_avg(|| {
            let data = generate_data(num_objects, check_area);
            let lazy_checker = LazyChecker::new(lazy_check_distance);
            (data, lazy_checker)
        }, |(data, mut lazy_checker)| {
            for (id, object) in data.into_iter() {
                lazy_checker.insert(id, object);
            }
            std::hint::black_box(lazy_checker);
        }, NUMBER_OF_REPEATS_FOR_AVG);

        println!("\nLazy checker adjacency checks:");
        let mut lazy_check_count = 0;
        benchmark_avg(|| {
            let data = generate_data(num_objects, check_area);
            let check_positions = generate_positions(num_checks, check_area);
            let mut lazy_checker = LazyChecker::new(lazy_check_distance);
            for (id, object) in data.into_iter() {
                lazy_checker.insert(id, object);
            }
            (lazy_checker, check_positions)
        }, |(lazy_checker, check_positions)| {
            for pos in check_positions.into_iter() {
                lazy_checker.get_adjacent(pos).for_each(|o| {
                    std::hint::black_box(o);
                    lazy_check_count += 1;
                });
            }
        }, NUMBER_OF_REPEATS_FOR_AVG);
        println!("Average amount of adjacencies per check set: {}", lazy_check_count / NUMBER_OF_REPEATS_FOR_AVG);

        println!("\nLazy checker movement checks:");
        benchmark_avg(|| {
            let data = generate_data(num_objects, check_area);
            let movements = generate_movements(num_checks);
            let mut lazy_checker = LazyChecker::new(lazy_check_distance);
            for (id, object) in data.into_iter() {
                lazy_checker.insert(id, object);
            }
            (lazy_checker, movements)
        }, |(lazy_checker, movements)| {
            for ((_id, player), delta) in lazy_checker.iter().zip(movements.iter()) {
                let mut b = player.borrow_mut();
                b.pos.0 += delta.0;
                b.pos.1 += delta.1;
            }
        }, NUMBER_OF_REPEATS_FOR_AVG);
    }

    fn run_hash_tests(num_objects: usize, num_checks: usize, check_area: i32, hash_check_radius: i32, hash_grid_size: i32) {
        let base_spatial_hash: SpatialHash<i32, Rc<RefCell<TestObject>>> = SpatialHash::new(
            hash_grid_size,
            (-check_area, -check_area),
            (check_area, check_area),
            hash_check_radius
        );

        println!("\nSpatial hash insertion:");
        benchmark_avg(|| {
            let data = generate_data(num_objects, check_area);
            let spatial_hash = base_spatial_hash.clone();
            (data, spatial_hash)
        }, |(data, mut spatial_hash)| {
            for (id, object) in data.into_iter() {
                let pos = object.borrow().pos;
                spatial_hash.insert(id, object, pos);
            }
            std::hint::black_box(spatial_hash);
        }, NUMBER_OF_REPEATS_FOR_AVG);

        println!("\nSpatial hash adjacency check:");
        let mut hash_check_count = 0;
        benchmark_avg(|| {
            let data = generate_data(num_objects, check_area);
            let check_positions = generate_positions(num_checks, check_area);
            let mut spatial_hash = base_spatial_hash.clone();
            for (id, object) in data.into_iter() {
                let pos = object.borrow().pos;
                spatial_hash.insert(id, object, pos);
            }
            (spatial_hash, check_positions)
        }, |(spatial_hash, check_positions)| {
            for pos in check_positions.into_iter() {
                spatial_hash.for_each_adjacent(pos, |o| {
                    std::hint::black_box(o);
                    hash_check_count += 1;
                });
            }
        }, NUMBER_OF_REPEATS_FOR_AVG);
        println!("Average amount of adjacencies per check set: {}", hash_check_count / NUMBER_OF_REPEATS_FOR_AVG);

        println!("\nSpatial hash movement check:");
        benchmark_avg(|| {
            let data = generate_data(num_objects, check_area);
            let movements= generate_movements(num_checks);
            let starting_data: Vec<(i32, (i32, i32))> = data.iter().map(|o| (o.0, o.1.borrow().pos)).collect();
            let mut spatial_hash = base_spatial_hash.clone();
            for (id, object) in data.into_iter() {
                let pos = object.borrow().pos;
                spatial_hash.insert(id, object, pos);
            }
            (spatial_hash, movements, starting_data)
        }, |(mut spatial_hash, movements, starting_data)| {
            for (delta, (id, oldpos)) in movements.into_iter().zip(starting_data.into_iter()) {
                let newpos = (oldpos.0 + delta.0, oldpos.1 + delta.1);
                spatial_hash.update_pos(id, oldpos, newpos);
            }
        }, NUMBER_OF_REPEATS_FOR_AVG);
    }
}

fn generate_movements(how_many: usize) -> Vec<(i32, i32)> {
    (0..how_many).map(|_| (rand::rng().random_range(-1..=1), rand::rng().random_range(-1..=1))).collect()
}

fn generate_data(how_many: usize, range: i32) -> Vec<(i32, Rc<RefCell<TestObject>>)> {
    let mut out: Vec<(i32, Rc<RefCell<TestObject>>)> = generate_positions(how_many, range).into_iter().enumerate().map(|(i, pos)| {
        (i as i32, TestObject::new_rc(pos))
    }).collect();

    out.shuffle(&mut rand::rng());

    out
}

fn generate_positions(how_many: usize, range: i32) -> Vec<(i32, i32)> {
    (0..how_many).map(|_| {
        (random_range(-range..=range), random_range(-range..=range))
    }).collect()
}

// fn generate_shuffled_indexes(how_many: usize) -> Vec<usize> {
//     let mut v: Vec<usize> = (0..how_many).collect();
//     v.shuffle(&mut rand::rng());
//     v
// }
