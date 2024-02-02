use std::error::Error;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = false)]
    nanosleep: bool,

    #[arg(long, default_value_t = false)]
    nanosleepgettime: bool,
}

pub fn run_with_nanosleep() -> Result<(), Box<dyn Error>> {
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
