//! Facades for logging.
//!
//! These are equivalent to using `log`'s macros directly, but they only expand if the `log`
//! feature is enabled.
//!
//! Only the macros used in this framework are provided here. More can be added as needed.

macro_rules! info {
    ($($tokens:tt)*) => {
        #[cfg(feature = "log")]
        {
            ::log::info!($($tokens)*)
        }
    }
}

macro_rules! error {
    ($($tokens:tt)*) => {
        #[cfg(feature = "log")]
        {
            ::log::error!($($tokens)*)
        }
    }
}

pub(crate) use {error, info};
