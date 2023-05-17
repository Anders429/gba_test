//! `postcard` `Flavor`s for serializing data.
//!
//! Due to the nature of the storage targets on the GBA, custom storage targets are defined here.

use core::ptr;
use postcard::ser_flavors::Flavor;

/// The end of the SRAM.
const SRAM_END: *mut u8 = 0x0E00_FFFF as *mut u8;

/// Storage within SRAM.
///
/// This struct manages writing serialized data directly to SRAM. It is a `postcard` flavor and can
/// therefore be used in combination with other flavors.
pub(crate) struct Sram {
    /// The current position in SRAM.
    cursor: *mut u8,
}

impl Sram {
    /// Create a new SRAM writer.
    ///
    /// This creates a writer to SRAM at the given pointer location.
    ///
    /// # Safety
    /// The pointer location must be a valid location within SRAM (0x0E00_0000 to 0x0E00_FFFF).
    pub(crate) unsafe fn new(ptr: *mut u8) -> Self {
        Self { cursor: ptr }
    }
}

impl Flavor for Sram {
    /// Returns the position of the cursor at the end of writing.
    type Output = *mut u8;

    fn try_push(&mut self, data: u8) -> postcard::Result<()> {
        // We can write up to and including SRAM_END.
        if self.cursor >= SRAM_END {
            return Err(postcard::Error::SerializeBufferFull);
        }
        // SAFETY: These writes will always be to a valid location.
        unsafe {
            ptr::write_volatile(self.cursor, data);
            self.cursor = self.cursor.add(1);
        }
        Ok(())
    }

    fn finalize(self) -> postcard::Result<Self::Output> {
        Ok(self.cursor)
    }
}
