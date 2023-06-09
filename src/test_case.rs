//! All types related to defining a test.
//!
//! This module provides the [`TestCase`] trait and associated types, allowing the user to define a
//! test to be run by the test [`runner`]. It also provides a [`Test`] struct, which is used by the
//! [`test`] macro to create a default implementer of the `TestCase` trait. Note that the `Test`
//! struct is not considered part of the public API.
//!
//! [`runner`]: crate::runner()
//! [`test`]: crate::test

/// Defines whether a test should be ignored or not.
#[derive(Clone, Copy, Debug)]
pub enum Ignore {
    /// The test should be run.
    No,
    /// The test should not be run.
    Yes,
}

/// Defines a test case executable by the test runner.
pub trait TestCase {
    /// The name of the test.
    fn name(&self) -> &str;

    /// The actual test itself.
    ///
    /// If this method panics, the test is considered a failure. Otherwise, the test is considered
    /// to have passed.
    fn run(&self);

    /// Whether the test should be excluded or not.
    ///
    /// If this method returns true, the test function will not be run at all (but it will still be
    /// compiled). This allows for time-consuming or expensive tests to be conditionally disabled.
    fn ignore(&self) -> Ignore;
}

/// A standard test.
///
/// This struct is created by the `#[test]` attribute. This struct is not to be used directly and
/// is not considered part of the public API. If you want to use a similar struct, you should
/// define one locally and implement `TestCase` for it directly.
#[doc(hidden)]
pub struct Test {
    /// The name of the test.
    pub name: &'static str,
    /// The test function itself.
    pub test: fn(),
    /// Whether the test should be excluded.
    ///
    /// This is set by the `#[ignore]` attribute.
    pub ignore: Ignore,
}

impl TestCase for Test {
    fn name(&self) -> &str {
        self.name
    }

    fn run(&self) {
        (self.test)()
    }

    fn ignore(&self) -> Ignore {
        self.ignore
    }
}
