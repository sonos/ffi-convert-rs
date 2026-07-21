use crate::utils::{Field, TypeArrayOrTypePath, parse_no_drop_impl_flag, parse_struct_fields};
use proc_macro::TokenStream;
use quote::quote;

pub fn impl_cdrop_macro(input: &syn::DeriveInput) -> TokenStream {
    let name = &input.ident;
    let disable_drop_impl = parse_no_drop_impl_flag(&input.attrs);

    match &input.data {
        syn::Data::Struct(data_struct) => impl_cdrop_struct(name, disable_drop_impl, data_struct),
        syn::Data::Enum(_) => impl_cdrop_enum(name, disable_drop_impl),
        _ => panic!("CDrop can only be derived for structs and unit enums"),
    }
}

fn impl_cdrop_struct(
    struct_name: &syn::Ident,
    disable_drop_impl: bool,
    data: &syn::DataStruct,
) -> TokenStream {
    let fields = parse_struct_fields(data);

    let do_drop_fields = fields
        .iter()
        .map(|field| {
            let Field {
                name: field_name,
                field_type,
                ..
            } = field;

            // The pointer field is nulled before being freed so that a second
            // `do_drop` (e.g. from the generated `Drop` impl after a manual
            // `do_drop`) cannot free the same allocation twice.
            let drop_field = if field.is_string {
                quote!({
                    use ffi_convert::RawPointerConverter;
                    let field_ptr = self.#field_name;
                    self.#field_name = std::ptr::null::<u8>() as _;
                    unsafe { std::ffi::CString::drop_raw_pointer(field_ptr) }?
                })
            } else if field.is_pointer {
                let drop_call = match field_type {
                    TypeArrayOrTypePath::TypeArray(type_array) => {
                        quote!( unsafe { <#type_array>::drop_raw_pointer(field_ptr) }? )
                    }
                    TypeArrayOrTypePath::TypePath(type_path) => {
                        quote!( unsafe { #type_path::drop_raw_pointer(field_ptr) }? )
                    }
                };
                quote!({
                    let field_ptr = self.#field_name;
                    self.#field_name = std::ptr::null::<u8>() as _;
                    #drop_call
                })
            } else {
                // the other cases will be handled automatically by rust
                quote!()
            };

            if field.is_nullable {
                quote!(
                    if !self.#field_name.is_null() {
                       # drop_field
                    }
                )
            } else {
                drop_field
            }
        })
        .collect::<Vec<_>>();

    let c_drop_impl = quote!(
        impl CDrop for #struct_name {
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
        quote!(#c_drop_impl)
    } else {
        quote!(#c_drop_impl #drop_impl)
    }
    .into()
}

fn impl_cdrop_enum(enum_name: &syn::Ident, disable_drop_impl: bool) -> TokenStream {
    let c_drop_impl = quote!(
        impl CDrop for #enum_name {
            fn do_drop(&mut self) -> Result<(), ffi_convert::CDropError> {
                Ok(())
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
        quote!(#c_drop_impl)
    } else {
        quote!(#c_drop_impl #drop_impl)
    }
    .into()
}
