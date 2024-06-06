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
///
/// The easiest way to define if a test should be ignored is to use the `#[ignore]` attribute when
/// defining a test.
///
/// ```
/// #[cfg(test)]
/// mod tests {
///     use gba_test_macros::test;
///
///     #[test]
///     #[ignore]
///     fn ignored_test() {
///         assert!(false);
///     }
/// }
/// ```
#[derive(Clone, Copy, Debug)]
pub enum Ignore {
    /// The test should be run.
    No,
    /// The test should not be run.
    Yes,
    /// The test should not be run, and a message should be displayed.
    YesWithMessage(&'static str),
}

/// Whether a test is expected to panic.
#[derive(Clone, Copy, Debug)]
pub enum ShouldPanic {
    /// The test is expected to run successfully.
    No,
    /// The test is expected to panic during execution.
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
    /// If this method returns `Ignore::Yes`, the test function will not be run at all (but it will
    /// still be compiled). This allows for time-consuming or expensive tests to be conditionally
    /// disabled.
    fn ignore(&self) -> Ignore;

    /// Whether the test is expected to panic.
    fn should_panic(&self) -> ShouldPanic;

    /// Returns the ignore message, if it exists.
    fn message(&self) -> Option<&'static str>;
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
    /// Whether the test is expected to panic.
    ///
    /// This is set by the `#[should_panic]` attribute.
    pub should_panic: ShouldPanic,
}

impl TestCase for Test {
    fn name(&self) -> &str {
        if let Some((_, path)) = self.name.split_once("::") {
            path
        } else {
            self.name
        }
    }

    fn run(&self) {
        (self.test)()
    }

    fn ignore(&self) -> Ignore {
        self.ignore
    }

    fn should_panic(&self) -> ShouldPanic {
        self.should_panic
    }

    fn message(&self) -> Option<&'static str> {
        if let Ignore::YesWithMessage(message) = self.ignore {
            Some(message)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Ignore, ShouldPanic, Test, TestCase};
    use claims::{assert_matches, assert_none, assert_some_eq};
    use gba_test_macros::test;

    #[test]
    fn test_name_split() {
        let test = Test {
            name: "foo::bar",
            test: || {},
            ignore: Ignore::No,
            should_panic: ShouldPanic::No,
        };

        assert_eq!(test.name(), "bar");
    }

    #[test]
    fn test_name_no_split() {
        let test = Test {
            name: "foo",
            test: || {},
            ignore: Ignore::No,
            should_panic: ShouldPanic::No,
        };

        assert_eq!(test.name(), "foo");
    }

    #[test]
    fn test_run_no_panic() {
        let test = Test {
            name: "",
            test: || {
                assert!(true);
            },
            ignore: Ignore::No,
            should_panic: ShouldPanic::No,
        };

        test.run();
    }

    #[test]
    #[should_panic(expectd = "assertion failed: false")]
    fn test_run_panic() {
        let test = Test {
            name: "",
            test: || {
                assert!(false);
            },
            ignore: Ignore::No,
            should_panic: ShouldPanic::No,
        };

        test.run();
    }

    #[test]
    fn test_ignore() {
        let test = Test {
            name: "",
            test: || {},
            ignore: Ignore::Yes,
            should_panic: ShouldPanic::No,
        };

        assert_matches!(test.ignore(), Ignore::Yes);
    }

    #[test]
    fn test_should_panic() {
        let test = Test {
            name: "",
            test: || {},
            ignore: Ignore::No,
            should_panic: ShouldPanic::Yes,
        };

        assert_matches!(test.should_panic(), ShouldPanic::Yes);
    }

    #[test]
    fn test_message() {
        let test = Test {
            name: "",
            test: || {},
            ignore: Ignore::YesWithMessage("foo"),
            should_panic: ShouldPanic::No,
        };

        assert_some_eq!(test.message(), "foo");
    }

    #[test]
    fn test_no_message() {
        let test = Test {
            name: "",
            test: || {},
            ignore: Ignore::Yes,
            should_panic: ShouldPanic::No,
        };

        assert_none!(test.message());
    }
}
