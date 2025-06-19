//! A bump allocator based in EWRAM.
//!
//! This is a very simple allocator implementation that allocates space on EWRAM sequentially,
//! starting from the end of EWRAM. Deallocated space is not reused unless it was the last block to
//! be allocated. This is not very efficient, but it is sufficient for running tests that shouldn't
//! need to reallocate often enough for it to matter.
//!
//! The allocator should be re-initialized before every test is run. Since EWRAM is not cleared
//! between tests, data that was previously allocated will still be present. However, those
//! addresses can safely be reused, since all previously-allocated data from the previous test can
//! be treated as though it was dropped.

use core::{
    alloc::{GlobalAlloc, Layout},
    ptr,
};

/// The allocator's state.
///
/// This value is mutated whenever a new allocation is made.
static mut STATE: State = State {
    cursor: 0x0204_0000 as *const u8,
    limit: {
        extern "C" {
            static __ewram_data_end: u8;
        }

        &raw const __ewram_data_end
    },
};

/// The allocator's state.
///
/// This is the actual value that is mutated when allocations are made.
///
/// # Safety
/// The main safety concerns are if an allocation is interrupted by either an interrupt or a panic,
/// which then creates a new allocation and returns back to the previous allocation process.
/// However, when running user code, no interrupts will ever create allocations, and all panics
/// will simply mark the test as a failure and reset to the next test. Therefore, the case of
/// allocating during allocation, then returning back to the first allocation, should not occur.
struct State {
    cursor: *const u8,
    limit: *const u8,
}

impl State {
    unsafe fn alloc(this: *mut Self, layout: Layout) -> *mut u8 {
        // Align.
        let mask = layout.align() - 1;
        let offset = (*this).cursor as usize & mask;
        (*this).cursor = (((*this).cursor as usize) - offset) as *const u8;

        (*this).cursor = ((*this).cursor as usize).saturating_sub(layout.size()) as *const u8;

        if (*this).cursor >= (*this).limit {
            (*this).cursor as *mut u8
        } else {
            ptr::null_mut()
        }
    }

    unsafe fn dealloc(this: *mut Self, ptr: *mut u8, layout: Layout) {
        // If this is the last allocation, we can move the cursor back.
        if ptr::eq(ptr, (*this).cursor) {
            (*this).cursor = (*this).cursor.add(layout.size())
        }
    }
}

/// A handle to the allocator.
///
/// This does not contain any state itself. Instead, the state is contained within the `STATE`
/// static mutable value.
pub(crate) struct Allocator;

unsafe impl Sync for Allocator {}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { State::alloc(&raw mut STATE, layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { State::dealloc(&raw mut STATE, ptr, layout) }
    }
}

/// Initialize the allocator with `limit` as the maximum byte address.
///
/// This should be called before every test is run.
pub(crate) unsafe fn init(limit: *const u8) {
    unsafe {
        STATE = State {
            cursor: 0x0204_0000 as *const u8,
            limit,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::State;
    use core::{alloc::Layout, ptr};
    use gba_test_macros::test;

    #[test]
    fn allocate() {
        let mut state = State {
            cursor: 0x0000_0020 as *const u8,
            limit: ptr::null(),
        };

        unsafe {
            assert_eq!(
                State::alloc(&raw mut state, Layout::from_size_align_unchecked(8, 4)),
                0x0000_0018 as *mut u8
            );

            assert_eq!(state.cursor, 0x0000_0018 as *const u8);
            assert_eq!(state.limit, ptr::null());
        }
    }

    #[test]
    fn allocate_align() {
        let mut state = State {
            cursor: 0x0000_0023 as *const u8,
            limit: ptr::null(),
        };

        unsafe {
            assert_eq!(
                State::alloc(&raw mut state, Layout::from_size_align_unchecked(8, 4)),
                0x0000_0018 as *mut u8
            );

            assert_eq!(state.cursor, 0x0000_0018 as *const u8);
            assert_eq!(state.limit, ptr::null());
        }
    }

    #[test]
    fn allocate_not_enough_space() {
        let mut state = State {
            cursor: 0x0000_0020 as *const u8,
            limit: 0x0000_001e as *const u8,
        };

        unsafe {
            assert_eq!(
                State::alloc(&raw mut state, Layout::from_size_align_unchecked(8, 4)),
                ptr::null_mut()
            );

            assert_eq!(state.cursor, 0x0000_0018 as *const u8);
            assert_eq!(state.limit, 0x0000_001e as *const u8);
        }
    }

    #[test]
    fn allocate_saturates_to_null() {
        let mut state = State {
            cursor: 0x0000_0004 as *const u8,
            limit: 0x0000_0002 as *const u8,
        };

        unsafe {
            assert_eq!(
                State::alloc(&raw mut state, Layout::from_size_align_unchecked(8, 4)),
                ptr::null_mut()
            );

            assert_eq!(state.cursor, ptr::null());
            assert_eq!(state.limit, 0x0000_0002 as *const u8);
        }
    }

    #[test]
    fn deallocate_last() {
        let mut state = State {
            cursor: 0x0000_0020 as *const u8,
            limit: ptr::null(),
        };

        unsafe {
            State::dealloc(
                &raw mut state,
                0x0000_0020 as *mut u8,
                Layout::from_size_align_unchecked(8, 4),
            );
        }

        assert_eq!(state.cursor, 0x0000_0028 as *const u8);
        assert_eq!(state.limit, ptr::null());
    }

    #[test]
    fn deallocate_not_last() {
        let mut state = State {
            cursor: 0x0000_0020 as *const u8,
            limit: ptr::null(),
        };

        unsafe {
            State::dealloc(
                &raw mut state,
                0x0000_0028 as *mut u8,
                Layout::from_size_align_unchecked(8, 4),
            );
        }

        assert_eq!(state.cursor, 0x0000_0020 as *const u8);
        assert_eq!(state.limit, ptr::null());
    }
}
