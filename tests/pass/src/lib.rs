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
    test_harness()
}

#[cfg(test)]
mod tests {
    use super::add;
    use gba_test::test;

    #[test]
    fn basic_add() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn chained_add() {
        let result = add(2, 2);
        let result2 = add(result, 2);
        assert_eq!(result2, 6);
    }

    #[test]
    fn zero() {
        let result = add(2, 1);
        assert_eq!(result, 2);
    }

    #[test]
    fn zeros() {
        let result = add(0, 0);
        assert_eq!(result, 0);
    }

    #[test]
    #[should_panic]
    fn overflow() {
        let result = add(usize::MAX, usize::MAX);
    }
}
