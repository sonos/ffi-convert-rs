//! Traits and helpers to convert between idiomatic Rust values and C-compatible
//! representations when crossing an FFI boundary.
//!
//! The crate is built around two conversion traits, [`CReprOf`] and [`AsRust`],
//! and two supporting traits, [`CDrop`] and [`RawPointerConverter`]. Derive
//! macros for all four are provided by the companion
//! [`ffi-convert-derive`](https://docs.rs/ffi-convert-derive) crate and
//! re-exported here.
//!
//! Common containers (arrays, string arrays, ranges) live in the separate
//! [`ffi-convert-extra-ctypes`](https://docs.rs/ffi-convert-extra-ctypes)
//! crate and can be pulled in on demand.
//!
//! # Philosophy
//!
//! `ffi-convert`'s memory-management model makes as few assumptions as
//! possible about how the C side allocates, holds, or frees memory.
//!
//! Two traits cover the two directions across the FFI boundary:
//!
//! - **Incoming from C** — [`AsRust`] takes a `&CFoo` and returns an owned
//!   `Foo` built by deep-copying every field. It is a defensive copy: once
//!   `as_rust` returns, the resulting Rust value does not reference any
//!   C-owned memory, and nothing else in the crate reads from the original
//!   pointer afterwards. The C caller is free to keep, reuse, or release
//!   the pointer however its own rules require.
//! - **Outgoing to C** — [`CReprOf`] consumes a `Foo` and produces a `CFoo`
//!   that owns any heap memory its pointer fields reference. The `CFoo` is
//!   then handed to C as a raw pointer; to release everything, C sends the
//!   pointer back to Rust through a `free`-style FFI function that lets the
//!   value drop (releasing its pointer fields via [`CDrop`]).
//!
//! ```text
//!             CPizza::c_repr_of(pizza)
//!         ┌───────────────────────────────┐
//!         │                               ▼
//!   ┌──────────┐                     ┌──────────┐
//!   │  Pizza   │                     │  CPizza  │
//!   │  (Rust)  │                     │   (C)    │
//!   └──────────┘                     └──────────┘
//!         ▲                               │
//!         └───────────────────────────────┘
//!                 c_pizza.as_rust()
//! ```
//!
//! # Quick example
//!
//! Define the Rust type you want to expose, then define a `#[repr(C)]` mirror
//! and derive the conversion traits. The mirror's fields use C-compatible
//! types (see [the mapping table](#type-mapping)).
//!
//! ```
//! use ffi_convert::{AsRust, CDrop, CReprOf, RawBorrow, RawPointerConverter};
//! use libc::{c_char, c_float};
//!
//! pub struct Sauce {
//!     pub spiciness: f32,
//! }
//!
//! #[repr(C)]
//! #[derive(CReprOf, AsRust, CDrop, RawPointerConverter)]
//! #[target_type(Sauce)]
//! pub struct CSauce {
//!     pub spiciness: c_float,
//! }
//!
//! pub struct Pizza {
//!     pub name: String,
//!     pub base: Option<Sauce>,
//!     pub weight: f32,
//! }
//!
//! #[repr(C)]
//! #[derive(CReprOf, AsRust, CDrop, RawPointerConverter)]
//! #[target_type(Pizza)]
//! pub struct CPizza {
//!     pub name: *const c_char,
//!     #[nullable]
//!     pub base: *const CSauce,
//!     pub weight: c_float,
//! }
//! ```
//!
//! Two things to notice:
//!
//! - `CSauce` derives [`RawPointerConverter`] because `CPizza::base` stores a
//!   `*const CSauce`; `CPizza` derives it too so it can itself be handed to C
//!   as a `*const CPizza`. In both cases the derived [`CReprOf`] turns a value
//!   into a raw pointer via `into_raw_pointer`.
//! - `CPizza::base` carries `#[nullable]` because the Rust field is
//!   `Option<Sauce>`. The attribute tells the derives to map `None` to a null
//!   pointer on the way out and a null pointer to `None` on the way back.
//!
//! With the derives in place, let's write an FFI wrapper with three small functions —
//! one to read a C-owned value, one to hand a Rust value to C, and one to free
//! it:
//!
//! ```
//! # use ffi_convert::{AsRust, CDrop, CReprOf, RawBorrow, RawPointerConverter};
//! # use libc::{c_char, c_float};
//! # pub struct Sauce { pub spiciness: f32 }
//! # #[repr(C)]
//! # #[derive(CReprOf, AsRust, CDrop, RawPointerConverter)]
//! # #[target_type(Sauce)]
//! # pub struct CSauce { pub spiciness: c_float }
//! # pub struct Pizza {
//! #     pub name: String,
//! #     pub base: Option<Sauce>,
//! #     pub weight: f32,
//! # }
//! # #[repr(C)]
//! # #[derive(CReprOf, AsRust, CDrop, RawPointerConverter)]
//! # #[target_type(Pizza)]
//! # pub struct CPizza {
//! #     pub name: *const c_char,
//! #     #[nullable]
//! #     pub base: *const CSauce,
//! #     pub weight: c_float,
//! # }
//! // Read a CPizza handed to us by C: deep-copy its contents into an owned
//! // Rust `Pizza`, then run whatever logic we need. The original pointer is
//! // untouched; C keeps ownership of it.
//! #[unsafe(no_mangle)]
//! pub unsafe extern "C" fn inspect_pizza(c_pizza: *const CPizza) {
//!     let c_pizza = unsafe { CPizza::raw_borrow(c_pizza) }
//!         .expect("c_pizza must not be null");
//!     let pizza: Pizza = c_pizza.as_rust().expect("invalid CPizza contents");
//!     println!("{} ({}g)", pizza.name, pizza.weight);
//! }
//!
//! // Build a Rust `Pizza`, convert it to `CPizza`, and hand C a raw pointer
//! // via [`RawPointerConverter::into_raw_pointer`]. The caller must
//! // eventually invoke `free_pizza` to release the allocation.
//! #[unsafe(no_mangle)]
//! pub extern "C" fn make_pizza() -> *const CPizza {
//!     let pizza = Pizza {
//!         name: "Margarita".to_owned(),
//!         base: Some(Sauce { spiciness: 1.5 }),
//!         weight: 450.0,
//!     };
//!     CPizza::c_repr_of(pizza)
//!         .expect("pizza name contains an interior NUL byte")
//!         .into_raw_pointer()
//! }
//!
//! // Reclaim a pointer produced by `make_pizza`.
//! // [`RawPointerConverter::drop_raw_pointer`] takes ownership back and
//! // drops the value, releasing every inner pointer field via [`CDrop`].
//! #[unsafe(no_mangle)]
//! pub unsafe extern "C" fn free_pizza(c_pizza: *const CPizza) {
//!     let _ = unsafe { CPizza::drop_raw_pointer(c_pizza) };
//! }
//! ```
//!
//! # Type mapping
//!
//! `T: CReprOf<U> + AsRust<U>` — in the table below, `T` is the C-compatible
//! Rust type and `U` is the idiomatic Rust type.
//!
//! | C type                 | Rust type (`U`)   | C-compatible Rust type (`T`)                                                                                        | Provided by                  |
//! |------------------------|-------------------|---------------------------------------------------------------------------------------------------------------------|------------------------------|
//! | any scalar (`int`, …)  | same scalar       | same scalar                                                                                                         | `ffi-convert`                |
//! | `const char*`          | `String`          | `*const libc::c_char`                                                                                               | `ffi-convert`                |
//! | `const T*`             | `U`               | `*const T`                                                                                                          | `ffi-convert`                |
//! | `T*`                   | `U`               | `*mut T`                                                                                                            | `ffi-convert`                |
//! | `const T*` (nullable)  | `Option<U>`       | `*const T` with `#[nullable]`                                                                                       | `ffi-convert`                |
//! | `T[N]`                 | `[U; N]`          | `[T; N]`                                                                                                            | `ffi-convert`                |
//! | `CArrayT`              | `Vec<U>`          | [`CArray<T>`](https://docs.rs/ffi-convert-extra-ctypes/latest/ffi_convert_extra_ctypes/struct.CArray.html)          | `ffi-convert-extra-ctypes`   |
//! | `CStringArray`         | `Vec<String>`     | [`CStringArray`](https://docs.rs/ffi-convert-extra-ctypes/latest/ffi_convert_extra_ctypes/struct.CStringArray.html) | `ffi-convert-extra-ctypes`   |
//! | `CRangeT`              | `Range<U>`        | [`CRange<T>`](https://docs.rs/ffi-convert-extra-ctypes/latest/ffi_convert_extra_ctypes/struct.CRange.html)          | `ffi-convert-extra-ctypes`   |
//!
//! The derives accept both `*const T` and `*mut T` for any pointer row.
//!
//! # Traits at a glance
//!
//! | Trait                    | Direction            | Purpose                                                                                               |
//! |--------------------------|----------------------|-------------------------------------------------------------------------------------------------------|
//! | [`CReprOf<U>`]           | Rust → C             | Consume an idiomatic Rust value and produce its C-compatible twin.                                    |
//! | [`AsRust<U>`]            | C → Rust             | Produce an owned Rust value from a borrowed C-compatible value.                                       |
//! | [`CDrop`]                | cleanup              | Free heap data owned by a C-compatible struct.                                                        |
//! | [`RawPointerConverter`]  | pointer boxing       | Box a value into `*const T` / `*mut T` and take it back.                                              |
//! | [`RawBorrow`]            | pointer borrowing    | Borrow `&T` from a raw pointer without taking ownership. Returns an error if the pointer is null.     |
//! | [`RawBorrowMut`]         | pointer borrowing    | Borrow `&mut T` from a raw pointer without taking ownership. Returns an error if the pointer is null. |
//!
//! [`CReprOf`], [`AsRust`], [`CDrop`], and [`RawPointerConverter`] all have
//! derive macros.
//!
//! # Deriving the traits
//!
//! The derives are the intended way to use the crate. Typical derive
//! combinations on a `#[repr(C)]` type are:
//!
//! - `#[derive(CReprOf, CDrop)]` for a type created in Rust and read from C
//! - `#[derive(AsRust)]` for a type created in C and read in Rust
//! - `#[derive(AsRust, CReprOf, CDrop)]` for a type created and read in C and Rust
//!
//! Deriving `CDrop` and `CReprOf` together is recommended: `CDrop` assumes raw
//! pointers were initialized the way the `CReprOf` derive initializes them.
//!
//! The derives expect:
//!
//! - `#[target_type(Path)]` on every struct or enum that derives [`CReprOf`]
//!   or [`AsRust`], pointing at the idiomatic Rust type being mirrored.
//! - `#[nullable]` on every pointer field whose Rust counterpart is an
//!   [`Option`]. The attribute is shared by all three derives: [`CReprOf`]
//!   reads it to emit a null for `None`, [`AsRust`] to return `None` on a
//!   null pointer, and [`CDrop`] to skip the free on null. A mismatch
//!   between the Rust-side `Option<T>` and the C-side `#[nullable]` is a
//!   compile error.
//! - [`RawPointerConverter`] to be implemented on any nested C-compatible
//!   struct reached through a pointer field, usually by
//!   `#[derive(RawPointerConverter)]`.
//!
//! The available attributes are:
//!
//! | Attribute                                | Applies to              | Used by                     | Purpose                                                                                                                                       |
//! |------------------------------------------|-------------------------|-----------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------|
//! | `#[target_type(Path)]`                   | struct / enum           | `CReprOf`, `AsRust`         | The idiomatic Rust type this C-compatible type mirrors.                                                                                       |
//! | `#[no_drop_impl]`                        | struct / enum           | `CDrop`                     | Only implement [`CDrop`]; skip the blanket [`Drop`] impl so you can write one manually.                                                       |
//! | `#[as_rust_extra_field(name = expr)]`    | struct                  | `AsRust`                    | Initialize an extra field on the Rust side that has no C counterpart. Repeatable; `self` (the C-side value) is in scope inside `expr`.        |
//! | `#[nullable]`                            | pointer field           | `CReprOf`, `AsRust`, `CDrop`| Treat a `*const T` / `*mut T` as `Option<…>`. Required for every optional pointer field.                                                      |
//! | `#[target_name(ident)]`                  | field                   | `CReprOf`, `AsRust`         | Name of the corresponding field on the Rust side when it differs from the C-side name.                                                        |
//! | `#[c_repr_of_convert(expr)]`             | field                   | `CReprOf`, `AsRust`         | Override the `CReprOf` conversion with a custom expression. The owned `input: TargetType` is in scope. The field is also skipped by `AsRust`. |
//!
//! ## Constraints
//!
//! - **C strings**: a field is recognized as a C string only when the
//!   pointee's type name is literally `c_char` — `*const libc::c_char`,
//!   `*mut libc::c_char`, and `*const c_char` all qualify. A `type` alias
//!   for `c_char` is not recognized.
//! - **Multi-level pointer fields** (such as `*const *const CFoo`) are
//!   accepted by the [`AsRust`] derive only when the field is also
//!   `#[nullable]`.
//! - **Enums with data**:not supported. the derives accept enums only when
//!   every variant is a unit variant.
//!
//! # Interop checklist
//!
//! A typical FFI-exposed function follows this pattern:
//!
//! 1. Receive a `*const CInput` from C and convert it with [`AsRust`], or
//!    borrow it with [`RawBorrow`] if the C side keeps ownership.
//! 2. Run the Rust logic.
//! 3. Build a `COutput` with [`CReprOf`] and return it to C via
//!    [`RawPointerConverter::into_raw_pointer`].
//! 4. Expose a `free`-style function that takes the pointer back with
//!    [`RawPointerConverter::from_raw_pointer`] and lets the value drop.

pub use ffi_convert_derive::*;

mod conversions;

pub use conversions::*;
