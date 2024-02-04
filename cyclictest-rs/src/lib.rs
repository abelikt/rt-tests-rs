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

pub fn setaffinity(cpu: u64) -> Result<(), Box<dyn Error>> {
    //! Set process affinity to given cpu
    // https://linux.die.net/man/2/sched_setaffinity
    // https://docs.rs/libc/0.2.153/libc/fn.sched_setaffinity.html
    println!("Setting CPU affinity");
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
        _ => {
            let code = errno();
            return Err(format!("setaffinity fails: {}, {}", code, code.0).into());
        }
    }
    Ok(())
}

pub fn getscheduler() -> Result<&'static str, Box<dyn Error>> {
    //! Get current scheduling policy
    // https://linux.die.net/man/2/sched_getscheduler
    let policy = match unsafe { libc::sched_getscheduler(0) } {
        libc::SCHED_OTHER => "SCHED_OTHER",
        libc::SCHED_IDLE => "SCHED_IDLE",
        libc::SCHED_FIFO => "SCHED_FIFO",
        libc::SCHED_RR => "SCHED_RR",
        _ => return Err("Unexpected policy".into()),
    };
    println!("Getscheduler reports: {}", policy);
    Ok(policy)
}

pub fn block_alarm() -> Result<(), &'static str> {
    //! Block SIGALRM signal

    //sigemptyset(&sigset);
    //sigaddset(&sigset, signum);
    //sigprocmask (SIG_BLOCK, &sigset, NULL);

    // https://manpages.debian.org/bookworm/manpages-dev/alarm.2.en.html
    // https://manpages.debian.org/bookworm/manpages-dev/signal.2.en.html
    // https://manpages.debian.org/bookworm/manpages-dev/sigprocmask.2.en.html

    //https://docs.rs/libc/0.2.153/libc/fn.sigemptyset.html

    println!("Blocking Unix signals");
    let mut ret;
    let mut sigset: libc::sigset_t = unsafe { mem::zeroed() };

    // passing libc::PT_NULL did not work
    let mut oldsigset: libc::sigset_t = unsafe { mem::zeroed() };

    unsafe {
        ret = libc::sigemptyset(&mut sigset);
    }
    if ret != 0 {
        return Err("sigemptyset fails");
    }

    unsafe {
        ret = libc::sigaddset(&mut sigset, libc::SIGALRM);
    }
    if ret != 0 {
        return Err("sigaddset fails");
    }

    unsafe {
        ret = libc::sigprocmask(libc::SIG_BLOCK, &sigset, &mut oldsigset);
    }
    if ret != 0 {
        return Err("sigaddset fails");
    };
    Ok(())
}

fn mlockall() -> Result<(), Box<dyn Error>> {
    //! Lock all current and future memory pages
    // https://linux.die.net/man/3/mlockall
    // https://docs.rs/libc/latest/libc/fn.mlockall.html
    // TODO Maybe replace with nix version https://docs.rs/nix/0.27.1/nix/sys/mman/fn.mlockall.html

    println!("Locking memory");

    let flags: libc::c_int = libc::MCL_CURRENT | libc::MCL_FUTURE;
    match unsafe { libc::mlockall(flags) } {
        0 => Ok(()),
        -1 => {
            let e = errno();
            let code = e.0;
            println!("Error {}: {}", code, e);
            Err("Mlocall fails".into())
        }
        _ => Err("Mlocall fails strangely".into()),
    }
}

/* Latency trick, see cyclictest*/
fn set_latency_target() -> Result<File, Box<dyn Error>> {
    println!("Disabling power management");
    let filename = String::from("/dev/cpu_dma_latency");

    // plain open did not work out
    //let mut f = File::open(filename)?;
    let mut f = OpenOptions::new().write(true).open(filename)?;

    f.write_all(&[0, 0, 0, 0])?;
    //f.set_len(4)?; // did not work out on the 6.10 Kernel

    Ok(f)
}

#[derive(Debug)]
#[allow(dead_code)]
enum Policy {
    Other = libc::SCHED_OTHER as isize,
    Fifo = libc::SCHED_FIFO as isize,
    Rr = libc::SCHED_RR as isize,
    Idle = libc::SCHED_IDLE as isize,
}

fn setscheduler(prio: i32, policy: Policy) -> Result<(), Box<dyn Error>> {
    //! Set our prority, will fail if we request a real time prio and policy
    //! without root rights.
    //
    // https://linux.die.net/man/2/sched_setscheduler
    // https://docs.rs/libc/0.2.153/libc/fn.sched_setscheduler.html

    getscheduler()?;
    println!("Setting policy to {:?} and prio to {}", policy, prio);
    let pid: libc::c_int = 0;
    let libcpolicy = policy as libc::c_int;
    let params = libc::sched_param {
        sched_priority: prio,
    };

    match unsafe { libc::sched_setscheduler(pid, libcpolicy, &params) } {
        0 => (),
        _ => {
            let e = errno();
            let code = e.0;
            println!("Error {}: {}", code, e);
            return Err("sched_setscheduler fails".into());
        }
    }

    getscheduler()?;

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

struct Timespec {
    sec: i64,
    nsec: i64,
}

impl Timespec {
    pub fn diff_ns(begin: Timespec, end: Timespec) -> i64 {
        //! Returns the difference of end - begin in nanoseconds
        let diff_s = (end.sec - begin.sec) * 1_000_000_000;
        end.nsec - begin.nsec + diff_s
    }
}

fn clock_gettime() -> Timespec {
    // https://docs.rs/libc/0.2.153/libc/fn.clock_gettime.html

    let mut timespec = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    let clockid: libc::clockid_t = libc::CLOCK_MONOTONIC;
    let ret;

    unsafe { ret = libc::clock_gettime(clockid, &mut timespec) }
    if ret != 0 {
        panic!("clock_gettime fails");
    }
    Timespec {
        sec: timespec.tv_sec,
        nsec: timespec.tv_nsec,
    }
}

fn sample_clock_nanosleep_with_gettime(
    samples: u32,
    wait_time_ns: u32,
) -> Result<(), Box<dyn Error>> {
    let sleep_time: u64 = wait_time_ns as u64;
    let mut diff: i64;
    let mut accumulator: u64 = 0; //probably not the right value here
    let mut max_diff: i64 = 0;

    for _s in 0..samples {
        let start = clock_gettime();
        sleep_clock_nanosleep();
        let end = clock_gettime();
        diff = Timespec::diff_ns(start, end);

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
    println!("Starting measurement cycle ...");
    for _i in 0..10 {
        sample_sleep_with_duration(1000, 1_000_000)?;
    }
    Ok(())
}

pub fn run_with_nanosleep() -> Result<(), Box<dyn Error>> {
    mlockall()?;
    setscheduler(99, Policy::Fifo)?;
    setaffinity(1)?;
    block_alarm()?;

    // We need to keep the file open to disable power management
    let _file = set_latency_target()?;

    println!("Starting measurement cycle ...");
    for _i in 0..10 {
        sample_clock_nanosleep_with_duration(1000, 1_000_000)?;
    }

    Ok(())
}

pub fn run_with_nanosleep_gettime() -> Result<(), Box<dyn Error>> {
    mlockall()?;
    setscheduler(99, Policy::Fifo)?;
    setaffinity(1)?;
    block_alarm()?;

    // We need to keep the file open to disable power management
    let _file = set_latency_target()?;

    println!("Starting measurement cycle ...");
    for _i in 0..10 {
        sample_clock_nanosleep_with_gettime(1000, 1_000_000)?;
    }

    Ok(())
}

pub fn cyclictest_main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    if args.sleep {
        println!("Testing with simple sleep");
        run_with_sleep()?;
    }

    if args.nanosleep {
        println!("Testing with clock_nanosleep");
        run_with_nanosleep()?;
    }

    if args.nanosleepgettime {
        println!("Testing with clock_nanosleep and clock_gettime");
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
        // TODO Can we also check the error message?
        let cpu = 99; // Will fail unless we have many cpus :)
        match setaffinity(cpu) {
            Ok(_) => Err("This should fail".into()),
            Err(_) => Ok(()),
        }
    }

    #[test]
    fn test_sched_getscheduler() -> Result<(), Box<dyn Error>> {
        // in a non rt context we expect libc::SCHED_OTHER aka 0
        assert_eq!(getscheduler().unwrap(), "SCHED_OTHER");
        Ok(())
    }

    #[test]
    fn test_block_alarm() -> Result<(), &'static str> {
        block_alarm()
    }

    #[test]
    fn test_mlockall() -> Result<(), Box<dyn Error>> {
        mlockall()
    }

    #[test]
    fn test_set_latency_target() -> Result<(), Box<dyn Error>> {
        let _file = set_latency_target();
        Ok(())
    }

    // TODO all scheduler tests fail if they are in the same test
    //     could be related on how often we call this and into how
    //     many libraries the tests are compiled.
    #[test]
    fn test_setscheduler_idle() -> Result<(), Box<dyn Error>> {
        setscheduler(0, Policy::Idle)?;
        Ok(())
    }
    #[test]
    fn test_setscheduler_other() -> Result<(), Box<dyn Error>> {
        setscheduler(0, Policy::Other)?;
        Ok(())
    }

    #[test]
    fn test_setscheduler_fifo() -> Result<(), Box<dyn Error>> {
        match setscheduler(99, Policy::Fifo) {
            Ok(()) => return Err("Should fail".into()),
            Err(_) => Ok(()),
        }
    }

    #[test]
    fn test_setscheduler_rr() -> Result<(), Box<dyn Error>> {
        match setscheduler(99, Policy::Rr) {
            Ok(()) => return Err("Should fail".into()),
            Err(_) => Ok(()),
        }
    }

    #[test]
    fn test_clock_gettime() {
        let begin = clock_gettime();
        let end = clock_gettime();
        assert!(Timespec::diff_ns(begin, end) > 0);
    }

    #[test]
    fn test_diff_larger() {
        let begin = Timespec { sec: 0, nsec: 10 };
        let end = Timespec { sec: 0, nsec: 20 };
        assert_eq!(Timespec::diff_ns(begin, end), 10);
    }
    #[test]
    fn test_diff_smaller() {
        let begin = Timespec { sec: 0, nsec: 20 };
        let end = Timespec { sec: 0, nsec: 10 };
        assert_eq!(Timespec::diff_ns(begin, end), -10);
    }
    #[test]
    fn test_diff_1s() {
        let begin = Timespec { sec: 0, nsec: 10 };
        let end = Timespec { sec: 1, nsec: 20 };
        assert_eq!(Timespec::diff_ns(begin, end), 1_000_000_010);
    }
    #[test]
    fn test_diff_smaller_1s() {
        let begin = Timespec { sec: 0, nsec: 20 };
        let end = Timespec { sec: 1, nsec: 10 };
        assert_eq!(Timespec::diff_ns(begin, end), 999_999_990);
    }
    #[test]
    fn test_diff_smaller_s_overflow() {
        let begin = Timespec {
            sec: 0,
            nsec: 999_999_990,
        };
        let end = Timespec { sec: 1, nsec: 10 };
        assert_eq!(Timespec::diff_ns(begin, end), 20);
    }

    // Sleep tests

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
