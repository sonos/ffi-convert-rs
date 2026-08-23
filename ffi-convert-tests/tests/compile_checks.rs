//! Compile-time checks for the derive macros: cases that must fail to
//! compile, and deprecation warnings, escalated to errors in the test files
//! via `#![deny(deprecated)]`.
//!
//! The `.stderr` snapshots are compared byte-for-byte with rustc's output,
//! which changes formatting between versions, so these tests are gated
//! behind the `compile-checks` feature and only run on the MSRV toolchain:
//!
//! ```text
//! cargo +1.88 test -p ffi-convert-tests --features compile-checks
//! ```
//!
//! Regenerate the snapshots with the same command prefixed by
//! `TRYBUILD=overwrite`.

#![cfg(feature = "compile-checks")]

#[test]
fn compile_checks() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_error/*.rs");
}
