use super::{KEYINPUT, TEXT_ENTRIES, UI_ENTRIES, cursor::Cursor, wait_for_vblank};
use crate::{Outcome, mmio::KeyInput, test_case::TestCase};
use core::fmt::Write;

pub(super) fn show(test_case: &dyn TestCase, outcome: Outcome<&'static str>) {
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
    // Write test name.
    for module in test_case.modules() {
        write!(cursor, "{}::", module).expect("failed to write module")
    }
    writeln!(cursor, "{}", test_case.name()).expect("failed to write test name");

    // Write test result.
    cursor.set_palette(outcome.palette());
    writeln!(cursor, "{}", outcome.as_str()).expect("failed to write test outcome");

    // Write message.
    cursor.set_palette(0);
    match outcome {
        Outcome::Passed => {
            write!(cursor, "The test passed!").expect("failed to write passed message");
        }
        Outcome::Ignored => {
            if let Some(message) = test_case.message() {
                write!(cursor, "The test was ignored:\n{}", message)
                    .expect("failed to write ignored message");
            } else {
                write!(cursor, "The test was ignored.").expect("failed to write ignored message");
            }
        }
        Outcome::Failed(message) => {
            write!(cursor, "{}", message).expect("failed to write failure message");
        }
    }

    // Wait for input.
    loop {
        wait_for_vblank();
        let keys = unsafe { KEYINPUT.read_volatile() };

        if keys.contains(KeyInput::B) {
            // B
            return;
        }
    }
}
