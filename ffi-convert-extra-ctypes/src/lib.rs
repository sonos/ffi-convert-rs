//! Extra C-compatible utility types for [`ffi_convert`].
//!
//! These types mirror common stdlib types ([`Vec<String>`], [`Vec<T>`], [`std::ops::Range`])
//! with C-compatible representations. They are provided as a convenience but are not
//! required to use the `ffi-convert` core crate: users who prefer to define their own
//! layouts can skip this crate entirely.

mod c_array;
mod c_range;
mod c_string_array;

pub use c_array::CArray;
pub use c_range::CRange;
pub use c_string_array::CStringArray;
