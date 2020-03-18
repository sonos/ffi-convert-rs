use quote::quote;

pub fn parse_target_type(attrs: &Vec<syn::Attribute>) -> syn::Path {
    let target_type_attribute = attrs
        .iter()
        .find(|attribute| {
            attribute.path.get_ident().map(|it| it.to_string()) == Some("target_type".into())
        })
        .expect("Can't derive CReprOf without target_type helper attribute.");

    target_type_attribute.parse_args().unwrap()
}

pub fn parse_no_drop_impl_flag(attrs: &Vec<syn::Attribute>) -> bool {
    attrs
        .iter()
        .find(|attribute| {
            attribute.path.get_ident().map(|it| it.to_string()) == Some("no_drop_impl".to_string())
        })
        .is_some()
}

pub fn parse_struct_fields(data: &syn::Data) -> Vec<Field> {
    match &data {
        syn::Data::Struct(data_struct) => data_struct
            .fields
            .iter()
            .map(parse_field)
            .collect::<Vec<Field>>(),
        _ => panic!("CReprOf / AsRust can only be derived for structs"),
    }
}

pub struct Field<'a> {
    pub name: &'a syn::Ident,
    pub field_type: proc_macro2::TokenStream,
    pub is_nullable: bool,
    pub is_string: bool,
    pub is_pointer: bool,
    pub levels_of_indirection: u32,
}

pub fn parse_field(field: &syn::Field) -> Field {
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

pub fn generic_path_to_concrete_type_path(path: &syn::Path) -> proc_macro2::TokenStream {
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
        quote!(#segments#turbofished_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::Path;

    #[test]
    fn test_generic_path_to_concrete_type_path() {
        let original_path = syn::parse_str::<Path>("std::module1::module2::Hello")
            .expect("Could not parse str into syn::Path");
        let transformed_path =
            syn::parse2::<Path>(generic_path_to_concrete_type_path(&original_path))
                .expect("could not parse tok stream into syn::Path");
        assert_eq!(transformed_path, original_path);
    }

    #[test]
    fn test_generic_path_to_concrete_type_path_with_type_param() {
        // This tests checks that the following transformation happens :
        //                                   generic_path_to_concrete_type_path
        // "std::module1::module2::Vec<Hello>" ----------------------------> "std::module1::module2::Vec::<Hello>"

        assert_eq!(
            syn::parse_str::<Path>("std::module1::module2::Vec::<Hello>")
                .expect("could not parse str into syn::Path"),
            syn::parse2::<Path>(generic_path_to_concrete_type_path(
                &syn::parse_str::<Path>("std::module1::module2::Vec<Hello>")
                    .expect("could not parse str into syn::Path")
            ))
            .expect("could not parse token stream into syn::Path")
        )
    }

    #[test]
    fn test_generic_path_to_concrete_type_path_without_segments() {
        let original_path =
            syn::parse_str::<Path>("Hello").expect("Could not parse str into syn::Path");
        let transformed_path =
            syn::parse2::<Path>(generic_path_to_concrete_type_path(&original_path))
                .expect("could not parse tok stream into syn::Path");
        assert_eq!(transformed_path, original_path);
    }
}
