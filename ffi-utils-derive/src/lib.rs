extern crate proc_macro;

use proc_macro::TokenStream;

use syn;

use quote::quote;

#[proc_macro_derive(CReprOf, attributes(target_type, nullable))]
pub fn creprof_derive(token_stream: TokenStream) -> TokenStream {
    let ast = syn::parse(token_stream).unwrap();
    impl_creprof_macro(&ast)
}

fn impl_creprof_macro(input: &syn::DeriveInput) -> TokenStream {
    let struct_name = &input.ident;
    let target_type = parse_target_type(&input.attrs);

    let fields = parse_data_to_fields(&input.data)
        .iter()
        .map(|(field_name, field_type, is_str, is_nullable)| {
            match (is_nullable, is_str) {
                (true, true) =>
                    quote!(
                        #field_name: if let Some(value) = input.#field_name {
                            convert_to_c_string_result!(value)?
                        } else {
                            std::ptr::null() as _
                        }
                    ),
                (true, false) =>
                    quote!(
                        #field_name: if let Some(value) = input.#field_name {
                            #field_type::c_repr_of(value)?
                        } else {
                            std::ptr::null() as _
                        }
                    ),
                (false, true) =>
                    quote!(#field_name: convert_to_c_string_result!(input.#field_name)?),
                (false, false) =>
                    quote!(#field_name: #field_type ::c_repr_of(input.#field_name)?),
            }
        })
        .collect::<Vec<_>>();

    quote!(
        impl CReprOf<# target_type> for # struct_name {
            fn c_repr_of(input: # target_type) -> Result<Self, ffi_utils::Error> {
                use failure::ResultExt;
                Ok(Self {
                    # ( # fields, )*
                })
            }
        }
    ).into()
}

#[proc_macro_derive(AsRust, attributes(target_type, nullable))]
pub fn asrust_derive(token_stream: TokenStream) -> TokenStream {
    let ast = syn::parse(token_stream).unwrap();
    impl_asrust_macro(&ast)
}

fn impl_asrust_macro(input: &syn::DeriveInput) -> TokenStream {
    let struct_name = &input.ident;
    let target_type = parse_target_type(&input.attrs);

    let fields = parse_data_to_fields(&input.data)
        .iter()
        .map(|(field_name, _, is_str, is_nullable)| {
            match (is_nullable, is_str) {
                (true, true) =>
                    quote!(
                        #field_name: if self.#field_name != std::ptr::null() {
                            Some(ffi_utils::create_rust_string_from!(self.#field_name))
                        } else {
                            None
                        }
                    ),
                (true, false) =>
                    quote!(
                        #field_name: if self.#field_name != std::ptr::null() {
                            Some(self.#field_name.as_rust()?)
                        } else {
                            None
                        }
                    ),
                (false, true) =>
                    quote!(#field_name : ffi_utils::create_rust_string_from!(self.#field_name)),
                (false, false) =>
                    quote!(#field_name : self.#field_name.as_rust()?)
            }
        })
        .collect::<Vec<_>>();

    quote!(
        impl AsRust<#target_type> for #struct_name {
            fn as_rust(&self) -> Result<#target_type, ffi_utils::Error> {
                use failure::ResultExt;
                Ok(#target_type {
                    #(#fields, )*
                })
            }
        }
    ).into()
}

fn parse_target_type(attrs: &Vec<syn::Attribute>) -> syn::Path {
    let target_type_attribute= attrs
        .iter()
        .find(|attribute| {
            attribute.path.get_ident().map(|it| it.to_string()) == Some("target_type".into())
        })
        .expect("Can't derive CReprOf without target_type helper attribute.");

    target_type_attribute.parse_args().unwrap()
}

fn parse_data_to_fields(data: &syn::Data) -> Vec<(&syn::Ident, proc_macro2::TokenStream, bool, bool)> {
    match &data {
        syn::Data::Struct(data_struct) =>
            data_struct.fields
                .iter()
                .map(|field| parse_field(field))
                .collect::<Vec<(&syn::Ident, proc_macro2::TokenStream, bool, bool)>>(),
        _ => panic!("CReprOf / AsRust can only be derived for structs"),
    }
}

fn parse_field(field: &syn::Field) -> (&syn::Ident, proc_macro2::TokenStream, bool, bool) {
    let field_name = field.ident.as_ref().expect("Field should have an ident");

    let is_nullable = field.attrs.iter().find(|attr| {
        attr.path.get_ident().map(|it| it.to_string()) == Some("nullable".into())
    }).is_some();

    let (field_type, is_str) = match &field.ty {
        syn::Type::Ptr(ptr_t) => {
            match &*ptr_t.elem {
                syn::Type::Path(path_t) => {
                    // Check if it's string type
                    let is_str = path_t
                        .path
                        .segments
                        .iter()
                        .find(|it| it.ident.to_string().contains("c_char"));

                    match is_str {
                        Some(_) => (quote!(ffi_utils::RawPointerTo::< #path_t >), true),
                        None => (quote!(ffi_utils::RawPointerTo::< #path_t >), false)
                    }
                }
                _ => panic!("Pointer type is not supported")
            }
        }
        syn::Type::Path(path_t) => (generic_path_to_concrete_type_path(&path_t.path), false),
        _ => { panic!("Field type is not supported") }
    };

    (field_name, field_type, is_str, is_nullable)
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
