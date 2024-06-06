# Changelog

## 0.1.0 - 2024-06-06
### Added
- `#[test]` attribute for writing tests.
- Test `runner()` for running tests on the Game Boy Advance.
- `TestCase` trait for defining types that can be run on the test runner.
- `Ignore` `enum` to define whether a test should be ignored.
- `ShouldPanic` `enum` to define a test that is expected to panic.
- Runtime for handling running tests on the Game Boy Advance.
