//! Defines a single basic test.

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(gba_test::runner)]
#![reexport_test_harness_main = "test_harness"]

#[cfg(test)]
#[no_mangle]
pub fn main() {
    test_harness()
}

#[cfg(test)]
mod tests {
    use gba_test::test;

    #[test]
    fn it_works() -> Result<(), ()> {
        Err(())
    }
}
