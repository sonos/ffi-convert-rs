use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, quote_spanned};

/// `#[derive(CDrop)]` is deprecated: the trait has been merged into
/// [`CReprOf`](ffi_convert::CReprOf), whose derive now also emits the
/// [`Drop`] impl. The macro remains as a no-op (apart from a deprecation
/// warning) so existing `#[derive(CReprOf, AsRust, CDrop, ...)]` annotations
/// keep compiling.
pub fn impl_cdrop_macro(input: &syn::DeriveInput) -> TokenStream {
    // The deprecation warning fires at the `use` of the const. Anchor that
    // use to `Span::call_site()` — which for a derive points at the macro
    // invocation, i.e. the `CDrop` ident inside `#[derive(...)]`. The
    // declaration uses the type's own span so the const itself doesn't trip
    // any other diagnostics on the user's `CDrop` ident.
    let decl_span = input.ident.span();
    let use_span = Span::mixed_site();

    let decl_const = syn::Ident::new(
        &format!("_FFI_CONVERT_CDROP_DEPRECATED_{}", input.ident),
        decl_span,
    );
    let use_const = syn::Ident::new(
        &format!("_FFI_CONVERT_CDROP_DEPRECATED_{}", input.ident),
        use_span,
    );

    let decl = quote_spanned!(decl_span =>
        #[doc(hidden)]
        #[deprecated(
            note = "`CDrop` has been merged into `CReprOf`; this derive is a no-op and \
                    can be removed — the cleanup hook is now emitted by \
                    `#[derive(CReprOf)]`."
        )]
        #[allow(non_upper_case_globals)]
        const #decl_const: () = ();
    );
    let trigger = quote_spanned!(use_span =>
        const _: () = #use_const;
    );

    quote!(#decl #trigger).into()
}
