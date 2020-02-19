use crate::utils::{parse_no_drop_impl_flag, parse_struct_fields, Field};
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
                quote!(ffi_utils::take_back_c_string!(self.#field_name))
            } else {
                if field.is_pointer {
                    quote!( unsafe { #field_type::drop_raw_pointer(self.#field_name) }? )
                } else {
                    quote!( self.# field_name.do_drop()? )
                }
            };

            let conversion = if field.is_nullable {
                quote!(
                    if !self.#field_name.is_null() {
                       # drop_field
                    }
                )
            } else {
                drop_field
            };
            conversion
        })
        .collect::<Vec<_>>();

    let c_drop_impl = quote!(
        impl CDrop for # struct_name {
            fn do_drop(&mut self) -> Result<(), ffi_utils::Error> {
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
