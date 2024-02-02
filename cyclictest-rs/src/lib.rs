use std::error::Error;
use std::thread;
use std::time::Duration;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = false)]
    nanosleep: bool,

    #[arg(long, default_value_t = false)]
    nanosleepgettime: bool,
}

fn sample_clock_nanosleep_with_duration(
    samples: u32,
    wait_time_ns: u32,
) -> Result<(), Box<dyn Error>> {
    let sleep_time = Duration::from_nanos(wait_time_ns.into());

    for _s in 0..samples {
        thread::sleep(sleep_time);
    }

    Ok(())
}

pub fn run_with_nanosleep() -> Result<(), Box<dyn Error>> {
    for _i in 0..10 {
        sample_clock_nanosleep_with_duration(1000, 1000)?;
    }
    Ok(())
}

pub fn run_with_nanosleep_gettime() -> Result<(), Box<dyn Error>> {
    Ok(())
}

pub fn cyclictest_main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

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

    #[test]
    pub fn test1() {
        assert_eq!(1, 1);
    }
}
