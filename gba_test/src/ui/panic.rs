//! UI display for panic messages that occur outside of tests.

use super::{
    BG0CNT, BG1CNT, Cursor, DISPCNT, TEXT_ENTRIES, UI_ENTRIES, font, load_ui_tiles, wait_for_vblank,
};
use crate::{
    mmio::{DisplayStatus, Interrupt},
    runner::report_result,
};
use core::{fmt::Write, panic::PanicInfo};

const DISPSTAT: *mut DisplayStatus = 0x0400_0004 as *mut DisplayStatus;
const IME: *mut bool = 0x0400_0208 as *mut bool;
const IE: *mut Interrupt = 0x0400_0200 as *mut Interrupt;

/// Displays the panic info.
///
/// This is a terminating function. It is meant to simply display errors that occurred within the
/// framework to the user. It should not be used for panics that happen within test execution.
pub(crate) fn display(info: &PanicInfo) -> ! {
    // Enable interrupts.
    unsafe {
        DISPSTAT.write_volatile(DisplayStatus::ENABLE_VBLANK_INTERRUPTS);
        IE.write_volatile(Interrupt::VBLANK);
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
    // If this write fails, just ignore it since we are already panicking.
    let _ = write!(
        cursor,
        "The framework panicked outsideof testing:\n\n{info}\n\nPlease report this error!"
    );

    // Disable interrupts.
    unsafe {
        DISPSTAT.write_volatile(DisplayStatus::NONE);
        IE.write_volatile(Interrupt::NONE);
        IME.write(false);
    }

    // Report panic and halt.
    report_result(2);

    // This empty loop is just a catch-all in case the halt from the above report is somehow broken
    // from. Normally, the halt will pause the CPU indefinitely, as interrupts are enabled and
    // therefore cannot break the system from the halt state.
    #[allow(clippy::empty_loop)]
    loop {}
}
