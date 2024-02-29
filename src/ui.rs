use crate::{font, outcome::{Outcome, Outcomes}, test_case::TestCase};
use core::arch::asm;

const DISPCNT: *mut u16 = 0x0400_0000 as *mut u16;
const BG0CNT: *mut u16 = 0x0400_0008 as *mut u16;
const TEXT_ENTRIES: *mut u16 = 0x0600_4000 as *mut u16;

/// Waits until a new v-blank interrupt occurs.
#[instruction_set(arm::t32)]
fn wait_for_vblank() {
    unsafe {
        asm! {
            "swi #0x05",
            out("r0") _,
            out("r1") _,
            out("r3") _,
            options(preserves_flags),
        }
    };
}

pub(crate) fn run(tests: &[&dyn TestCase], outcomes: &Outcomes) -> ! {
    // Enable BG0;
    unsafe {
        BG0CNT.write_volatile(8 << 8);
        DISPCNT.write_volatile(256);
    }
    font::load();

    // Display outcomes.
    for (row, (test, outcome)) in tests.iter().zip(outcomes.iter_outcomes()).enumerate() {
        log::info!("{}: {:?}", test.name(), outcome);

        let palette = match outcome {
            Outcome::Passed => 1,
            Outcome::Ignored => 2,
            Outcome::Failed(_) => 3,
        };
        // Write the test results.
        // We will first naively do this for every single test without worrying about scrolling.
        // This naturally will not work with larger amounts of tests, since they won't all fit on the screen.
        let mut cursor = unsafe {TEXT_ENTRIES.byte_add(0x40 * row)};
        wait_for_vblank();
        for character in test.name().chars().chain(": ".chars()) {
            let ascii: u32 = character.into();
            // Only account for basic characters.
            if ascii < 128 {
                unsafe {
                    cursor.write_volatile((ascii) as u16);
                    cursor = cursor.add(1);
                }
            }
        }
        for character in outcome.as_str().chars() {
            let ascii: u32 = character.into();
            // Only account for basic characters.
            if ascii < 128 {
                unsafe {
                    cursor.write_volatile((ascii | (palette << 12)) as u16);
                    cursor = cursor.add(1);
                }
            }
        }
    }

    loop {}
}