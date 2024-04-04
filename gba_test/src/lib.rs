#![no_std]
#![feature(asm_const, naked_functions)]
#![cfg_attr(test, no_main)]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, test_runner(runner))]
#![cfg_attr(test, reexport_test_harness_main = "test_harness")]

#[cfg(test)]
extern crate self as gba_test;

mod alignment;
mod runner;
mod runtime;
mod test_case;
mod test;
mod ui;

#[cfg(feature = "macros")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "macros")))]
pub use gba_test_macros::test;
pub use runner::runner;
pub use test_case::{Ignore, ShouldPanic, Test, TestCase};

use test::{Outcome, Tests};

#[cfg(test)]
#[no_mangle]
pub fn main() {
    test_harness()
}
