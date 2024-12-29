/// Works like [`include_bytes!`], but the value is wrapped in [`Align4`].
macro_rules! include_aligned_bytes {
    ($file:expr $(,)?) => {{
        crate::ui::Align4(*include_bytes!($file))
    }};
}

pub(crate) mod panic;

mod cursor;
mod entry;
mod font;
mod palette;

use crate::{test, test::TestOutcomes, test_case::TestCase, Outcome};
use core::{arch::asm, cmp::min, fmt::Write};
use cursor::Cursor;

const DISPCNT: *mut u16 = 0x0400_0000 as *mut u16;
const BG0CNT: *mut u16 = 0x0400_0008 as *mut u16;
const BG1CNT: *mut u16 = 0x0400_000A as *mut u16;
const KEYINPUT: *mut u16 = 0x0400_0130 as *mut u16;
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
    for (index, row) in include_aligned_bytes!("../../data/ui.4bpp")
        .as_u32_slice()
        .iter()
        .take(8)
        .enumerate()
    {
        tile[index] = *row;
    }
    unsafe {
        CHARBLOCK2.add(1).write_volatile(tile);
    }
}

// TODO: Make this a `Selection` struct. Should contain its index, the index within the shown stuff, the pointer to the top-most shown element, etc.
fn draw_test_outcomes<'a, TestOutcomes, const SIZE: usize>(
    test_outcomes: TestOutcomes,
    index: usize,
    lengths: [usize; 4],
    page: &Page<SIZE>,
) where
    TestOutcomes: Iterator<Item = (&'a dyn TestCase, Outcome<&'static str>)>,
{
    wait_for_vblank();
    // Draw UI.
    for row in 0..2 {
        let mut cursor = unsafe { UI_ENTRIES.byte_add(0x40 * row) };
        for i in 0..30 {
            unsafe {
                if match page {
                    Page::All(_) => i < 7,
                    Page::Failed(_) => (7..14).contains(&i),
                    Page::Passed(_) => (14..21).contains(&i),
                    Page::Ignored(_) => i >= 21,
                } {
                    cursor.write_volatile((4 << 12) | 1);
                } else {
                    cursor.write_volatile(0);
                }
                cursor = cursor.add(1);
            }
        }
    }
    // Highlight selected.
    for row in 0..18 {
        let mut cursor = unsafe { UI_ENTRIES.byte_add(0x40 * (row + 2)) };
        if index == row
            && match page {
                Page::All(_) => lengths[0] > 0,
                Page::Failed(_) => lengths[1] > 0,
                Page::Passed(_) => lengths[2] > 0,
                Page::Ignored(_) => lengths[3] > 0,
            }
        {
            for _ in 0..30 {
                unsafe {
                    cursor.write_volatile((4 << 12) | 1);
                    cursor = cursor.add(1);
                }
            }
        } else {
            for _ in 0..30 {
                unsafe {
                    cursor.write_volatile(0);
                    cursor = cursor.add(1);
                }
            }
        }
    }

    // Clear previous text.
    for y in 0..20 {
        for x in 0..30 {
            unsafe {
                TEXT_ENTRIES.add(0x20 * y + x).write_volatile(0);
            }
        }
    }

    // Write outcome text.
    let mut cursor = unsafe { Cursor::new(TEXT_ENTRIES) };
    writeln!(cursor, "  All  Failed Passed Ignored").expect("failed to write page names");
    for length in lengths {
        write!(cursor, "({:^4}) ", length).expect("failed to write test counts");
    }
    for (test, outcome) in test_outcomes.take(18) {
        cursor.set_palette(0);
        if test.name().chars().count() < 22 {
            write!(cursor, "\n{}: ", test.name()).expect("failed to write full test name");
        } else {
            // TODO: Make this indexing more robust. It needs to account for unicode, not just ascii.
            write!(
                cursor,
                "\n{}..{}: ",
                &test.name()[..9],
                &test.name()[(test.name().len() - 10)..]
            )
            .expect("failed to write partial test name");
        }
        cursor.set_palette(outcome.palette());
        write!(cursor, "{}", outcome.as_str()).expect("failed to write test outcome");
    }
}

enum Page<'a, const SIZE: usize> {
    All(&'a mut test::Window<test::All, SIZE>),
    Failed(&'a mut test::Window<test::Failed, SIZE>),
    Passed(&'a mut test::Window<test::Passed, SIZE>),
    Ignored(&'a mut test::Window<test::Ignored, SIZE>),
}

impl<const SIZE: usize> Page<'_, SIZE> {
    fn prev(&mut self) {
        match self {
            Self::All(window) => window.prev(),
            Self::Failed(window) => window.prev(),
            Self::Passed(window) => window.prev(),
            Self::Ignored(window) => window.prev(),
        };
    }

    fn next(&mut self) {
        match self {
            Self::All(window) => window.next(),
            Self::Failed(window) => window.next(),
            Self::Passed(window) => window.next(),
            Self::Ignored(window) => window.next(),
        };
    }

    fn get(&mut self, index: usize) -> Option<(&dyn TestCase, Outcome<&'static str>)> {
        match self {
            Self::All(window) => window.get(index),
            Self::Failed(window) => window.get(index),
            Self::Passed(window) => window.get(index),
            Self::Ignored(window) => window.get(index),
        }
    }
}

pub(crate) fn run(test_outcomes: TestOutcomes) -> ! {
    // Enable BG0 and BG1.
    unsafe {
        BG0CNT.write_volatile(8 << 8);
        BG1CNT.write_volatile((2 << 2) | (24 << 8));
        DISPCNT.write_volatile(768);
    }
    font::load();
    load_ui_tiles();

    // Test selection.
    let all_length = test_outcomes.iter().count();
    let failed_length = test_outcomes
        .iter()
        .filter(|(_, outcome)| matches!(outcome, Outcome::Failed(_)))
        .count();
    let passed_length = test_outcomes
        .iter()
        .filter(|(_, outcome)| matches!(outcome, Outcome::Passed))
        .count();
    let ignored_length = test_outcomes
        .iter()
        .filter(|(_, outcome)| matches!(outcome, Outcome::Ignored))
        .count();
    let lengths = [all_length, failed_length, passed_length, ignored_length];
    let mut all_window = test::Window::<test::All, 18>::new(&test_outcomes, all_length);
    let mut failed_window = test::Window::<test::Failed, 18>::new(&test_outcomes, failed_length);
    let mut passed_window = test::Window::<test::Passed, 18>::new(&test_outcomes, passed_length);
    let mut ignored_window = test::Window::<test::Ignored, 18>::new(&test_outcomes, ignored_length);
    let mut page = Page::All(&mut all_window);
    let mut all_index = 0;
    let mut failed_index = 0;
    let mut passed_index = 0;
    let mut ignored_index = 0;
    let mut old_keys = 0b0000_0011_1111_1111;
    loop {
        // Draw the tests that should currently be viewable.
        match page {
            Page::All(ref window) => draw_test_outcomes(window.iter(), all_index, lengths, &page),
            Page::Failed(ref window) => {
                draw_test_outcomes(window.iter(), failed_index, lengths, &page)
            }
            Page::Passed(ref window) => {
                draw_test_outcomes(window.iter(), passed_index, lengths, &page)
            }
            Page::Ignored(ref window) => {
                draw_test_outcomes(window.iter(), ignored_index, lengths, &page)
            }
        }
        // Wait until input is received from the user.
        loop {
            let (index, length) = match page {
                Page::All(_) => (&mut all_index, all_length),
                Page::Failed(_) => (&mut failed_index, failed_length),
                Page::Passed(_) => (&mut passed_index, passed_length),
                Page::Ignored(_) => (&mut ignored_index, ignored_length),
            };
            wait_for_vblank();
            let keys = unsafe { KEYINPUT.read_volatile() };
            if keys != old_keys {
                if keys == 0b0000_0011_1011_1111 {
                    // Up
                    if *index == 0 {
                        page.prev();
                    } else {
                        *index -= 1;
                    }
                    old_keys = keys;
                    break;
                }
                if keys == 0b0000_0011_0111_1111 {
                    // Down
                    if *index == min(17, length.saturating_sub(1)) {
                        page.next();
                    } else {
                        *index += 1;
                    }
                    old_keys = keys;
                    break;
                }
                if keys == 0b0000_0010_1111_1111 {
                    // R
                    page = match page {
                        Page::All(_) => Page::Failed(&mut failed_window),
                        Page::Failed(_) => Page::Passed(&mut passed_window),
                        _ => Page::Ignored(&mut ignored_window),
                    };
                    old_keys = keys;
                    break;
                }
                if keys == 0b0000_0001_1111_1111 {
                    // L
                    page = match page {
                        Page::Ignored(_) => Page::Passed(&mut passed_window),
                        Page::Passed(_) => Page::Failed(&mut failed_window),
                        _ => Page::All(&mut all_window),
                    };
                    old_keys = keys;
                    break;
                }
                if keys == 0b0000_0011_1111_1110 {
                    // A
                    if let Some((test_case, outcome)) = page.get(*index) {
                        entry::show(test_case, outcome);
                        old_keys = keys;
                        break;
                    }
                }
            }

            old_keys = keys;
        }
    }
}
