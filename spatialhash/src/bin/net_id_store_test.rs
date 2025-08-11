use spatialhash::benchmark::{benchmark, benchmark_avg};
use rand::{rng, rngs::StdRng, Rng, SeedableRng};
use rand::seq::SliceRandom;
use std::collections::{HashMap, BTreeMap};

fn main() {
    single_multibenchmark(2000, 10000)
}

fn single_multibenchmark(num_ids: usize, num_repeats: usize) {
    println!("Hashmap {} inserts", num_ids);
    benchmark_avg(|| {
        (get_ids_and_checks(num_ids), HashMap::new())
    }, |(ids, mut map)| {
        for id in ids.0 {
            std::hint::black_box(map.insert(id, -1));
        }
    }, num_repeats);

    println!("Hashmap {} reads", num_ids);
    benchmark_avg(|| {
        let mut map = HashMap::new();
        let (ids, cids) = get_ids_and_checks(num_ids);
        for id in ids {map.insert(id, -1);}
        (cids, map)
    }, |(ids, map)| {
        for id in ids {
            std::hint::black_box(map.get(&id));
        }
    }, num_repeats);

    println!("Hashmap {} removals", num_ids);
    benchmark_avg(|| {
        let mut map = HashMap::new();
        let (ids, cids) = get_ids_and_checks(num_ids);
        for id in ids {map.insert(id, -1);}
        (cids, map)
    }, |(ids, mut map)| {
        for id in ids {
            std::hint::black_box(map.remove(&id));
        }
    }, num_repeats);
}

fn get_ids_and_checks(num_ids: usize) -> (Vec<i32>, Vec<i32>) {
    let ids: Vec<i32> = (0..num_ids).map(|_| (rng().random())).collect();
    let mut id_checks = ids.clone(); id_checks.shuffle(&mut rng());
    (ids, id_checks)
}

fn single_benchmark(num_ids: usize, seed: u64) {
    println!("For {} ids and checks:", num_ids);
    let ids = generate_uniform(num_ids, seed);
    let mut id_reads = ids.clone();
    id_reads.shuffle(&mut rng());
    
    let mut hash_map: HashMap<i32, i64> = HashMap::new();

    benchmark("HashMap inserts", || {
        for id in ids.iter() {
            hash_map.insert(*id, 1);
        }
    });

    benchmark("HashMap reads", || {
        for id_read in id_reads.iter() {
            std::hint::black_box(hash_map.get(id_read));
        }
    });

    benchmark("HashMap removals", || {
        for id_read in id_reads.iter() {
            std::hint::black_box(hash_map.remove(id_read));
        }
    });

    let mut btree_map: BTreeMap<i32, i64> = BTreeMap::new();

    benchmark("BTreeMap inserts", || {
        for id in ids.iter() {
            btree_map.insert(*id, 1);
        }
    });

    benchmark("BTreeMap reads", || {
        for id_read in id_reads.iter() {
            std::hint::black_box(btree_map.get(id_read));
        }
    });

    benchmark("BTreeMap removals", || {
        for id_read in id_reads.iter() {
            std::hint::black_box(btree_map.remove(id_read));
        }
    });
}

fn generate_uniform(length: usize, seed: u64) -> Vec<i32> {
    let mut rng = StdRng::seed_from_u64(seed);
    (0..length)
        .map(|_| (rng.random()))
        .collect()
}
