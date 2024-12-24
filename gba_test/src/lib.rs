//! Testing framework for the Game Boy Advance.
//!
//! This crate enables developers to run tests directly on the Game Boy Advance (or on a Game Boy
//!  Advance emulator). To accomplish this, the crate provides both a test macro and a test runner.
//!
//! # Test Macro
//! Instead of using the default `#[test]` macro, a custom `#[test]` macro is provided. It must be
//! used to create tests that can be run by the test runner.
//!
//! This custom `#[test]` macro supports the same testing attributes as the default macro.
//! Specifically, both `#[ignore]` and `#[should_panic]` are supported.
//!
//! In order to use the `#[test]` macro, the `macros` feature must be enabled. It is enabled by
//! default.
//!
//! ## Example
//! A very simple test can be written as follows:
//!
//! ```
//! // A very simple function to test.
//! pub fn add(left: usize, right: usize) -> usize {
//!     left + right
//! }
//!
//! #[cfg(test)]
//! mod tests {
//!     use super::add;
//!     use gba_test::test;
//!
//!     #[test]
//!     fn it_works() {
//!         let result = add(2, 2);
//!         assert_eq!(result, 4);
//!     }
//! }
//! ```
//!
//! The `#[test]` macro will pass this test to the test runner.
//!
//! # Test Runner
//! In order to run the tests you define, you must use the test runner provided by this crate. This
//! test runner is created using the unstable
//! [`custom_test_frameworks`](https://doc.rust-lang.org/unstable-book/language-features/custom-test-frameworks.html)
//! language feature.
//!
//! ## Example
//! ```
//! #![no_std]
//! #![cfg_attr(test, no_main)]
//! #![cfg_attr(test, feature(custom_test_frameworks))]
//! #![cfg_attr(test, test_runner(gba_test::runner))]
//! #![cfg_attr(test, reexport_test_harness_main = "test_harness")]
//!
//! #[cfg(test)]
//! #[no_mangle]
//! pub fn main() {
//!     test_harness()
//! }
//! ```
//!
//! This will run all tests defined within your project.
//!
//! Note that this can be done in libraries, as defining a `main()` function using `#[cfg(test)]`
//! will not cause any problems for downstream users.
//!
//! # Stability
//! This library relies the following unstable language feature:
//! - [`custom_test_frameworks`](https://doc.rust-lang.org/unstable-book/language-features/custom-test-frameworks.html)
//!
//! As such, the stability cannot be guaranteed. This feature is subject to change at any time,
//! potentially breaking this framework.

#![no_std]
#![cfg_attr(test, no_main)]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, test_runner(runner))]
#![cfg_attr(test, reexport_test_harness_main = "test_harness")]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![allow(clippy::needless_doctest_main, static_mut_refs)]

#[cfg(test)]
extern crate self as gba_test;

mod alignment;
mod allocator;
mod log;
mod runner;
mod runtime;
mod test;
mod test_case;
mod ui;

#[cfg(feature = "macros")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "macros")))]
pub use gba_test_macros::test;
pub use runner::runner;
#[doc(hidden)]
pub use test_case::Test;
pub use test_case::{Ignore, ShouldPanic, TestCase};

use test::{Outcome, Tests};

#[cfg(test)]
#[no_mangle]
pub fn main() {
    // We don't care if logging doesn't initialize, we just want to initialize it if we happen to
    // be running on mGBA.
    let _ = mgba_log::init();
    test_harness()
}
