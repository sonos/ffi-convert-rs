use proc_macro::TokenStream;

use quote::{quote, quote_spanned};

use crate::utils::{
    Field, TypeArrayOrTypePath, parse_enum_variants, parse_no_drop_impl_flag, parse_struct_fields,
    parse_target_type,
};

pub fn impl_creprof_macro(input: &syn::DeriveInput) -> TokenStream {
    let name = &input.ident;
    let target_type = parse_target_type(&input.attrs);
    let disable_drop_impl = parse_no_drop_impl_flag(&input.attrs);

    match &input.data {
        syn::Data::Struct(data_struct) => {
            impl_creprof_struct(name, &target_type, disable_drop_impl, data_struct)
        }
        syn::Data::Enum(data_enum) => {
            impl_creprof_enum(name, &target_type, disable_drop_impl, data_enum)
        }
        _ => panic!("CReprOf can only be derived for structs and unit enums"),
    }
}

fn impl_creprof_struct(
    struct_name: &syn::Ident,
    target_type: &syn::Path,
    disable_drop_impl: bool,
    data: &syn::DataStruct,
) -> TokenStream {
    let fields = parse_struct_fields(data);
    let c_repr_of_fields = fields
        .iter()
        .map(|field| {
            let Field {
                name: field_name,
                target_name: target_field_name,
                field_type,
                ..
            } = field;
            let field_span = field_name.span();

            let mut conversion = if field.is_string {
                quote_spanned!(field_span => std::ffi::CString::c_repr_of(field)?)
            } else {
                match field_type {
                    TypeArrayOrTypePath::TypeArray(type_array) => {
                        quote_spanned!(field_span => <#type_array>::c_repr_of(field)?)
                    }
                    TypeArrayOrTypePath::TypePath(type_path) => {
                        quote_spanned!(field_span => #type_path::c_repr_of(field)?)
                    }
                }
            };

            if field.is_pointer {
                for _ in 0..field.levels_of_indirection {
                    conversion = quote_spanned!(field_span => #conversion.into_raw_pointer())
                }
            }

            conversion = if field.is_nullable {
                quote_spanned!(field_span =>
                    #field_name: if let Some(field) = input.#target_field_name {
                        #conversion
                    } else {
                        std::ptr::null() as _
                    }
                )
            } else {
                quote_spanned!(field_span => #field_name: { let field = input.#target_field_name ; #conversion })
            };
            if let Some(convert) = &field.c_repr_of_convert {
                quote_spanned!(field_span => #field_name: #convert)
            } else {
                conversion
            }
        })
        .collect::<Vec<_>>();

    let do_drop_fields = fields
        .iter()
        .map(|field| {
            let Field {
                name: field_name,
                field_type,
                ..
            } = field;
            let field_span = field_name.span();

            let drop_field = if field.is_string {
                quote_spanned!(field_span => {
                    unsafe { std::ffi::CString::drop_raw_pointer(self.#field_name) }?
                })
            } else if field.is_pointer {
                match field_type {
                    TypeArrayOrTypePath::TypeArray(type_array) => {
                        quote_spanned!(field_span => unsafe { <#type_array>::drop_raw_pointer(self.#field_name) }? )
                    }
                    TypeArrayOrTypePath::TypePath(type_path) => {
                        quote_spanned!(field_span => unsafe { #type_path::drop_raw_pointer(self.#field_name) }? )
                    }
                }
            } else {
                // the other cases will be handled automatically by rust
                quote!()
            };

            if field.is_nullable {
                quote_spanned!(field_span =>
                    if !self.#field_name.is_null() {
                       # drop_field
                    }
                )
            } else {
                drop_field
            }
        })
        .collect::<Vec<_>>();

    let creprof_impl = quote!(
        impl CReprOf<#target_type> for #struct_name {
            fn c_repr_of(input: #target_type) -> Result<Self, ffi_convert::CReprOfError> {
                use ffi_convert::RawPointerConverter;
                Ok(Self {
                    #(#c_repr_of_fields,)*
                })
            }

            fn do_drop(&mut self) -> Result<(), ffi_convert::CDropError> {
                use ffi_convert::RawPointerConverter;
                #(#do_drop_fields;)*
                Ok(())
            }
        }
    );

    let drop_impl = quote!(
        impl Drop for #struct_name {
            fn drop(&mut self) {
                let _ = self.do_drop();
            }
        }
    );

    if disable_drop_impl {
        quote!(#creprof_impl)
    } else {
        quote!(#creprof_impl #drop_impl)
    }
    .into()
}

fn impl_creprof_enum(
    enum_name: &syn::Ident,
    target_type: &syn::Path,
    disable_drop_impl: bool,
    data: &syn::DataEnum,
) -> TokenStream {
    let variants = parse_enum_variants(data);

    let match_arms = variants
        .iter()
        .map(|variant| quote!(#target_type::#variant => Ok(#enum_name::#variant)));

    let creprof_impl = quote!(
        impl CReprOf<#target_type> for #enum_name {
            fn c_repr_of(input: #target_type) -> Result<Self, ffi_convert::CReprOfError> {
                match input {
                    #(#match_arms,)*
                }
            }
        }
    );

    let drop_impl = quote!(
        impl Drop for #enum_name {
            fn drop(&mut self) {
                let _ = self.do_drop();
            }
        }
    );

    if disable_drop_impl {
        quote!(#creprof_impl)
    } else {
        quote!(#creprof_impl #drop_impl)
    }
    .into()
}
