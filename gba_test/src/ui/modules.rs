use super::{Cursor, KEYINPUT, TEXT_ENTRIES, UI_ENTRIES, wait_for_vblank};
use crate::{
    mmio::KeyInput,
    test::{ModuleFilter, TestOutcomes, TestOutcomesModules},
};
use core::fmt::Write;

/// Identify how many entries before the index were removed, and adjust the index accordingly.
///
/// If the offset index needs to be adjusted up as well (because more than index - offset_index
/// entries were removed), it will also be adjusted.
fn adjust_index_for_new_parent(
    offset_index: usize,
    modules: TestOutcomesModules,
    parent: &[&str],
) -> (usize, usize) {
    let mut index = None;
    for (module_index, module) in modules.enumerate() {
        if module == parent {
            index = Some(module_index);
            break;
        }
    }
    if let Some(index) = index {
        if index.saturating_sub(offset_index) > 17 {
            (index, index.saturating_sub(17))
        } else if offset_index > index {
            (index, index)
        } else {
            (index, offset_index)
        }
    } else {
        // We couldn't find the parent in the module list; default to the top of the list.
        (0, 0)
    }
}

fn draw_modules(index: usize, offset_index: usize, modules: TestOutcomesModules) {
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

    // Write the modules.
    let mut cursor = unsafe { Cursor::new(TEXT_ENTRIES) };
    for (row, module) in modules.skip(offset_index).take(18).enumerate() {
        for _ in 0..(module.len() - 1) {
            write!(cursor, "  ").expect("couldn't write spaces");
        }
        writeln!(
            cursor,
            "{}",
            module.last().expect("got an empty module list")
        )
        .expect("couldn't write");

        let mut ui_cursor = unsafe { UI_ENTRIES.byte_add(0x40 * row) };
        if index.saturating_sub(offset_index) == row {
            for _ in 0..30 {
                unsafe {
                    ui_cursor.write_volatile((4 << 12) | 1);
                    ui_cursor = ui_cursor.add(1);
                }
            }
        } else {
            for _ in 0..30 {
                unsafe {
                    ui_cursor.write_volatile(0);
                    ui_cursor = ui_cursor.add(1);
                }
            }
        }
    }
}

fn find_parent_in_outcomes<'a>(
    test_outcomes: &'a TestOutcomes,
    parent: &[&str],
) -> Option<&'a [&'a str]> {
    test_outcomes
        .modules(parent)
        .find(|&module| module == parent)
}

pub(super) fn show<'a, 'b>(
    test_outcomes: &'a TestOutcomes,
    parent: &'b [&'b str],
) -> Option<Option<ModuleFilter<'a>>>
where
    'a: 'b,
{
    let mut index = 0;
    let mut offset_index = 0;
    let mut old_keys = KeyInput::START;
    let mut parent = find_parent_in_outcomes(test_outcomes, parent).unwrap_or(&[]);
    loop {
        draw_modules(index, offset_index, test_outcomes.modules(parent));

        // Wait until input is received from the user.
        loop {
            wait_for_vblank();
            let keys = unsafe { KEYINPUT.read_volatile() };
            if keys != old_keys {
                if keys.contains(KeyInput::UP) {
                    index = index.saturating_sub(1);
                    old_keys = keys;
                    break;
                }
                if keys.contains(KeyInput::DOWN) {
                    index = index.saturating_add(1);
                    old_keys = keys;
                    break;
                }
                if keys.contains(KeyInput::LEFT) {
                    // Collapse the currently selected module.
                    if let Some(current_module) = test_outcomes.modules(parent).nth(index)
                        && parent.starts_with(current_module)
                        && let Some((_, new_parent)) = current_module.split_last()
                    {
                        parent = new_parent;
                        old_keys = keys;
                        break;
                    }
                }
                if keys.contains(KeyInput::RIGHT) {
                    // Expand the currently selected module.
                    if let Some(current_module) = test_outcomes.modules(parent).nth(index) {
                        parent = current_module;
                        (index, offset_index) = adjust_index_for_new_parent(
                            offset_index,
                            test_outcomes.modules(parent),
                            parent,
                        );
                        old_keys = keys;
                        break;
                    }
                }
                if keys.contains(KeyInput::A) {
                    return Some(
                        test_outcomes
                            .modules(parent)
                            .nth(index)
                            .map(|module| ModuleFilter::new(module)),
                    );
                }
                if keys.contains(KeyInput::B) || keys.contains(KeyInput::START) {
                    // Don't change the module filter at all.
                    return None;
                }
            }

            old_keys = keys;
        }
    }
}
