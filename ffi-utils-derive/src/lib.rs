extern crate proc_macro;

use proc_macro::TokenStream;

use syn;
use syn::Type;

use quote::quote;

#[proc_macro_derive(CReprOf, attributes(converted))]
pub fn creprof_derive(token_stream: TokenStream) -> TokenStream {
    let ast = syn::parse(token_stream).unwrap();
    impl_creprof_macro(&ast)
}

fn impl_creprof_macro(input: &syn::DeriveInput) -> TokenStream {
    let data = match &input.data {
        syn::Data::Struct(data) => data,
        _ => panic!("CReprOf can only be derived for structs"),
    };

    let struct_name = &input.ident;

    let converted_attribute: &syn::Attribute = input
        .attrs
        .iter()
        .find(|attribute| {
            attribute.path.get_ident().map(|it| it.to_string()) == Some("converted".into())
        })
        .expect("Can't derive CReprOf without converted helper attribute.");

    let target_type: syn::Path = converted_attribute.parse_args().unwrap();

    let fields: Vec<_> = data.fields.iter()
        .map(|field|
            (field.ident.as_ref().expect("field should have an ident"),
             match &field.ty {
                 Type::Ptr(ptr_t) => { match &*ptr_t.elem {
                     Type::Path(path_t) => quote!(RawPointerTo::< #path_t >),
                     _ => panic!("")
                 }}
                 Type::Path(path_t) => { generic_path_to_concrete_type_path(&path_t.path) }
                 _ => { panic!("") }
             }))
        .map(|(field_name, field_type)|
            quote!(#field_name: #field_type ::c_repr_of(input.#field_name)?)
        )
        .collect::<Vec<_>>();

    quote!(
        impl CReprOf<# target_type> for # struct_name {
            fn c_repr_of(input: # target_type) -> Result<Self, Error> {
                Ok(Self {
                    # ( # fields, )*
                })
            }
        }
    )
    .into()
}

fn generic_path_to_concrete_type_path(path: &syn::Path) -> proc_macro2::TokenStream {
    let mut path = path.clone();
    let last_segment = path.segments.pop().unwrap();
    let segments = &path.segments;
    let ident = &last_segment.value().ident;
    let turbofished_type = if let syn::PathArguments::AngleBracketed(bracketed_args) =
        &last_segment.value().arguments
    {
        quote!(#ident::#bracketed_args)
    } else {
        quote!(#ident)
    };
    if segments.is_empty() {
        turbofished_type
    } else {
        quote!(#segments::#turbofished_type)
    }
}

#[proc_macro_derive(AsRust, attributes(converted))]
pub fn asrust_derive(token_stream: TokenStream) -> TokenStream {
    let ast = syn::parse(token_stream).unwrap();
    impl_asrust_macro(&ast)
}

fn impl_asrust_macro(input: &syn::DeriveInput) -> TokenStream {
    let struct_name = &input.ident;
    let converted_attribute: &syn::Attribute = input
        .attrs
        .iter()
        .find(|attribute| {
            attribute.path.get_ident().map(|it| it.to_string()) == Some("converted".into())
        })
        .expect("Can't derive CReprOf without converted helper attribute.");

    let target_type: syn::Path = converted_attribute.parse_args().unwrap();

    let data = match &input.data {
        syn::Data::Struct(data) => data,
        _ => panic!("CReprOf can only be derived for structs"),
    };

    let fields: Vec<_> = data
        .fields
        .iter()
        .map(|field| field.ident.as_ref().expect("field should have an ident"))
        .map(|field_name| quote!(#field_name : self.#field_name .as_rust()?))
        .collect::<Vec<_>>();

    quote!(
        impl AsRust<#target_type> for #struct_name {
            fn as_rust(&self) -> Result<#target_type, Error> {
                Ok(#target_type {
                    #(#fields, )*
                })
            }
        }
    )
    .into()
}
