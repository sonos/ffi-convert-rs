//! C-compatible mirrors of a few standard Rust containers, for use with
//! [`ffi-convert`](https://docs.rs/ffi-convert).
//!
//! The three types exposed here cover the containers most FFI boundaries
//! inevitably need:
//!
//! | Rust type        | C-compatible mirror            |
//! |------------------|--------------------------------|
//! | `Vec<T>`         | [`CArray<T>`]                  |
//! | `Vec<String>`    | [`CStringArray`]               |
//! | `std::ops::Range<T>` | [`CRange<T>`]              |
//!
//! Each mirror implements [`CReprOf`](ffi_convert::CReprOf),
//! [`AsRust`](ffi_convert::AsRust), and [`CDrop`](ffi_convert::CDrop), so it
//! can be embedded in any struct you derive the conversion traits on — see
//! the top-level [`ffi-convert`](https://docs.rs/ffi-convert) documentation
//! for how the pieces fit together.
//!
//! This crate is optional: if none of these types fit your layout, depend on
//! `ffi-convert` alone and define your own `#[repr(C)]` container with the
//! conversion trait impls you need.

mod c_array;
mod c_range;
mod c_string_array;

pub use c_array::CArray;
pub use c_range::CRange;
pub use c_string_array::CStringArray;
