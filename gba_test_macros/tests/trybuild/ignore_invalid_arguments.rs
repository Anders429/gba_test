#![feature(custom_test_frameworks)]

use gba_test_macros::test;

#[test]
#[ignore("foo")]
fn foo() {}


fn main() {}
