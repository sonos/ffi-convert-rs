//! Derive macros for the conversion traits provided by the
//! [`ffi-convert`](https://docs.rs/ffi-convert) crate.
//!
//! The four derives are also re-exported from `ffi-convert`, so users of
//! `ffi-convert` typically do not need to depend on this crate directly.
//!
//! See the top-level [`ffi-convert`](https://docs.rs/ffi-convert) documentation
//! for the overall design, the type-mapping table, and the caveats that apply
//! to all four derives. The per-macro documentation below lists the supported
//! helper attributes.

extern crate proc_macro;

mod asrust;
mod cdrop;
mod creprof;
mod rawpointerconverter;
mod utils;

use asrust::impl_asrust_macro;
use cdrop::impl_cdrop_macro;
use creprof::impl_creprof_macro;
use proc_macro::TokenStream;
use rawpointerconverter::impl_rawpointerconverter_macro;

/// Derive [`CReprOf<T>`](../ffi_convert/trait.CReprOf.html) for a struct or unit enum.
///
/// Generates a consuming conversion from the idiomatic Rust type named in
/// `#[target_type(...)]` to `Self`. String fields (`*const c_char`) are
/// re-allocated as C strings, pointer fields are boxed via
/// [`RawPointerConverter::into_raw_pointer`](../ffi_convert/trait.RawPointerConverter.html),
/// and scalar fields are passed through unchanged.
///
/// `CReprOf` and [`CDrop`](../ffi_convert/trait.CDrop.html) share an
/// ownership contract: each pointer field produced by `c_repr_of` is a
/// `Box::into_raw`'d value that the derived `CDrop` reclaims with
/// `Box::from_raw`. The recommended shape is
/// `#[derive(CReprOf, AsRust, CDrop)]` as a set — deriving `CReprOf` while
/// hand-writing `CDrop` (or the reverse) breaks the contract and causes
/// double frees or leaks. For per-field customization, prefer
/// `#[c_repr_of_convert]` / `#[as_rust_extra_field]` over hand-writing one
/// of the impls.
///
/// # Struct-level attributes
///
/// - `#[target_type(Path)]` — **required**. The idiomatic Rust type this
///   `#[repr(C)]` struct mirrors.
///
/// # Field-level attributes
///
/// - `#[nullable]` — required on every pointer field whose Rust counterpart is
///   an [`Option`]. A `None` value is written as a null pointer.
/// - `#[target_name(ident)]` — name of the matching field on the Rust side
///   when it differs from the C-side name.
/// - `#[c_repr_of_convert(expr)]` — replace the generated conversion with a
///   custom expression. The owned Rust value `input: TargetType` is in scope.
///   A field marked with this attribute is also skipped by the `AsRust`
///   derive — if the reverse direction is needed, provide it with
///   `#[as_rust_extra_field(...)]` on the struct.
///
/// # Enums
///
/// Enums are supported only if every variant is a unit variant.
#[proc_macro_derive(
    CReprOf,
    attributes(target_type, nullable, c_repr_of_convert, target_name)
)]
pub fn creprof_derive(token_stream: TokenStream) -> TokenStream {
    let ast = syn::parse(token_stream).unwrap();
    impl_creprof_macro(&ast)
}

/// Derive [`AsRust<T>`](../ffi_convert/trait.AsRust.html) for a struct or unit enum.
///
/// Generates a non-consuming conversion that returns a freshly-allocated value
/// of the type named in `#[target_type(...)]`. C strings are decoded as UTF-8
/// and copied; pointer fields are borrowed via
/// [`RawBorrow`](../ffi_convert/trait.RawBorrow.html) and then recursively
/// converted with `AsRust`.
///
/// The derived `AsRust` reads pointer fields under the same ownership
/// contract that `CReprOf` / `CDrop` use (each pointer is a
/// `Box::into_raw`'d value). If you derive any of the three, derive all of
/// them; mixing a derived `AsRust` with a hand-written `CReprOf` that
/// allocates pointer fields differently is undefined behavior.
///
/// # Struct-level attributes
///
/// - `#[target_type(Path)]` — **required**.
/// - `#[as_rust_extra_field(name = expr)]` — initialise an extra field on the
///   Rust side that has no C counterpart. The attribute can be repeated; `self`
///   (the C-compatible value) is in scope inside `expr`, allowing
///   reconstruction from unrelated C-side fields.
///
/// # Field-level attributes
///
/// - `#[nullable]` — map a null pointer to [`None`] instead of failing.
/// - `#[target_name(ident)]` — name of the matching field on the Rust side
///   when it differs from the C-side name.
///
/// A field annotated with `#[c_repr_of_convert(...)]` (see [`CReprOf`]) is
/// skipped by this derive; pair it with `#[as_rust_extra_field]` if the Rust
/// struct still has a matching field.
///
/// # Enums
///
/// Enums are supported only if every variant is a unit variant.
#[proc_macro_derive(
    AsRust,
    attributes(
        target_type,
        nullable,
        as_rust_extra_field,
        as_rust_ignore,
        target_name
    )
)]
pub fn asrust_derive(token_stream: TokenStream) -> TokenStream {
    let ast = syn::parse(token_stream).unwrap();
    impl_asrust_macro(&ast)
}

/// Derive [`CDrop`](../ffi_convert/trait.CDrop.html) and (by default) [`Drop`]
/// for a struct or unit enum.
///
/// The generated `do_drop` takes back ownership of every owning pointer field
/// with [`RawPointerConverter::from_raw_pointer`](../ffi_convert/trait.RawPointerConverter.html)
/// and lets it drop. Scalar and array fields are left to Rust's regular drop
/// glue.
///
/// Deriving [`CDrop`] assumes the struct owns its pointer fields — typically
/// because it was built via `CReprOf::c_repr_of`. Dropping a value built from
/// borrowed or shared pointers is undefined behavior.
///
/// `CDrop` and [`CReprOf`](../ffi_convert/trait.CReprOf.html) must agree on
/// how pointer fields are allocated: derive both together, or write both by
/// hand. Mixing a derived `CDrop` with a hand-written `CReprOf` (or the
/// reverse) breaks the contract and leads to double frees or leaks.
///
/// The default output is the intended shape: both impls should coexist, so
/// that letting a value go out of scope actually frees the memory. A
/// `CDrop` impl without a matching `Drop` silently leaks every pointer
/// field.
///
/// # Struct-level attributes
///
/// - `#[no_drop_impl]` — generate only the `CDrop` impl and skip the blanket
///   `Drop` impl. Use this when you need a manual `Drop`; that manual `Drop`
///   must call `do_drop`, otherwise the struct leaks.
///
/// # Field-level attributes
///
/// - `#[nullable]` — skip the free when the pointer is null. This is the
///   same attribute that `CReprOf` and `AsRust` read on the field; annotate
///   the field once and all three derives agree.
#[proc_macro_derive(CDrop, attributes(no_drop_impl, nullable))]
pub fn cdrop_derive(token_stream: TokenStream) -> TokenStream {
    let ast = syn::parse(token_stream).unwrap();
    impl_cdrop_macro(&ast)
}

/// Derive [`RawPointerConverter<Self>`](../ffi_convert/trait.RawPointerConverter.html)
/// for a struct.
///
/// The derived implementation boxes `self` into `*const Self` / `*mut Self`
/// (and conversely). It is needed on any C-compatible struct that is reached
/// through a raw pointer field in another C-compatible struct, because the
/// derived [`CReprOf`] of the parent calls `into_raw_pointer()` on it.
///
/// No helper attributes.
#[proc_macro_derive(RawPointerConverter)]
pub fn rawpointerconverter_derive(token_stream: TokenStream) -> TokenStream {
    let ast = syn::parse(token_stream).unwrap();
    impl_rawpointerconverter_macro(&ast)
}
