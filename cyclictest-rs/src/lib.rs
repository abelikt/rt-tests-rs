pub fn cyclictest_main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod test {

    #[test]
    pub fn test1() {
        assert_eq!(1, 1);
    }
}
