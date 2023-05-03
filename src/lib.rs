#![no_std]

/// Defines a test case executable by the test runner.
pub trait TestCase {}

/// A test runner to execute tests as a Game Boy Advance ROM.
pub fn test_runner(tests: &'static [&'static dyn TestCase]) {
    todo!()
}
