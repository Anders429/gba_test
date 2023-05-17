#![no_std]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![cfg_attr(
    all(feature = "runner", target = "thumbv4t-none-eabi"),
    feature(panic_info_message)
)]

#[cfg(any(feature = "alloc", test))]
extern crate alloc;

#[cfg(all(feature = "runner", any(target = "thumbv4t-none-eabi", doc)))]
pub(crate) mod flavors;

#[cfg(all(feature = "runner", any(target = "thumbv4t-none-eabi", doc)))]
mod runner;
mod test_case;
mod trial;

#[cfg(feature = "gba_test_macros")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "macros")))]
pub use gba_test_macros::test;
#[cfg(all(feature = "runner", any(target = "thumbv4t-none-eabi", doc)))]
pub use runner::runner;
pub use test_case::{Ignore, Test, TestCase};
pub use trial::{Outcome, Trial};
