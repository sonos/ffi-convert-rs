use proc_macro::TokenStream;
use quote::{quote, quote_spanned};

/// `#[derive(CDrop)]` is deprecated: `#[derive(CReprOf)]` now emits the
/// [`Drop`] impl that frees the type's pointer fields. The macro remains as
/// a no-op (apart from a deprecation warning) so existing
/// `#[derive(CReprOf, AsRust, CDrop, ...)]` annotations keep compiling.
pub fn impl_cdrop_macro(input: &syn::DeriveInput) -> TokenStream {
    // The span must be borrowed from a user token (the type's ident): rustc
    // suppresses deprecation lints on macro-generated spans like
    // `Span::call_site()`.
    let span = input.ident.span();

    let deprecated_const = syn::Ident::new(
        &format!("_FFI_CONVERT_CDROP_DEPRECATED_{}", input.ident),
        span,
    );

    let decl = quote_spanned!(span =>
        #[doc(hidden)]
        #[deprecated(
            note = "`CDrop` is deprecated; this derive is a no-op and can be removed — \
                    `#[derive(CReprOf)]` now emits a `Drop` impl freeing the type's \
                    pointer fields when needed."
        )]
        #[allow(non_upper_case_globals)]
        const #deprecated_const: () = ();
    );
    let trigger = quote_spanned!(span =>
        const _: () = #deprecated_const;
    );

    quote!(#decl #trigger).into()
}
