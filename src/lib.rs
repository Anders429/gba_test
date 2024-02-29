#![no_std]
#![feature(asm_const, naked_functions)]

mod font;
mod outcome;
mod runner;
mod runtime;
mod test_case;
mod ui;

#[cfg(feature = "macros")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "macros")))]
pub use gba_test_macros::test;
pub use runner::runner;
pub use test_case::{Ignore, Test, TestCase};

use outcome::{Outcome, Outcomes};
