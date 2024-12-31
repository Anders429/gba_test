# Changelog

## 0.3.1 - 2024-12-31
### Changed
- `ShouldPanic::YesWithMessage(message)` no longer requires dynamic allocation.
### Fixed
- `ShouldPanic::YesWithMessage(message)` now matches against the panic message itself, not including location info.
- Attempting to store panic data with not enough space now correctly displays the data in the error message.

## 0.3.0 - 2024-12-27
### Added
- Support for returning `Result<T, E>` from tests, as well as other custom return types.
- `ShouldPanic::YesWithMessage(message)` to indicate that a panic message should contain the given substring.
### Fixed
- Scrolling through multiple failed tests no longer breaks due to incorrect pointer alignment arithmetic.
- Filtered test scrolling will no longer panic due to underflows.

## 0.2.0 - 2024-12-24
### Added
- A dynamic memory allocator, allowing use of `alloc` in tests.
### Changed
- Added `module()` to `TestCase`. `name()` should now only return the test name, not the module.

## 0.1.4 - 2024-10-09
### Fixed
- Resolved issue with unstable `naked_functions` feature which caused compilation failures on the latest nightly builds.
### Removed
- Reliance on `asm_const` unstable feature (the feature was recently stabilized).
- Reliance on `naked_functions` unstable feature, instead implementing the runtime using `global_asm!`.

## 0.1.3 - 2024-06-17
### Fixed
- Panic displaying now properly clears all previous text before displaying the panic info.
- User interface no longer panics when attempting to scroll down an empty list.
- Panics are now properly reported to `mgba-rom-test` using the same bios call as pass/fail reports.
- Ignored tests with long names no longer cause misaligned display of tests in user interface.

## 0.1.2 - 2024-06-06
### Fixed
- Provide correct target for docs.rs build.

## 0.1.1 - 2024-06-06
### Fixed
- Enabled `doc_cfg` feature to fix documentation build.

## 0.1.0 - 2024-06-06
### Added
- `#[test]` attribute for writing tests.
- Test `runner()` for running tests on the Game Boy Advance.
- `TestCase` trait for defining types that can be run on the test runner.
- `Ignore` `enum` to define whether a test should be ignored.
- `ShouldPanic` `enum` to define a test that is expected to panic.
- Runtime for handling running tests on the Game Boy Advance.
