use std::error::Error;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::mem;
use std::thread;
use std::time::{Duration, Instant};

use clap::Parser;
use errno::errno;

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

observe dma setting

    sudo cat /dev/cpu_dma_latency

To-Do
* Check if we can replace calls with the nix crate https://crates.io/crates/nix


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

fn setaffinity(cpu: u64) -> Result<(), Box<dyn Error>> {
    // https://linux.die.net/man/2/sched_setaffinity
    // https://docs.rs/libc/0.2.153/libc/fn.sched_setaffinity.html
    let pid = 0;
    let cpusetsize: libc::size_t = libc::CPU_SETSIZE as libc::size_t;
    let mut cpuset: libc::cpu_set_t = unsafe { mem::zeroed() };
    unsafe { libc::CPU_ZERO(&mut cpuset) };
    let pmask: *mut libc::cpu_set_t = &mut cpuset;
    unsafe { libc::CPU_SET(usize::try_from(cpu).unwrap(), &mut cpuset) };
    match unsafe { libc::CPU_ISSET(usize::try_from(cpu).unwrap(), &cpuset) } {
        true => (),
        false => return Err("CPU_ISSET fails".into()),
    }
    match unsafe { libc::sched_setaffinity(pid, cpusetsize, pmask) } {
        0 => (),
        _ => return Err("setaffinity fails".into()),
    }
    Ok(())
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

fn mlockall() -> Result<(), Box<dyn Error>> {
    // https://linux.die.net/man/3/mlockall
    // mlockall(MCL_CURRENT|MCL_FUTURE) == -1) {

    let flags: libc::c_int = libc::MCL_CURRENT | libc::MCL_FUTURE;

    match unsafe { libc::mlockall(flags) } {
        0 => {}
        -1 => {
            let e = errno();
            let code = e.0;
            println!("Error {}: {}", code, e);
            return Err("Mlocall fails".into());
        }
        _ => return Err("Mlocall fails strangely".into()),
    }

    Ok(())
}

/* Latency trick, see cyclictest*/
fn set_latency_target() -> Result<File, Box<dyn Error>> {
    let filename = String::from("/dev/cpu_dma_latency");

    // plain open did not work out
    //let mut f = File::open(filename)?;
    let mut f = OpenOptions::new().write(true).open(filename)?;

    f.write_all(&[0, 0, 0, 0])?;
    //f.set_len(4)?; // did not work out on the 6.10 Kernel

    //f.flush(); // never helped
    //f.sync_all(); // never helped
    //f.sync_data(); // never helped
    Ok(f)
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

fn clock_gettime() -> i64 {
    // https://docs.rs/libc/0.2.153/libc/fn.clock_gettime.html

    let mut timespec = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    let clockid: libc::clockid_t = libc::CLOCK_MONOTONIC;
    let ret;

    unsafe { ret = libc::clock_gettime(clockid, &mut timespec) }
    if ret != 0 {
        println!("clock_gettime fails");
    }
    timespec.tv_nsec
}

fn sample_clock_nanosleep_with_gettime(
    samples: u32,
    wait_time_ns: u32,
) -> Result<(), Box<dyn Error>> {
    let sleep_time: u64 = wait_time_ns as u64;
    let mut diff: i64 = 0;
    let mut accumulator: u64 = 0; //probably not the right value here
    let mut max_diff: i64 = 0;

    for _s in 0..samples {
        let start: i64 = clock_gettime();
        sleep_clock_nanosleep();
        let end: i64 = clock_gettime();
        diff = end - start;

        if diff < 0 {
            // hack for now
            diff += 1_000_000_000;
        }

        accumulator += diff as u64;
        if diff > max_diff {
            max_diff = diff;
        }
    }
    let average_latency: u64 = accumulator / (samples as u64) - sleep_time;
    let max_latency: u64 = max_diff as u64 - sleep_time;
    println!(
        "Average Latency {:?} Maximal Latency {:?}",
        average_latency as f64 / 1000f64,
        max_latency as f64 / 1000f64
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
    mlockall()?;
    setscheduler()?;
    setaffinity(1)?;
    block_alarm();

    let _file = set_latency_target()?;

    for _i in 0..10 {
        sample_clock_nanosleep_with_duration(1000, 1_000_000)?;
    }

    Ok(())
}

pub fn run_with_nanosleep_gettime() -> Result<(), Box<dyn Error>> {
    mlockall()?;
    setscheduler()?;
    setaffinity(1)?;
    block_alarm();

    let _file = set_latency_target()?;

    for _i in 0..10 {
        sample_clock_nanosleep_with_gettime(1000, 1_000_000)?;
    }

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
    fn test_setaffinity() -> Result<(), Box<dyn Error>> {
        setaffinity(0)?;
        setaffinity(1)?;
        setaffinity(2)?;
        setaffinity(3)?;
        Ok(())
    }

    #[test]
    fn test_setaffinity_fail() -> Result<(), Box<dyn Error>> {
        // TODO This doesn't look nice
        let cpu = 99; // Will fail unless we have many cpus :)
        match setaffinity(cpu) {
            Ok(_) => Err("This should fail".into()),
            Err(_) => Ok(()),
        }
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
