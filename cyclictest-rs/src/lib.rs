use std::error::Error;
use std::thread;
use std::time::{Duration, Instant};

use clap::Parser;

/*

libc:
https://docs.rs/libc/
https://linux.die.net/man/2/clock_nanosleep
https://manpages.debian.org/bookworm/manpages-dev/clock_nanosleep.2.en.html

*/


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = false)]
    sleep: bool,

    #[arg(long, default_value_t = false)]
    nanosleep: bool,

    #[arg(long, default_value_t = false)]
    nanosleepgettime: bool,
}

fn sleep_clock_nanosleep() {
    //let clockid : libc::clockid_t = libc::CLOCK_REALTIME;
    let clockid: libc::clockid_t = libc::CLOCK_MONOTONIC;

    let flags: libc::c_int = libc::CLOCK_REALTIME; // 1: libc::TIMER_ABSTIME
    let request = libc::timespec {
        tv_sec: 0,
        tv_nsec: 1_000_000,
    };
    let mut remain = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    let premain: *mut libc::timespec = &mut remain;
    let ret;
    unsafe {
        ret = libc::clock_nanosleep(clockid, flags, &request, premain);
    }
    if ret != 0 {
        println!("clock_nanosleep fails");
    }
}

fn sample_sleep_with_duration(samples: u32, wait_time_ns: u32) -> Result<(), Box<dyn Error>> {
    let sleep_time = Duration::from_nanos(wait_time_ns.into());
    let mut diff: Duration;
    let mut accumulator: Duration = Duration::new(0, 0);
    let mut max_diff: Duration = Duration::new(0, 0);
    for _s in 0..samples {
        let start = Instant::now();
        thread::sleep(sleep_time);
        let end = Instant::now();
        diff = end - start;
        accumulator += diff;
        if diff > max_diff {
            max_diff = diff;
        }
    }
    let average_latency = accumulator / samples - sleep_time;
    let max_latency = max_diff - sleep_time;
    println!(
        "Average Latency {:?} Maximal Latency {:?}",
        average_latency, max_latency
    );
    Ok(())
}

fn sample_clock_nanosleep_with_duration(
    samples: u32,
    wait_time_ns: u32,
) -> Result<(), Box<dyn Error>> {
    let sleep_time = Duration::from_nanos(wait_time_ns.into());
    let mut diff: Duration;
    let mut accumulator: Duration = Duration::new(0, 0);
    let mut max_diff: Duration = Duration::new(0, 0);
    for _s in 0..samples {
        let start = Instant::now();
        sleep_clock_nanosleep();
        let end = Instant::now();
        diff = end - start;
        accumulator += diff;
        if diff > max_diff {
            max_diff = diff;
        }
    }
    let average_latency = accumulator / samples - sleep_time;
    let max_latency = max_diff - sleep_time;
    println!(
        "Average Latency {:?} Maximal Latency {:?}",
        average_latency, max_latency
    );
    Ok(())
}

pub fn run_with_sleep() -> Result<(), Box<dyn Error>> {
    for _i in 0..10 {
        sample_sleep_with_duration(1000, 1_000_000)?;
    }
    Ok(())
}

pub fn run_with_nanosleep() -> Result<(), Box<dyn Error>> {
    for _i in 0..10 {
        sample_clock_nanosleep_with_duration(1000, 1_000_000)?;
    }
    Ok(())
}

pub fn run_with_nanosleep_gettime() -> Result<(), Box<dyn Error>> {
    Ok(())
}

pub fn cyclictest_main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    if args.sleep {
        println!("simple sleep");
        run_with_sleep()?;
    }

    if args.nanosleep {
        println!("clock_nanosleep");
        run_with_nanosleep()?;
    }

    if args.nanosleepgettime {
        println!("clock_nanosleep clock_gettime");
        run_with_nanosleep_gettime()?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test1() {
        assert_eq!(1, 1);
    }

    #[test]
    fn test_sample_sleep_with_duration() -> Result<(), Box<dyn Error>> {
        sample_sleep_with_duration(1, 1)?;
        Ok(())
    }

    #[test]
    fn test_sample_clock_nanosleep_with_duration() -> Result<(), Box<dyn Error>> {
        sample_clock_nanosleep_with_duration(1, 1)?;
        Ok(())
    }
}
