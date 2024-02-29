//! Compilation tests using the `trybuild` crate.

#[test]
fn test() {
    trybuild::TestCases::new().compile_fail("tests/trybuild/*.rs");
}
