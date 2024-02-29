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
        assert_eq!(result, 4);
    }

    #[test]
    #[ignore]
    fn it_works_2() {
        let result = add(2, 2);
        assert_eq!(result, 5);
    }

    #[test]
    #[ignore]
    fn it_works_3() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn it_works_4() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn it_works_5() {
        let result = add(2, 2);
        assert_eq!(result, 5);
    }

    #[test]
    #[ignore]
    fn it_works_6() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn it_works_7() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn it_works_8() {
        let result = add(2, 2);
        assert_eq!(result, 5);
    }

    #[test]
    #[ignore]
    fn it_works_9() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
