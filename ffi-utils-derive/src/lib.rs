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
        .map(|field| {
            let Field {
                name: field_name,
                ref field_type,
                ..
            } = field;

            let mut conversion = if field.is_string {
                quote!(std::ffi::CString::c_repr_of(field)?)
            } else {
                quote!(#field_type::c_repr_of(field)?)
            };

            if field.is_pointer {
                for _ in 0..field.levels_of_indirection {
                    conversion = quote!(#conversion.into_raw_pointer())
                }
            }

            conversion = if field.is_nullable {
                quote!(
                    #field_name: if let Some(field) = input.#field_name {
                        #conversion
                    } else {
                        std::ptr::null() as _
                    }
                )
            } else {
                quote!(#field_name: { let field = input.#field_name ; #conversion })
            };
            conversion
        })
        .collect::<Vec<_>>();

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
                # c_repr_of_impl
            }
        } else {
            quote! {
                # c_repr_of_impl
                # drop_impl
            }
        }
    }
    .into()
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
        .map(|field| {
            let Field {
                name: field_name,
                ref field_type,
                ..
            } = field;

            let mut conversion = if field.is_string {
                quote!( ffi_utils::create_rust_string_from!(self.#field_name) )
            } else {
                if field.is_pointer {
                    quote!( {
                            let ref_to_struct = unsafe { #field_type::raw_borrow(self.#field_name)? };
                            let converted_struct = ref_to_struct.as_rust()?;
                            converted_struct
                        }
                    )
                } else {
                    quote!(self.#field_name.as_rust()?)
                }
            };

            conversion = if field.is_nullable {
                quote!(
                    #field_name: if !self.#field_name.is_null() {
                        Some(#conversion)
                    } else {
                        None
                    }
                )
            } else {
                quote!(
                    #field_name: #conversion
                )
            };
            conversion
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
    )
    .into()
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

fn parse_struct_fields(data: &syn::Data) -> Vec<Field> {
    match &data {
        syn::Data::Struct(data_struct) => data_struct
            .fields
            .iter()
            .map(parse_field)
            .collect::<Vec<Field>>(),
        _ => panic!("CReprOf / AsRust can only be derived for structs"),
    }
}

struct Field<'a> {
    name: &'a syn::Ident,
    field_type: proc_macro2::TokenStream,
    is_nullable: bool,
    is_string: bool,
    is_pointer: bool,
    levels_of_indirection: u32,
}

fn parse_field(field: &syn::Field) -> Field {
    let field_name = field.ident.as_ref().expect("Field should have an ident");

    let mut inner_field_type: syn::Type = field.ty.clone();
    let mut levels_of_indirection: u32 = 0;

    while let syn::Type::Ptr(ptr_t) = inner_field_type {
        inner_field_type = *ptr_t.elem;
        levels_of_indirection += 1;
    }

    let field_type = match inner_field_type {
        syn::Type::Path(path_t) => generic_path_to_concrete_type_path(&path_t.path),
        _ => panic!("Field type used in this struct is not supported by the proc macro"),
    };

    let is_nullable_field = field
        .attrs
        .iter()
        .find(|attr| attr.path.get_ident().map(|it| it.to_string()) == Some("nullable".into()))
        .is_some();

    let is_string_field = match &field.ty {
        syn::Type::Ptr(ptr_t) => {
            match &*ptr_t.elem {
                syn::Type::Path(path_t) => {
                    // We are trying to detect the c_char identifier in the last segment
                    if let Some(segment) = path_t.path.segments.last() {
                        &segment.ident.to_string() == "c_char"
                    } else {
                        false
                    }
                }
                _ => false,
            }
        }
        _ => false,
    };

    let is_ptr_field = match &field.ty {
        syn::Type::Ptr(_) => true,
        _ => false,
    };

    Field {
        name: field_name,
        field_type,
        is_nullable: is_nullable_field,
        is_string: is_string_field,
        is_pointer: is_ptr_field,
        levels_of_indirection,
    }
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
