use super::{cursor::Cursor, wait_for_vblank, KEYINPUT, TEXT_ENTRIES};
use crate::{Outcome, test_case::TestCase};
use core::fmt::Write;

pub(super) fn show(test_case: &dyn TestCase, outcome: Outcome<&'static str>) {
    // Clear previous text.
    for y in 0..20 {
        for x in 0..30 {
            unsafe {
                TEXT_ENTRIES.add(0x20 * y + x).write_volatile(0);
            }
        }
    }

    let mut cursor = unsafe { Cursor::new(TEXT_ENTRIES) };
    // Write test name.
    write!(cursor, "{}\n", test_case.name());

    // Write test result.
    let message = match outcome {
        Outcome::Passed => {
            cursor.set_palette(1);
            "The test passed!"
        }
        Outcome::Ignored => {
            cursor.set_palette(2);
            "The test was ignored."
        }
        Outcome::Failed(message) => {
            cursor.set_palette(3);
            message
        }
    };
    write!(cursor, "{}\n", outcome.as_str());

    // Write message.
    cursor.set_palette(0);
    write!(cursor, "{}", message);

    // Wait for input.
    loop {
        wait_for_vblank();
        let keys = unsafe { KEYINPUT.read_volatile() };

        if keys == 0b0000_0011_1111_1101 {
            // B
            return;
        }
    }
}
