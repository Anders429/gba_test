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

    #[test]
    fn it_works_10() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    #[ignore]
    fn it_works_11() {
        let result = add(2, 2);
        assert_eq!(result, 5);
    }

    #[test]
    #[ignore]
    fn it_works_12() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn it_works_13() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn it_works_14() {
        let result = add(2, 2);
        assert_eq!(result, 5);
    }

    #[test]
    #[ignore]
    fn it_works_15() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn it_works_16() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn it_works_17() {
        let result = add(2, 2);
        assert_eq!(result, 5);
    }

    #[test]
    #[ignore]
    fn it_works_18() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn it_works_19() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    #[ignore]
    fn it_works_20() {
        let result = add(2, 2);
        assert_eq!(result, 5);
    }

    #[test]
    #[ignore]
    fn it_works_21() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn it_works_22() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn it_works_23() {
        let result = add(2, 2);
        assert_eq!(result, 5);
    }

    #[test]
    #[ignore]
    fn it_works_24() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn it_works_25() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn it_works_26() {
        let result = add(2, 2);
        assert_eq!(result, 5);
    }

    #[test]
    #[ignore]
    fn it_works_27() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
