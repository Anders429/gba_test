//! Tools for ensuring proper alignment.
//!
//! On the GBA, we often need data to have an alignment of 4. Within this module are some tools to
//! help ensure that we have this alignment and can therefore properly access data.

/// Defines a pointer type as being able to be aligned to 4 bytes.
///
/// The types that implement this (namely, `*const u8` and `*mut u8`) can be aligned to 4 bytes (in
/// either direction) by calling the provided methods.
pub(crate) trait Align4 {
    /// Returns the next location aligned to four bytes, or the current location if it is already
    /// properly aligned.
    fn align_forward(self) -> Self;

    /// Returns the previous location aligned to four bytes, or the current location if it is
    /// already properly aligned.
    fn align_backward(self) -> Self;
}

impl<T> Align4 for *const T {
    #[inline]
    fn align_forward(self) -> Self {
        unsafe { self.byte_add((4 - (self as usize % 4)) % 4) }
    }

    #[inline]
    fn align_backward(self) -> Self {
        unsafe { self.byte_sub(self as usize % 4) }
    }
}

impl<T> Align4 for *mut T {
    #[inline]
    fn align_forward(self) -> Self {
        unsafe { self.byte_add((4 - (self as usize % 4)) % 4) }
    }

    #[inline]
    fn align_backward(self) -> Self {
        unsafe { self.byte_sub(self as usize % 4) }
    }
}

#[cfg(test)]
mod tests {
    use super::Align4;
    use gba_test::test;

    #[test]
    fn align_forward_aligned() {
        assert_eq!(
            (0x0200_0000 as *const u8).align_forward(),
            0x0200_0000 as *const u8
        );
    }

    #[test]
    fn align_forward_unaligned() {
        assert_eq!(
            (0x0200_0001 as *const u8).align_forward(),
            0x0200_0004 as *const u8
        );
    }

    #[test]
    fn align_backward_aligned() {
        assert_eq!(
            (0x0200_0000 as *const u8).align_backward(),
            0x0200_0000 as *const u8
        );
    }

    #[test]
    fn align_backward_unaligned() {
        assert_eq!(
            (0x0200_0002 as *const u8).align_backward(),
            0x0200_0000 as *const u8
        );
    }

    #[test]
    fn align_backward_null() {
        assert_eq!((0 as *const u8).align_backward(), 0 as *const u8);
    }
}
