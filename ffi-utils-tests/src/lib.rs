use failure::{bail, Fallible};
use ffi_utils::*;

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

pub fn round_trip_test_rust_c_rust<T, U>(value: U) -> Fallible<()>
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
}

#[repr(C)]
#[derive(CReprOf, AsRust)]
#[target_type(Pancake)]
pub struct CPancake {
    #[string]
    name: *const libc::c_char,
    #[nullable]
    #[string]
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
}


#[derive(Clone, Debug, PartialEq)]
pub struct Sauce {
    pub volume: f32,
}

#[repr(C)]
#[derive(CReprOf, AsRust)]
#[target_type(Sauce)]
pub struct CSauce {
    volume: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Topping {
    pub amount: i32,
}

#[repr(C)]
#[derive(CReprOf, AsRust)]
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
#[derive(CReprOf, AsRust)]
#[target_type(Layer)]
pub struct CLayer {
    number: i32,
    #[nullable]
    subtitle: *const libc::c_char,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dummy {
    pub count: i32,
}

#[repr(C)]
#[derive(CReprOf, AsRust)]
#[target_type(Dummy)]
pub struct CDummy {
    count: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    generate_round_trip_rust_c_rust!(round_trip_sauce, Sauce, CSauce, { Sauce { volume: 4.2 } });

    generate_round_trip_rust_c_rust!(round_trip_topping, Topping, CTopping, {
        Topping { amount: 2 }
    });

    generate_round_trip_rust_c_rust!(round_trip_dummy, Dummy, CDummy, { Dummy { count: 2 } });

    generate_round_trip_rust_c_rust!(round_trip_layer, Layer, CLayer, {
        Layer {
            number: 1,
            subtitle: Some(String::from("first layer"))
        }
    });

    generate_round_trip_rust_c_rust!(round_trip_pancake, Pancake, CPancake, {
        Pancake {
            name: String::from("Here is your pancake"),
            description: Some("I'm delicious ! ".to_string()),
            start: 0.0,
            end: Some(2.0),
            dummy: Dummy { count: 2 },
            sauce: Some(Sauce { volume: 32.23}),
            toppings: vec![Topping { amount: 2 }, Topping { amount: 3 }],
            layers: Some(vec![Layer { number: 1, subtitle: Some(String::from("first layer"))}]),
            is_delicious: true
        }
    });

    generate_round_trip_rust_c_rust!(round_trip_pancake_2, Pancake, CPancake, {
        Pancake {
            name: String::from("Here is your pancake"),
            description: Some("I'm delicious ! ".to_string()),
            start: 0.0,
            end: None,
            dummy: Dummy { count: 2 },
            sauce: None,
            toppings: vec![],
            layers: Some(vec![]),
            is_delicious: true
        }
    });
}
