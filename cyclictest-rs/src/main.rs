//use cyclictest_rs;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    cyclictest_rs::cyclictest_main()?;
    Ok(())
}
