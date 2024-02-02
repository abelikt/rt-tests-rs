#[cfg(test)]
mod test {

    use std::error::Error;

    #[test]
    pub fn test_int() -> Result<(), Box<dyn Error>> {
        cyclictest_rs::cyclictest_main()?;
        assert_eq!(1, 1);
        Ok(())
    }
}
