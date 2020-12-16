use anyhow::{bail, Result};
use ffi_convert::*;
use std::ops::Range;

#[macro_export]
macro_rules! generate_round_trip_rust_c_rust {
    ($func_name:ident, $rust_struct:ty, $c_struct:ty, $builder:block) => {
        #[test]
        fn $func_name() {
            use $crate::round_trip_test_rust_c_rust;
            let item = $builder;
            round_trip_test_rust_c_rust::<$c_struct, $rust_struct>(item)
                .expect("Round trip test failed!");
        }
    };
}

pub fn round_trip_test_rust_c_rust<T, U>(value: U) -> Result<()>
where
    T: AsRust<U> + CReprOf<U>,
    U: Clone + PartialEq,
{
    let value2 = value.clone();
    let intermediate: T = T::c_repr_of(value2)?;
    let value_roundtrip: U = intermediate.as_rust()?;

    if value != value_roundtrip {
        bail!("The value is not the same before and after the roundtrip");
    }

    Ok(())
}

#[derive(Clone, Debug, PartialEq)]
pub struct Pancake {
    pub name: String,
    pub description: Option<String>,
    pub start: f32,
    pub end: Option<f32>,
    pub dummy: Dummy,
    pub sauce: Option<Sauce>,
    pub toppings: Vec<Topping>,
    pub layers: Option<Vec<Layer>>,
    pub is_delicious: bool,
    pub range: Range<usize>,
    pub some_futile_info: Option<String>,
    pub flattened_range: Range<i64>,
    pub field_with_specific_rust_name: String,
}

#[repr(C)]
#[derive(CReprOf, AsRust, CDrop, RawPointerConverter)]
#[target_type(Pancake)]
#[as_rust_extra_field(some_futile_info = None)]
#[as_rust_extra_field(flattened_range = self.flattened_range_start..self.flattened_range_end)]
pub struct CPancake {
    name: *const libc::c_char,
    #[nullable]
    description: *const libc::c_char,
    start: f32,
    #[nullable]
    end: *const f32,
    dummy: CDummy,
    #[nullable]
    sauce: *const CSauce,
    toppings: *const CArray<CTopping>,
    #[nullable]
    layers: *const CArray<CLayer>,
    is_delicious: u8,
    pub range: CRange<i32>,
    #[c_repr_of_convert(input.flattened_range.start)]
    flattened_range_start: i64,
    #[c_repr_of_convert(input.flattened_range.end)]
    flattened_range_end: i64,
    #[target_name(field_with_specific_rust_name)]
    pub field_with_specific_c_name: *const libc::c_char,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sauce {
    pub volume: f32,
}

#[repr(C)]
#[derive(CReprOf, AsRust, CDrop, RawPointerConverter)]
#[target_type(Sauce)]
pub struct CSauce {
    volume: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Topping {
    pub amount: i32,
}

#[repr(C)]
#[derive(CReprOf, AsRust, CDrop, RawPointerConverter)]
#[target_type(Topping)]
pub struct CTopping {
    amount: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Layer {
    pub number: i32,
    pub subtitle: Option<String>,
}

#[repr(C)]
#[derive(CReprOf, AsRust, CDrop, RawPointerConverter)]
#[target_type(Layer)]
pub struct CLayer {
    number: i32,
    #[nullable]
    subtitle: *const libc::c_char,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dummy {
    pub count: i32,
    pub describe: String,
}

#[repr(C)]
#[derive(CReprOf, AsRust, CDrop, RawPointerConverter)]
#[target_type(Dummy)]
pub struct CDummy {
    count: i32,
    describe: *const libc::c_char,
}

#[cfg(test)]
mod tests {
    use super::*;

    generate_round_trip_rust_c_rust!(round_trip_sauce, Sauce, CSauce, { Sauce { volume: 4.2 } });

    generate_round_trip_rust_c_rust!(round_trip_topping, Topping, CTopping, {
        Topping { amount: 2 }
    });

    generate_round_trip_rust_c_rust!(round_trip_dummy, Dummy, CDummy, {
        Dummy {
            count: 2,
            describe: "yo".to_string(),
        }
    });

    generate_round_trip_rust_c_rust!(round_trip_layer, Layer, CLayer, {
        Layer {
            number: 1,
            subtitle: Some(String::from("first layer")),
        }
    });

    generate_round_trip_rust_c_rust!(round_trip_pancake, Pancake, CPancake, {
        Pancake {
            name: String::from("Here is your pancake"),
            description: Some("I'm delicious ! ".to_string()),
            start: 0.0,
            end: Some(2.0),
            dummy: Dummy {
                count: 2,
                describe: "yo".to_string(),
            },
            sauce: Some(Sauce { volume: 32.23 }),
            toppings: vec![Topping { amount: 2 }, Topping { amount: 3 }],
            layers: Some(vec![Layer {
                number: 1,
                subtitle: Some(String::from("first layer")),
            }]),
            is_delicious: true,
            range: Range { start: 20, end: 30 },
            some_futile_info: None,
            flattened_range: Range { start: 42, end: 64 },
            field_with_specific_rust_name: "renamed field".to_string(),
        }
    });

    generate_round_trip_rust_c_rust!(round_trip_pancake_2, Pancake, CPancake, {
        Pancake {
            name: String::from("Here is your pancake"),
            description: Some("I'm delicious ! ".to_string()),
            start: 0.0,
            end: None,
            dummy: Dummy {
                count: 2,
                describe: "yo".to_string(),
            },
            sauce: None,
            toppings: vec![],
            layers: Some(vec![]),
            is_delicious: true,
            range: Range {
                start: 50,
                end: 100,
            },
            some_futile_info: None,
            flattened_range: Range { start: 42, end: 64 },
            field_with_specific_rust_name: "renamed field".to_string(),
        }
    });
}
