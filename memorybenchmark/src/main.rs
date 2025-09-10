use std::collections::HashSet;
use std::env;

fn main() {
    // Read the command line argument
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <number_of_tuples>", args[0]);
        return;
    }

    let num_tuples: usize = match args[1].parse() {
        Ok(num) => num,
        Err(_) => {
            eprintln!("Please provide a valid integer.");
            return;
        }
    };

    // Create a HashSet to store (i32, i32) tuples
    let mut hash_set: HashSet<(i32, i32)> = HashSet::new();

    // Insert tuples into the HashSet
    for i in 0..num_tuples {
        hash_set.insert((i as i32, (i * 2) as i32));
    }

    std::hint::black_box(hash_set);
}
