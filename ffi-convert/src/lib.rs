//! Traits and helpers to convert between idiomatic Rust values and C-compatible
//! representations when crossing an FFI boundary.
//!
//! The crate is built around two conversion traits,
//! [`CReprOf`] and [`AsRust`], and two supporting traits,
//! [`CDrop`] and [`RawPointerConverter`]. Together they form a small framework
//! that makes it hard to get ownership wrong while moving data across the
//! boundary in either direction. Deriving them (via the companion
//! [`ffi-convert-derive`](https://docs.rs/ffi-convert-derive) crate, re-exported
//! here) removes most of the boilerplate.
//!
//! Common containers (arrays, string arrays, ranges) live in the separate
//! [`ffi-convert-extra-ctypes`](https://docs.rs/ffi-convert-extra-ctypes) crate
//! so that users who want to define their own C layouts can skip them.
//!
//! # Philosophy
//!
//! When a pointer crosses the FFI boundary, decide *once* which side owns it:
//!
//! - **Incoming from C**: immediately convert to an owned, idiomatic Rust value
//!   with [`AsRust`]. After that call the Rust value is fully self-contained
//!   and the C-side struct can be dropped or handed back to the caller.
//! - **Outgoing to C**: build a C-compatible struct from an owned Rust value
//!   with [`CReprOf`]. That struct now owns whatever heap memory it points to
//!   and must eventually be freed via [`CDrop`] (typically by sending the
//!   pointer back to Rust through a `free`-style FFI function that calls
//!   [`RawPointerConverter::from_raw_pointer`]).
//!
//! # Quick example
//!
//! Define the Rust type you want to expose, then define its `#[repr(C)]`
//! mirror and derive the conversion traits. The mirror's fields use
//! C-compatible types (see [the mapping table](#type-mapping)).
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
//! #[derive(CReprOf, AsRust, CDrop)]
//! #[target_type(Pizza)]
//! pub struct CPizza {
//!     pub name: *const c_char,
//!     #[nullable]
//!     pub base: *const CSauce,
//!     pub weight: c_float,
//! }
//!
//! // Rust -> C
//! let pizza = Pizza {
//!     name: "Margarita".to_owned(),
//!     base: Some(Sauce { spiciness: 1.5 }),
//!     weight: 450.0,
//! };
//! let c_pizza = CPizza::c_repr_of(pizza).unwrap();
//!
//! // C -> Rust
//! let pizza_again: Pizza = c_pizza.as_rust().unwrap();
//! assert_eq!(pizza_again.name, "Margarita");
//! // `c_pizza` still owns the C strings it allocated; its `Drop` impl will
//! // free them when it goes out of scope.
//! ```
//!
//! Two things to notice:
//!
//! - `CSauce` derives [`RawPointerConverter`] because `CPizza::base` stores a
//!   `*const CSauce`, and the derived [`CReprOf`] for `CPizza` needs to
//!   convert a `CSauce` value into that raw pointer.
//! - `CPizza::base` is annotated with `#[nullable]` because the Rust field is
//!   `Option<Sauce>`; without the attribute the derive would not know to map
//!   `None` to the null pointer.
//!
//! # Type mapping
//!
//! `T: CReprOf<U> + AsRust<U>` — in the table below, `T` is the C-compatible
//! type and `U` is the idiomatic Rust type.
//!
//! | C type                 | Rust type (`U`)   | C-compatible Rust type (`T`)                                                                                | Provided by                  |
//! |------------------------|-------------------|-------------------------------------------------------------------------------------------------------------|------------------------------|
//! | any scalar (`int`, …)  | same scalar       | same scalar                                                                                                 | `ffi-convert`                |
//! | `const char*`          | `String`          | `*const libc::c_char`                                                                                       | `ffi-convert`                |
//! | `const T*`             | `U`               | `*const T`                                                                                                  | `ffi-convert`                |
//! | `T*`                   | `U`               | `*mut T`                                                                                                    | `ffi-convert`                |
//! | `const T*` (nullable)  | `Option<U>`       | `*const T` with `#[nullable]`                                                                               | `ffi-convert`                |
//! | `T[N]`                 | `[U; N]`          | `[T; N]`                                                                                                    | `ffi-convert`                |
//! | `CArrayT`              | `Vec<U>`          | [`CArray<T>`](https://docs.rs/ffi-convert-extra-ctypes/latest/ffi_convert_extra_ctypes/struct.CArray.html) | `ffi-convert-extra-ctypes`   |
//! | `CStringArray`         | `Vec<String>`     | [`CStringArray`](https://docs.rs/ffi-convert-extra-ctypes/latest/ffi_convert_extra_ctypes/struct.CStringArray.html) | `ffi-convert-extra-ctypes`   |
//! | `CRangeT`              | `Range<U>`        | [`CRange<T>`](https://docs.rs/ffi-convert-extra-ctypes/latest/ffi_convert_extra_ctypes/struct.CRange.html) | `ffi-convert-extra-ctypes`   |
//!
//! Users are free to define additional C-compatible layouts for their own
//! container-like types by implementing the traits directly.
//!
//! # Traits at a glance
//!
//! | Trait                    | Direction            | Purpose                                                                       |
//! |--------------------------|----------------------|-------------------------------------------------------------------------------|
//! | [`CReprOf<U>`]           | Rust → C             | Consume an idiomatic Rust value and produce its C-compatible twin.            |
//! | [`AsRust<U>`]            | C → Rust             | Produce an owned Rust value from a borrowed C-compatible view.                |
//! | [`CDrop`]                | cleanup              | Free heap data owned by a C-compatible struct.                                |
//! | [`RawPointerConverter`]  | pointer boxing       | Box a value into `*const T` / `*mut T` and take it back.                      |
//! | [`RawBorrow`]            | pointer borrowing    | Borrow `&T` from a raw pointer without taking ownership.                      |
//!
//! All four conversion traits can be derived. The derives are re-exported from
//! this crate; see [`ffi-convert-derive`](https://docs.rs/ffi-convert-derive)
//! for the full list of supported attributes.
//!
//! # Deriving the traits
//!
//! The derives cover the common cases. They expect:
//!
//! - [`CReprOf`], [`AsRust`], and [`CDrop`] are derived **as a set** on the
//!   same type, or all written by hand. The three impls share an ownership
//!   contract (each pointer field is a `Box::into_raw`'d value, owned by
//!   the C-compatible struct), and the derives rely on that contract. Mixing
//!   a derived [`CDrop`] with a hand-written [`CReprOf`] that allocates
//!   differently — or the other way around — will cause double frees, leaks,
//!   or UB. If one of them needs custom behavior for a specific field,
//!   reach for `#[c_repr_of_convert]` / `#[as_rust_extra_field]` instead of
//!   writing one impl by hand.
//! - `#[target_type(Path)]` on every struct or enum that derives [`CReprOf`]
//!   or [`AsRust`], pointing at the idiomatic Rust type being mirrored.
//!   [`CDrop`] and [`RawPointerConverter`] do not need it.
//! - `#[nullable]` on every pointer field whose Rust counterpart is an
//!   [`Option`]. The attribute is shared by all three derives: [`CReprOf`]
//!   reads it to emit a null for `None`, [`AsRust`] to return `None` on a
//!   null pointer, and [`CDrop`] to skip the free on null.
//! - Any nested C-compatible struct used behind a pointer implements
//!   [`RawPointerConverter`] (typically via `#[derive(RawPointerConverter)]`).
//!
//! The available attributes are:
//!
//! | Attribute                                | Applies to              | Used by                     | Purpose                                                                                                  |
//! |------------------------------------------|-------------------------|-----------------------------|----------------------------------------------------------------------------------------------------------|
//! | `#[target_type(Path)]`                   | struct / enum           | `CReprOf`, `AsRust`         | The idiomatic Rust type this C-compatible type mirrors.                                                  |
//! | `#[nullable]`                            | pointer field           | `CReprOf`, `AsRust`, `CDrop`| Treat a `*const T` / `*mut T` as `Option<…>`. Required for every optional pointer field.                 |
//! | `#[target_name(ident)]`                  | field                   | `CReprOf`, `AsRust`         | Name of the corresponding field on the Rust side when it differs from the C-side name.                   |
//! | `#[c_repr_of_convert(expr)]`             | field                   | `CReprOf`, `AsRust`         | Override the `CReprOf` conversion with a custom expression. The owned `input: TargetType` is in scope. Excludes the field from `AsRust`. |
//! | `#[as_rust_extra_field(name = expr)]`    | struct                  | `AsRust`                    | Initialise an extra field on the Rust side that has no C counterpart. Repeatable.                        |
//! | `#[no_drop_impl]`                        | struct / enum           | `CDrop`                     | Only implement [`CDrop`]; skip the blanket [`Drop`] impl so you can write one manually.                  |
//!
//! Only unit enums (variants without fields) are supported by the derives for
//! enums.
//!
//! # Caveats with derivation of `CReprOf`, `AsRust`, and `CDrop`
//!
//! The derives are intentionally conservative. The most common pitfalls are:
//!
//! ## Derive all three, or hand-write all three
//!
//! [`CReprOf`], [`AsRust`], and [`CDrop`] share an ownership contract: the
//! derived [`CDrop`] assumes each pointer field was allocated by the derived
//! [`CReprOf`] with `Box::into_raw`, and the derived [`AsRust`] reads
//! pointer fields under the same assumption. Mixing a derived impl with a
//! hand-written one is how you end up with double frees, leaks, or UB. If a
//! single field needs custom handling, reach for `#[c_repr_of_convert]` or
//! `#[as_rust_extra_field]` instead of writing a full impl manually.
//!
//! ## Ownership of pointer fields
//!
//! A derived [`CDrop`] implementation assumes every pointer field points to a
//! [`Box`] allocated by this crate — typically because the struct was created
//! by [`CReprOf::c_repr_of`]. If you build the C struct manually with pointers
//! you do not own (for example pointers obtained from C, or stack addresses),
//! letting it drop will trigger undefined behavior.
//!
//! Similarly, **do not share inner pointers** between two owning C structs.
//! Both will try to free them.
//!
//! ## Pointers coming from C
//!
//! When a C-allocated struct is handed to Rust, convert it with
//! [`AsRust::as_rust`] as soon as possible to obtain an owned Rust value, and
//! leave the cleanup of the C-side memory to the C caller. Do **not** turn a
//! borrow into a `Box` (for example via [`RawPointerConverter::from_raw_pointer`])
//! unless Rust actually owns the allocation.
//!
//! ## `#[nullable]` and `Option` must agree
//!
//! If the Rust side is `Option<T>`, the C-side pointer field must carry
//! `#[nullable]` — otherwise the derives won't compile, because the generated
//! code tries to feed an `Option<T>` into `T::c_repr_of` (or write a `Some(…)`
//! into a non-optional Rust field). The single `#[nullable]` attribute is
//! shared by all three derives.
//!
//! ## Detecting C strings
//!
//! The derives treat a field as a C string only when its type spelling ends in
//! `c_char` (for example `*const libc::c_char` or `*const c_char`). Aliases
//! introduced via `type` declarations are not recognised — use the `c_char`
//! spelling directly, or implement [`CReprOf`] / [`AsRust`] manually.
//!
//! ## Multi-level pointer fields
//!
//! Fields with more than one level of pointer indirection (such as
//! `*const *const CFoo`) are not supported by the derives and must be
//! implemented manually.
//!
//! ## Only unit enums can be derived
//!
//! `#[derive(CReprOf, AsRust, CDrop)]` on an enum requires every variant to be
//! a unit variant. Enums carrying data need a manual implementation.
//!
//! ## `target_type` is required
//!
//! Both `CReprOf` and `AsRust` require a `#[target_type(...)]` attribute on
//! the struct or enum. Without it the derive panics at compile time.
//!
//! ## `#[c_repr_of_convert]` disables `AsRust` for that field
//!
//! A field annotated with `#[c_repr_of_convert]` is skipped by the `AsRust`
//! derive. If the Rust struct still has a matching field, provide it via
//! `#[as_rust_extra_field(name = expr)]` on the struct; otherwise the derive
//! will fail to compile.
//!
//! # Interop checklist
//!
//! When exposing a function through FFI, the typical flow is:
//!
//! 1. Receive a `*const CInput` from C, convert it to an owned `Input` with
//!    [`AsRust`] (or use [`RawBorrow`] to inspect without taking ownership).
//! 2. Run the Rust logic.
//! 3. Build a `COutput` with [`CReprOf`] and return it to C via
//!    [`RawPointerConverter::into_raw_pointer`].
//! 4. Expose a `free`-style function that the C caller invokes when it is
//!    done; that function should take back ownership with
//!    [`RawPointerConverter::from_raw_pointer`] and let the value drop.

pub use ffi_convert_derive::*;

mod conversions;

pub use conversions::*;
