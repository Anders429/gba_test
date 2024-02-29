//! Defines a single basic test.

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(gba_test::runner)]
#![reexport_test_harness_main = "test_harness"]

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
#[no_mangle]
pub fn main() {
    test_harness();
    loop {}
}

#[cfg(test)]
mod tests {
    use super::add;
    use gba_test::test;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 5);
    }
}
