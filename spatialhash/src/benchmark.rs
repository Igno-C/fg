


pub fn benchmark_micros<F: FnOnce() -> ()>(f: F) -> u128 {
    let now = std::time::Instant::now();
    f();
    let duration = now.elapsed();

    return duration.as_micros();
}

pub fn benchmark_avg<T, P: FnMut() -> T, F: FnMut(T) -> ()>(mut prep: P, mut bench: F, repeat_count: usize) -> (f64, f64) {
    let mut durations = Vec::with_capacity(repeat_count);
    
    for _ in 0..repeat_count {
        let prepped = prep();
        durations.push(benchmark_micros(|| {bench(prepped)}));
    }

    let total_duration: u128 = durations.iter().sum();
    let average = total_duration as f64 / repeat_count as f64;

    let variance = durations.iter()
        .map(|&duration| {
            let diff = duration as f64 - average;
            diff * diff
        })
        .sum::<f64>() / repeat_count as f64;

    let std_dev = variance.sqrt();

    println!("Average of {:.1} us, standard deviation of {:.1} us after {} repeats", average, std_dev, repeat_count);

    (average, std_dev)
}
