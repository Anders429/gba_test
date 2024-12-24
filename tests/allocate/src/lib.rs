//! Defines a single basic test.

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(gba_test::runner)]
#![reexport_test_harness_main = "test_harness"]

extern crate alloc;

#[cfg(test)]
#[no_mangle]
pub fn main() {
    test_harness();
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use gba_test::test;

    #[test]
    fn it_works() {
        let result = (0..10).collect::<Vec<_>>();
        assert_eq!(result, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }
}
