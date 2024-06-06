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
