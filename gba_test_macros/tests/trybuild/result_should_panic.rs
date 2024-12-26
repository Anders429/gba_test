#![feature(custom_test_frameworks)]

use gba_test_macros::test;

#[test]
#[should_panic]
fn foo() -> Result<(), ()> {
    Ok(())
}


fn main() {}
