use crate::utils::{parse_no_drop_impl_flag, parse_struct_fields, Field, TypeArrayOrTypePath};
use proc_macro::TokenStream;
use quote::quote;

pub fn impl_cdrop_macro(input: &syn::DeriveInput) -> TokenStream {
    let struct_name = &input.ident;
    let disable_drop_impl = parse_no_drop_impl_flag(&input.attrs);

    let fields = parse_struct_fields(&input.data);

    let do_drop_fields = fields
        .iter()
        .map(|field| {
            let Field {
                name: field_name,
                ref field_type,
                ..
            } = field;

            let drop_field = if field.is_string {
                quote!({
                    use ffi_convert::RawPointerConverter;
                    unsafe { std::ffi::CString::drop_raw_pointer(self.#field_name) }?
                })
            } else if field.is_pointer {
                match field_type {
                    TypeArrayOrTypePath::TypeArray(type_array) => {
                        quote!( unsafe { <#type_array>::drop_raw_pointer(self.#field_name) }? )
                    }
                    TypeArrayOrTypePath::TypePath(type_path) => {
                        quote!( unsafe { #type_path::drop_raw_pointer(self.#field_name) }? )
                    }
                }
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
        impl CDrop for # struct_name {
            fn do_drop(&mut self) -> Result<(), ffi_convert::CDropError> {
                use ffi_convert::RawPointerConverter;
                # ( #do_drop_fields; )*
                Ok(())
            }
        }
    );

    let drop_impl = quote!(
        impl Drop for # struct_name {
            fn drop(&mut self) {
                let _ = self.do_drop();
            }
        }
    );

    {
        if disable_drop_impl {
            quote! {
                # c_drop_impl
            }
        } else {
            quote! {
                # c_drop_impl
                # drop_impl
            }
        }
    }
    .into()
}
