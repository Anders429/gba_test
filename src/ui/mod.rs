/// Works like [`include_bytes!`], but the value is wrapped in [`Align4`].
#[macro_export]
macro_rules! include_aligned_bytes {
  ($file:expr $(,)?) => {{
    crate::ui::Align4(*include_bytes!($file))
  }};
}

mod cursor;
mod font;

use crate::{outcome::{Outcome, Outcomes}, test_case::TestCase};
use core::{arch::asm, fmt::Write};
use cursor::Cursor;

const DISPCNT: *mut u16 = 0x0400_0000 as *mut u16;
const BG0CNT: *mut u16 = 0x0400_0008 as *mut u16;
const BG1CNT: *mut u16 = 0x0400_000A as *mut u16;
const TEXT_ENTRIES: *mut u16 = 0x0600_4000 as *mut u16;
const UI_ENTRIES: *mut u16 = 0x0600_C000 as *mut u16;
const CHARBLOCK2: *mut [u32; 8] = 0x0600_8000 as *mut [u32; 8];

/// Wraps a value to be aligned to a minimum of 4.
///
/// If the size of the value held is already a multiple of 4 then this will be
/// the same size as the wrapped value. Otherwise the compiler will add
/// sufficient padding bytes on the end to make the size a multiple of 4.
#[derive(Debug)]
#[repr(C, align(4))]
struct Align4<T>(pub T);

impl<const N: usize> Align4<[u8; N]> {
    #[inline]
    #[must_use]
    pub fn as_u32_slice(&self) -> &[u32] {
        assert!(self.0.len() % 4 == 0);
        // Safety: our struct is aligned to 4, so the pointer will already be
        // aligned, we only need to check the length
        unsafe {
            let data: *const u8 = self.0.as_ptr();
            let len: usize = self.0.len();
            core::slice::from_raw_parts(data.cast::<u32>(), len / 4)
        }
    }
}

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

/// Loads the tiles needed for the UI.
fn load_ui_tiles() {
    let mut tile = [0u32; 8];
    for (index, row) in include_aligned_bytes!("../../data/ui.4bpp").as_u32_slice().iter().take(8).enumerate() {
        tile[index] = *row;
    }
    unsafe {
        CHARBLOCK2.add(1).write_volatile(tile);
    }
}

fn draw_test_outcomes<'a, TestOutcomes>(test_outcomes: TestOutcomes) where TestOutcomes: Iterator<Item = (&'a &'a dyn TestCase, Outcome<&'static str>)> {
    wait_for_vblank();
    // Draw UI.
    for row in 0..2 {
        let mut cursor = unsafe {UI_ENTRIES.byte_add(0x40 * row)};
        for _ in 0..30 {
            unsafe {
                cursor.write_volatile(4 << 12 | 1);
                cursor = cursor.add(1);
            }
        }
    }

    // Write outcome text.
    let mut cursor = unsafe {Cursor::new(TEXT_ENTRIES)};
    write!(cursor, "  All  Failed Passed Ignored\n(####) (####) (####) (####)");
    for (test, outcome) in test_outcomes.take(18) {
        let palette = match outcome {
            Outcome::Passed => 1,
            Outcome::Ignored => 2,
            Outcome::Failed(_) => 3,
        };

        cursor.set_palette(0);
        write!(cursor, "\n{}: ", test.name());
        cursor.set_palette(palette);
        write!(cursor, "{}", outcome.as_str());
    }
}

pub(crate) fn run(tests: &[&dyn TestCase], outcomes: &Outcomes) -> ! {
    // Enable BG0 and BG1.
    unsafe {
        BG0CNT.write_volatile(8 << 8);
        BG1CNT.write_volatile((2 << 2) | (24 << 8));
        DISPCNT.write_volatile(768);
    }
    font::load();
    load_ui_tiles();

    // Test selection.
    loop {
        // Draw the tests that should currently be viewable.
        draw_test_outcomes(tests.iter().zip(outcomes.iter_outcomes()));
        // Wait until input is received from the user.
        loop {}
        // Then redraw the next screen.
    }
}