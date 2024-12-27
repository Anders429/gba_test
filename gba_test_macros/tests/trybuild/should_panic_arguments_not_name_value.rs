#![feature(custom_test_frameworks)]

use gba_test_macros::test;

#[test]
#[should_panic("foo")]
fn foo() {}


fn main() {}
