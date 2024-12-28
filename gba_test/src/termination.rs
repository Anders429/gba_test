//! Defines a `Termination` trait to allow for different return types on `#[test]` definitions.

use core::fmt::Debug;

/// A trait for implementing arbitrary return types for [`#[test]`](crate::test)s.
///
/// This trait is analogous to the standard library's
/// [`Termination`](https://doc.rust-lang.org/std/process/trait.Termination.html). The main
/// difference is that rather than returning an exit code, this trait simply returns `()` on
/// success and panics on failure.
///
/// This trait is implemented for `()`, which covers the standard `#[test]` definition, and on
/// `Result<T, E>`, which allows for tests with signatures like `fn foo() -> Result<(), E>`.
pub trait Termination {
    /// Called to determine whether the test result is a success or a failure.
    ///
    /// On success, this should do nothing. On failure, it should panic.
    fn terminate(self);
}

impl Termination for () {
    fn terminate(self) {}
}

impl<T, E> Termination for Result<T, E>
where
    T: Termination,
    E: Debug,
{
    fn terminate(self) {
        match self {
            Ok(value) => value.terminate(),
            Err(error) => panic!("{error:?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Termination;
    use gba_test::test;

    #[test]
    fn terminate_unit() {
        // Verify that this does not panic.
        ().terminate();
    }

    #[test]
    fn terminate_ok() {
        // Verify that this does not panic.
        Ok::<_, ()>(()).terminate()
    }

    #[test]
    #[should_panic]
    fn terminate_err() {
        Err::<(), _>("foo").terminate()
    }
}
