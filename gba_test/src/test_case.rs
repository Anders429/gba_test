//! All types related to defining a test.
//!
//! This module provides the [`TestCase`] trait and associated types, allowing the user to define a
//! test to be run by the test [`runner`]. It also provides a [`Test`] struct, which is used by the
//! [`test`] macro to create a default implementer of the `TestCase` trait. Note that the `Test`
//! struct is not considered part of the public API.
//!
//! [`runner`]: crate::runner()
//! [`test`]: crate::test

use crate::Termination;
use core::{mem::MaybeUninit, str};

/// Defines whether a test should be ignored or not.
///
/// The easiest way to define if a test should be ignored is to use the `#[ignore]` attribute when
/// defining the test.
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
///
/// The easiest way to define a test that should panic is to use the `#[should_panic]` attribute
/// when defining the test.
///
/// ```
/// #[cfg(test)]
/// mod tests {
///     use gba_test_macros::test;
///
///     #[test]
///     #[should_panic]
///     fn ignored_test() {
///         panic!("something was expected to go wrong");
///     }
/// }
/// ```
#[derive(Clone, Copy, Debug)]
pub enum ShouldPanic {
    /// The test is expected to run successfully.
    No,
    /// The test is expected to panic during execution.
    Yes,
    /// The test is expected to panic with the given substring present in the panic message.
    YesWithMessage(&'static str),
}

/// Defines a test case executable by the test runner.
///
/// Any type implementing this trait can be passed to the test runner using the `#[test_case]`
/// attribute. For most cases, using the `#[test]` attribute provided by this crate is sufficient.
///
/// See the [`custom_test_frameworks`](https://doc.rust-lang.org/unstable-book/language-features/custom-test-frameworks.html)
/// language feature for more information about using the `#[test_case]` attribute.
pub trait TestCase {
    /// The name of the test.
    fn name(&self) -> &str;

    /// The module the test is in.
    fn modules(&self) -> &[&str];

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

/// Determines the amount of module sections in a given module path.
///
/// This function is used by the `#[test]` attribute. It is not considered a part of the public API.
#[doc(hidden)]
pub const fn split_module_path_len(module_path: &'static str) -> usize {
    let mut len = 1;

    let mut i = 1;
    while i < module_path.len() {
        if module_path.as_bytes()[i - 1] == b':' && module_path.as_bytes()[i] == b':' {
            len += 1;
            i += 1;
        }
        i += 1;
    }

    len
}

/// Splits a module path into its individual parts.
///
/// This function is used by the `#[test]` attribute. It is not considered a part of the public API.
#[doc(hidden)]
pub const fn split_module_path<const LEN: usize>(module_path: &'static str) -> [&'static str; LEN] {
    let mut result: MaybeUninit<[&'static str; LEN]> = MaybeUninit::uninit();
    let mut result_index = 0;
    let mut module_path_start = 0;
    // Look at two bytes at a time.
    let mut module_path_index = 1;
    while module_path_index < module_path.len() {
        if module_path.as_bytes()[module_path_index - 1] == b':'
            && module_path.as_bytes()[module_path_index] == b':'
        {
            let module = unsafe {
                str::from_utf8_unchecked(core::slice::from_raw_parts(
                    module_path.as_ptr().add(module_path_start),
                    module_path_index - 1 - module_path_start,
                ))
            };
            // Check that we have not already filled in the full result.
            if result_index >= LEN {
                panic!("module path was split into too many parts")
            }
            unsafe {
                (result.as_mut_ptr() as *mut &str)
                    .add(result_index)
                    .write(module);
            }
            result_index += 1;
            module_path_index += 1;
            module_path_start = module_path_index;
        }
        module_path_index += 1;
    }
    // Add the final path.
    let module = unsafe {
        str::from_utf8_unchecked(core::slice::from_raw_parts(
            module_path.as_ptr().add(module_path_start),
            module_path.len() - module_path_start,
        ))
    };
    // Check that we have not already filled in the full result.
    if result_index >= LEN {
        panic!("module path was split into too many parts")
    }
    unsafe {
        (result.as_mut_ptr() as *mut &str)
            .add(result_index)
            .write(module);
    }
    result_index += 1;

    // Check that we actually filled the result.
    if result_index < LEN {
        panic!("unable to split module path into enough separate parts")
    }

    unsafe { result.assume_init() }
}

/// A standard test.
///
/// This struct is created by the `#[test]` attribute. This struct is not to be used directly and
/// is not considered part of the public API. If you want to use a similar struct, you should
/// define one locally and implement `TestCase` for it directly.
#[doc(hidden)]
pub struct Test<T> {
    /// The name of the test.
    pub name: &'static str,
    /// The modules the test is in.
    pub modules: &'static [&'static str],
    /// The test function itself.
    pub test: fn() -> T,
    /// Whether the test should be excluded.
    ///
    /// This is set by the `#[ignore]` attribute.
    pub ignore: Ignore,
    /// Whether the test is expected to panic.
    ///
    /// This is set by the `#[should_panic]` attribute.
    pub should_panic: ShouldPanic,
}

impl<T> TestCase for Test<T>
where
    T: Termination,
{
    fn name(&self) -> &str {
        self.name
    }

    fn modules(&self) -> &[&str] {
        if self.modules.len() <= 1 {
            self.modules
        } else {
            &self.modules[1..]
        }
    }

    fn run(&self) {
        (self.test)().terminate()
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
    use super::{split_module_path, split_module_path_len, Ignore, ShouldPanic, Test, TestCase};

    use claims::{assert_matches, assert_none, assert_some_eq};
    use gba_test_macros::test;

    #[test]
    fn test_name() {
        let test = Test {
            name: "foo",
            modules: &[""],
            test: || {},
            ignore: Ignore::No,
            should_panic: ShouldPanic::No,
        };

        assert_eq!(test.name(), "foo")
    }

    #[test]
    fn test_module_split() {
        let test = Test {
            name: "",
            modules: &["foo", "bar"],
            test: || {},
            ignore: Ignore::No,
            should_panic: ShouldPanic::No,
        };

        assert_eq!(test.modules(), &["bar"]);
    }

    #[test]
    fn test_module_no_split() {
        let test = Test {
            name: "",
            modules: &["foo"],
            test: || {},
            ignore: Ignore::No,
            should_panic: ShouldPanic::No,
        };

        assert_eq!(test.modules(), &["foo"]);
    }

    #[test]
    fn test_run_no_panic() {
        let test = Test {
            name: "",
            modules: &[""],
            test: || {
                assert!(true);
            },
            ignore: Ignore::No,
            should_panic: ShouldPanic::No,
        };

        test.run();
    }

    #[test]
    #[should_panic(expected = "assertion failed: false")]
    fn test_run_panic() {
        let test = Test {
            name: "",
            modules: &[""],
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
            modules: &[""],
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
            modules: &[""],
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
            modules: &[""],
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
            modules: &[""],
            test: || {},
            ignore: Ignore::Yes,
            should_panic: ShouldPanic::No,
        };

        assert_none!(test.message());
    }

    #[test]
    fn split_module_path_len_empty() {
        assert_eq!(split_module_path_len(""), 1);
    }

    #[test]
    fn split_module_path_len_single() {
        assert_eq!(split_module_path_len("foo"), 1);
    }

    #[test]
    fn split_module_path_len_single_colon() {
        assert_eq!(split_module_path_len(":"), 1);
    }

    #[test]
    fn split_module_path_len_empty_with_separator() {
        assert_eq!(split_module_path_len("::"), 2);
    }

    #[test]
    fn split_module_path_len_separator_with_extra_colon() {
        assert_eq!(split_module_path_len(":::"), 2);
    }

    #[test]
    fn split_module_path_len_modules_split_by_separator() {
        assert_eq!(split_module_path_len("foo::bar"), 2);
    }

    #[test]
    fn split_module_path_len_many_modules_split_by_separators() {
        assert_eq!(split_module_path_len("foo::bar::baz::quux"), 4);
    }

    #[test]
    fn split_module_path_len_modules_leading_separator() {
        assert_eq!(split_module_path_len("::foo::bar"), 3);
    }

    #[test]
    fn split_module_path_len_modules_trailing_separator() {
        assert_eq!(split_module_path_len("foo::bar::"), 3);
    }

    #[test]
    fn split_module_path_empty() {
        assert_eq!(split_module_path::<1>(""), [""]);
    }

    #[test]
    fn split_module_path_single() {
        assert_eq!(split_module_path::<1>("foo"), ["foo"]);
    }

    #[test]
    fn split_module_path_single_colon() {
        assert_eq!(split_module_path::<1>(":"), [":"]);
    }

    #[test]
    fn split_module_path_empty_with_separator() {
        assert_eq!(split_module_path::<2>("::"), ["", ""]);
    }

    #[test]
    fn split_module_path_separator_with_extra_colon() {
        assert_eq!(split_module_path::<2>(":::"), ["", ":"]);
    }

    #[test]
    fn split_module_path_modules_split_by_separator() {
        assert_eq!(split_module_path::<2>("foo::bar"), ["foo", "bar"]);
    }

    #[test]
    fn split_module_path_many_modules_split_by_separators() {
        assert_eq!(
            split_module_path::<4>("foo::bar::baz::quux"),
            ["foo", "bar", "baz", "quux"]
        );
    }

    #[test]
    fn split_module_path_modules_leading_separator() {
        assert_eq!(split_module_path::<3>("::foo::bar"), ["", "foo", "bar"]);
    }

    #[test]
    fn split_module_path_modules_trailing_separator() {
        assert_eq!(split_module_path::<3>("foo::bar::"), ["foo", "bar", ""]);
    }

    #[test]
    #[should_panic(expected = "module path was split into too many parts")]
    fn split_module_path_size_too_small() {
        split_module_path::<0>("foo");
    }

    #[test]
    #[should_panic(expected = "module path was split into too many parts")]
    fn split_module_path_size_too_small_multiple_parts() {
        split_module_path::<2>("foo::bar::baz");
    }

    #[test]
    #[should_panic(expected = "unable to split module path into enough separate parts")]
    fn split_module_path_size_too_large() {
        split_module_path::<2>("foo");
    }
}
