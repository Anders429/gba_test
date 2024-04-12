use super::{cursor::Cursor, wait_for_vblank, KEYINPUT, TEXT_ENTRIES};
use crate::{test_case::TestCase, Outcome};
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
    cursor.set_palette(outcome.palette());
    write!(cursor, "{}\n", outcome.as_str());

    // Write message.
    cursor.set_palette(0);
    match outcome {
        Outcome::Passed => {
            write!(cursor, "The test passed!");
        }
        Outcome::Ignored => {
            if let Some(message) = test_case.message() {
                write!(cursor, "The test was ignored:\n{}", message);
            } else {
                write!(cursor, "The test was ignored.");
            }
        }
        Outcome::Failed(message) => {
            write!(cursor, "{}", message);
        }
    }

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
