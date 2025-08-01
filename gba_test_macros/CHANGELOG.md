# Changelog

## 0.4.0 - 2025-06-30
### Added
- Populates modules by splitting the current module path at compile time.
### Removed
- No longer populates the module field (since it was removed from `gba_test`).

## 0.3.0 - 2024-12-27
### Added
- Support for returning `Result<T, E>` from tests, as well as other custom return types.
- Expected messages for `#[should_panic]` attributes.
### Fixed
- `#[ignore]` no longer accepts incorrect arguments.

## 0.2.0 - 2024-12-24
### Fixed
- Errors now highlight test name instead of test macro.

## 0.1.1 - 2024-06-06
### Fixed
- Corrected workflow badge path.

## 0.1.0 - 2024-06-06
### Added
- `#[test]` attribute for writing tests.
- Support for `#[ignore]` attribute to mark tests as ignored.
- Support for `#[should_panic]` to define tests that are expected to panic.
