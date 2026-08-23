//! Compile-time checks for the derive macros: cases that must fail to
//! compile, and deprecation warnings, escalated to errors in the test files
//! via `#![deny(deprecated)]`.

#[test]
fn compile_checks() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_error/*.rs");
}
