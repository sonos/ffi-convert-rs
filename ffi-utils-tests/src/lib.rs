use failure::{bail, Fallible};
use ffi_utils::{AsRust, CReprOf};

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

#[cfg(test)]
mod tests {
    use ffi_utils::{AsRust, CArray, CReprOf};

    #[derive(Clone, Debug, PartialEq)]
    pub struct Pancake {
        pub start: f32,
        pub end: f32,
        pub dummy: Dummy,
        pub sauce: Option<Sauce>,
        pub toppings: Vec<Topping>,
    }

    #[repr(C)]
    #[derive(CReprOf, AsRust)]
    #[converted(Pancake)]
    pub struct CPancake {
        start: f32,
        end: f32,
        dummy: CDummy,
        #[nullable]
        sauce: *const CSauce,
        toppings: *const CArray<CTopping>
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct Sauce {
        pub volume: f32,
    }

    #[repr(C)]
    #[derive(CReprOf, AsRust)]
    #[converted(Sauce)]
    pub struct CSauce {
        volume: f32,
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct Topping {
        pub amount: i32,
    }

    #[repr(C)]
    #[derive(CReprOf, AsRust)]
    #[converted(Topping)]
    pub struct CTopping {
        amount: i32,
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct Dummy {
        pub count: i32,
    }

    #[repr(C)]
    #[derive(CReprOf, AsRust)]
    #[converted(Dummy)]
    pub struct CDummy {
        count: i32,
    }

    generate_round_trip_rust_c_rust!(round_trip_sauce, Sauce, CSauce, { Sauce { volume: 4.2 } });

    generate_round_trip_rust_c_rust!(round_trip_topping, Topping, CTopping, {
        Topping { amount: 2 }
    });

    generate_round_trip_rust_c_rust!(round_trip_dummy, Dummy, CDummy, { Dummy { count: 2 } });

    generate_round_trip_rust_c_rust!(round_trip_pancake, Pancake, CPancake, {
        Pancake {
            start: 0.0,
            end: 2.0,
            dummy: Dummy { count: 2 },
            sauce: Some(Sauce { volume: 4.2 }),
            toppings: vec![Topping { amount: 2 }, Topping { amount: 3 }],
        }
    });
}
