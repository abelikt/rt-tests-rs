use crate::*;
use std::error::Error;

pub fn run_benchmarks() -> Result<(), Box<dyn Error>> {
    //! Run some experimental benchmarks
    //! They will be probably not be representative but should gvive a
    //! some rule of thumb values.

    benchmark_push(10)?;
    benchmark_push(1_000)?;
    benchmark_small_box(10)?;
    benchmark_small_box(1_000)?;
    Ok(())
}

fn benchmark_push(samples: u32) -> Result<(), Box<dyn Error>> {
    println!("Running push benchmark with {} samples", samples);
    let mut diff: i64;
    let mut accumulator: u64 = 0;
    let mut max_diff: i64 = 0;
    let mut vec: Vec<i32> = vec![0];

    for _s in 0..samples {
        let start = clock_gettime();

        vec.push(42);

        let end = clock_gettime();
        diff = Timespec::diff_ns(start, end);

        accumulator += diff as u64;
        if diff > max_diff {
            max_diff = diff;
        }
    }

    // Doesn't really worse when we use them, means no optimisation happens here
    vec.push(88);
    let _ = vec.get(20);

    let average: u64 = accumulator / (samples as u64);
    let max: u64 = max_diff as u64;
    println!(
        "Average Time {:?} µs Maximal {:?} µs",
        average as f64 / 1000f64,
        max as f64 / 1000f64
    );
    Ok(())
}

fn benchmark_small_box(samples: u32) -> Result<(), Box<dyn Error>> {
    println!("Running box benchmark with {} samples", samples);
    let mut diff: i64;
    let mut accumulator: u64 = 0;
    let mut max_diff: i64 = 0;

    for _s in 0..samples {
        let start = clock_gettime();

        let mybox = Box::new(88); // Just a simple box for now

        let end = clock_gettime();
        diff = Timespec::diff_ns(start, end);

        let _ = *mybox;

        accumulator += diff as u64;
        if diff > max_diff {
            max_diff = diff;
        }
    }

    let average: u64 = accumulator / (samples as u64);
    let max: u64 = max_diff as u64;
    println!(
        "Average Time {:?} µs Maximal {:?} µs",
        average as f64 / 1000f64,
        max as f64 / 1000f64
    );
    Ok(())
}
