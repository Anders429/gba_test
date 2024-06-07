# Changelog

## Unreleased
### Fixed
- Panic displaying now properly clears all previous text before displaying the panic info.
- User interface no longer panics when attempting to scroll down an empty list.
- Panics are now properly reported to `mgba-rom-test` using the same bios call as pass/fail reports.

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
