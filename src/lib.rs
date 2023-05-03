#![no_std]

/// The remaining tests to be run.
static mut TESTS: &[&dyn TestCase] = &[];

/// Defines a test case executable by the test runner.
pub trait TestCase {
    /// The name of the test.
    fn name(&self) -> &str;

    /// The actual test itself.
    /// 
    /// If this method panics, the test is considered a failure. Otherwise, the test is considered
    /// to have passed.
    fn run(&self);
}

/// Runs the remaining tests.
/// 
/// The current test being executed is tracked using global state. This allows the runner to
/// recover when a test panics.
fn run_tests() {
    // SAFETY: `TESTS` is only ever mutated on the main thread.
    while let Some((test, tests)) = unsafe {TESTS}.split_first() {
        // SAFETY: `TESTS` is only ever mutated on the main thread.
        unsafe {
            TESTS = tests;
        }
        test.run();
    }
}

/// A test runner to execute tests as a Game Boy Advance ROM.
pub fn test_runner(tests: &'static [&'static dyn TestCase]) {
    // SAFETY: `TESTS` is only ever mutated on the main thread.
    unsafe {
        TESTS = tests;
    }

    run_tests();
}
