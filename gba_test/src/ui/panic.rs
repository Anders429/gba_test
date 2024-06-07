//! UI display for panic messages that occur outside of tests.

use super::{
    font, load_ui_tiles, wait_for_vblank, Cursor, BG0CNT, BG1CNT, DISPCNT, TEXT_ENTRIES, UI_ENTRIES,
};
use core::{fmt::Write, panic::PanicInfo};

const DISPSTAT: *mut u16 = 0x0400_0004 as *mut u16;
const IME: *mut bool = 0x0400_0208 as *mut bool;
const IE: *mut u16 = 0x0400_0200 as *mut u16;

/// Displays the panic info.
///
/// This is a terminating function. It is meant to simply display errors that occurred within the
/// framework to the user. It should not be used for panics that happen within test execution.
pub(crate) fn display(info: &PanicInfo) -> ! {
    // Enable interrupts.
    unsafe {
        DISPSTAT.write_volatile(8);
        IE.write_volatile(1);
        IME.write(true);
    }

    // Enable BG0 and BG1.
    unsafe {
        BG0CNT.write_volatile(8 << 8);
        BG1CNT.write_volatile((2 << 2) | (24 << 8));
        DISPCNT.write_volatile(768);
    }
    font::load();
    load_ui_tiles();

    wait_for_vblank();

    // Clear previous text and highlights.
    for y in 0..20 {
        for x in 0..30 {
            unsafe {
                TEXT_ENTRIES.add(0x20 * y + x).write_volatile(0);
                UI_ENTRIES.add(0x20 * y + x).write_volatile(0);
            }
        }
    }

    let mut cursor = unsafe { Cursor::new(TEXT_ENTRIES) };
    write!(
        cursor,
        "The framework panicked outsideof testing:\n\n{}\n\nPlease report this error!",
        info
    );

    loop {}
}
