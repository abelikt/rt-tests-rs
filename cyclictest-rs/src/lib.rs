use std::error::Error;
use std::mem;
use std::thread;
use std::time::{Duration, Instant};

use clap::Parser;

/*

libc:
https://docs.rs/libc/
https://linux.die.net/man/2/clock_nanosleep
https://manpages.debian.org/bookworm/manpages-dev/clock_nanosleep.2.en.html

Run With:

    cargo build && sudo target/debug/cyclictest-rs  --nanosleep
    cargo build --release && sudo target/release/cyclictest-rs  --nanosleep

Observer real-time prio with:

    ps  -m -C cyclictest-rs -o pid,pri,rtprio,uid,cputime,cmd

    unclear why there is no prio displayed in ps
    for 6.1.0-17-rt-amd64

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

fn setaffinity() {
    // https://linux.die.net/man/2/sched_setaffinity
    // https://docs.rs/libc/0.2.153/libc/fn.sched_setaffinity.html

    // -> https://crates.io/crates/nix
    // https://docs.rs/nix/latest/src/nix/sched.rs.html#181-186
    // https://doc.rust-lang.org/std/mem/fn.zeroed.html

    let ret;
    let pid = 0;
    let cpusetsize: libc::size_t = 12;

    let mut cpuset: libc::cpu_set_t = unsafe { mem::zeroed() };

    unsafe { libc::CPU_ZERO(&mut cpuset) };

    let pmask: *mut libc::cpu_set_t = &mut cpuset;

    unsafe { libc::CPU_SET(1, &mut cpuset) };

    let isset: bool;
    unsafe { isset = libc::CPU_ISSET(1, &cpuset) };

    if !isset {
        println!("CPU_ISSET fails");
    }

    unsafe {
        ret = libc::sched_setaffinity(pid, cpusetsize, pmask);
    }

    if ret != 0 {
        println!("setaffinity fails");
    }
}

fn getscheduler() {
    let ret;
    unsafe {
        ret = libc::sched_getscheduler(0);
    }
    println!("Getscheduler {}", ret);
}

fn block_alarm() {
    //sigemptyset(&sigset);
    //sigaddset(&sigset, signum);
    //sigprocmask (SIG_BLOCK, &sigset, NULL);

    // https://manpages.debian.org/bookworm/manpages-dev/alarm.2.en.html
    // https://manpages.debian.org/bookworm/manpages-dev/signal.2.en.html
    // https://manpages.debian.org/bookworm/manpages-dev/sigprocmask.2.en.html

    //https://docs.rs/libc/0.2.153/libc/fn.sigemptyset.html

    let mut ret;
    let mut sigset: libc::sigset_t = unsafe { mem::zeroed() };

    // passing libc::PT_NULL did not work
    let mut oldsigset: libc::sigset_t = unsafe { mem::zeroed() };

    unsafe {
        ret = libc::sigemptyset(&mut sigset);
    }
    if ret != 0 {
        println!("sigemptyset fails");
    }

    unsafe {
        ret = libc::sigaddset(&mut sigset, libc::SIGALRM);
    }
    if ret != 0 {
        println!("sigaddset fails");
    }

    unsafe {
        ret = libc::sigprocmask(libc::SIG_BLOCK, &sigset, &mut oldsigset);
    }
    if ret != 0 {
        println!("sigaddset fails");
    }
}


fn setscheduler() -> Result<(), Box<dyn Error>> {
    // https://linux.die.net/man/2/sched_setscheduler
    // https://crates.io/crates/scheduler
    // https://docs.rs/scheduler/0.1.3/scheduler/
    // https://github.com/terminalcloud/rust-scheduler

    // https://docs.rs/libc/0.2.153/libc/fn.sched_setscheduler.html

    getscheduler();

    let prio = 99;
    let pid: libc::c_int = 0;
    let policy: libc::c_int = libc::SCHED_FIFO;
    let params = libc::sched_param {
        sched_priority: prio,
    };
    let ret;
    unsafe {
        ret = libc::sched_setscheduler(pid, policy, &params);
    }

    if ret != 0 {
        println!("sched_setscheduler fails");
    }

    getscheduler();

    Ok(())
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

    setscheduler()?;
    setaffinity();
    block_alarm();

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
