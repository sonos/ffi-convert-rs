extern crate proc_macro;

use proc_macro::TokenStream;

use syn;

use quote::quote;

#[proc_macro_derive(CReprOf, attributes(target_type, nullable, no_drop_impl, string))]
pub fn creprof_derive(token_stream: TokenStream) -> TokenStream {
    let ast = syn::parse(token_stream).unwrap();
    impl_creprof_macro(&ast)
}

fn impl_creprof_macro(input: &syn::DeriveInput) -> TokenStream {
    let struct_name = &input.ident;
    let target_type = parse_target_type(&input.attrs);
    let disable_drop_impl = parse_no_drop_impl_flag(&input.attrs);

    let fields = parse_struct_fields(&input.data);

    let c_repr_of_fields = fields
        .iter()
        .map(|(field_name, field_type, is_nullable_field, _is_string_field)| {
            if *is_nullable_field {
                quote!(
                    #field_name: if let Some(value) = input.#field_name {
                        #field_type::c_repr_of(value)?
                    } else {
                        std::ptr::null() as _
                    }
                )
            } else {
                quote!(#field_name: #field_type::c_repr_of(input.#field_name)?)
            }
        })
        .collect::<Vec<_>>();

    let do_drop_fields = fields
        .iter()
        .map(|(field_name, field_type, is_nullable_field, is_string_field)| {
            match (*is_nullable_field, *is_string_field) {
                (false, false) => {
                    quote!( self.# field_name.do_drop() )
                }
                (false, true) => {
                    quote!( take_back_c_string!(self.#field_name) )
                }
                (true, false) => {
                    quote!(
                        if !self.#field_name.is_null() {
                           self.#field_name.do_drop()
                        }
                    )
                }
                (true, true) => {
                    quote!(
                        if !self.#field_name.is_null() {
                           take_back_c_string!(self.#field_name)
                        }
                    )
                }
            }
        });

    let c_repr_of_impl = quote!(
        impl CReprOf<# target_type> for # struct_name {
            fn c_repr_of(input: # target_type) -> Result<Self, ffi_utils::Error> {
                use failure::ResultExt;
                Ok(Self {
                    # ( # c_repr_of_fields, )*
                })
            }
        }

        impl CDrop for # struct_name {
            fn do_drop(&mut self) {
                # ( #do_drop_fields );*
            }
        }
    );

    let drop_impl = quote!(
        impl Drop for # struct_name {
            fn drop(&mut self) {
                self.do_drop();
            }
        }
    );

    {
        if disable_drop_impl {
            quote! {
            # c_repr_of_impl
        }
        } else {
            quote! {
            # c_repr_of_impl
            # drop_impl
        }
        }
    }.into()
}

#[proc_macro_derive(AsRust, attributes(target_type, nullable))]
pub fn asrust_derive(token_stream: TokenStream) -> TokenStream {
    let ast = syn::parse(token_stream).unwrap();
    impl_asrust_macro(&ast)
}

fn impl_asrust_macro(input: &syn::DeriveInput) -> TokenStream {
    let struct_name = &input.ident;
    let target_type = parse_target_type(&input.attrs);

    let fields = parse_struct_fields(&input.data)
        .iter()
        .map(|(field_name, _, is_nullable, _is_string_field)| {
            match is_nullable {
                true =>
                    quote!(
                        #field_name: if !self.#field_name.is_null() {
                            Some(self.#field_name.as_rust()?)
                        } else {
                            None
                        }
                    ),
                false =>
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
    let target_type_attribute = attrs
        .iter()
        .find(|attribute| {
            attribute.path.get_ident().map(|it| it.to_string()) == Some("target_type".into())
        })
        .expect("Can't derive CReprOf without target_type helper attribute.");

    target_type_attribute.parse_args().unwrap()
}

fn parse_no_drop_impl_flag(attrs: &Vec<syn::Attribute>) -> bool {
    attrs
        .iter()
        .find(|attribute| {
            attribute.path.get_ident().map(|it| it.to_string()) == Some("no_drop_impl".to_string())
        })
        .is_some()
}

fn parse_struct_fields(data: &syn::Data) -> Vec<(&syn::Ident, proc_macro2::TokenStream, bool, bool)> {
    match &data {
        syn::Data::Struct(data_struct) =>
            data_struct.fields
                .iter()
                .map(parse_field)
                .collect::<Vec<(&syn::Ident, proc_macro2::TokenStream, bool, bool)>>(),
        _ => panic!("CReprOf / AsRust can only be derived for structs"),
    }
}

fn parse_field(field: &syn::Field) -> (&syn::Ident, proc_macro2::TokenStream, bool, bool) {
    let field_name = field.ident.as_ref().expect("Field should have an ident");

    let is_nullable_field = field.attrs.iter().find(|attr| {
        attr.path.get_ident().map(|it| it.to_string()) == Some("nullable".into())
    }).is_some();

    let is_string_field = field.attrs.iter().find(|attr| {
        attr.path.get_ident().map(|it| it.to_string()) == Some("string".into())
    }).is_some();

    let field_type = match &field.ty {
        syn::Type::Ptr(ptr_t) => {
            match &*ptr_t.elem {
                syn::Type::Path(path_t) => quote!(ffi_utils::RawPointerTo::< #path_t >),
                _ => panic!("Pointer type is not supported") // TODO : is this the correct behaviour ???? What if we have pointer of pointer ???
            }
        }
        syn::Type::Path(path_t) => generic_path_to_concrete_type_path(&path_t.path),
        _ => { panic!("Field type is not supported") }
    };

    (field_name, field_type, is_nullable_field, is_string_field)
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
