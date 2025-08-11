pub fn benchmark<F: FnOnce() -> ()>(name: impl ToString, f: F) {
    println!("\nPerforming benchmark: {}", name.to_string());

    let now = std::time::Instant::now();
    f();
    let duration = now.elapsed();
    
    println!("Operation finished in {:.2} s, {} ms, {} us\n", duration.as_secs_f32(), duration.as_millis(), duration.as_micros());
}

pub fn benchmark_micros<F: FnOnce() -> ()>(f: F) -> u128 {
    let now = std::time::Instant::now();
    f();
    let duration = now.elapsed();

    return duration.as_micros();
}

pub fn benchmark_avg<T, P: Fn() -> T, F: Fn(T) -> ()>(prep: P, bench: F, repeat_count: usize) -> (f64, f64) {
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

    println!("Average of {:.1} us, standard deviation of {:.1} us", average, std_dev);

    (average, std_dev)
}
