#![feature(custom_test_frameworks)]

use gba_test_macros::test;

#[test]
#[should_panic(expectd = "foo")]
fn foo() {}


fn main() {}
