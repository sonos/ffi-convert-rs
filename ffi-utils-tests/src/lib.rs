#[cfg(test)]
mod tests {
    use ffi_utils::{AsRust, CArray, CReprOf};

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

    pub struct Sauce {
        pub volume: f32,
    }

    #[repr(C)]
    #[derive(CReprOf, AsRust)]
    #[converted(Sauce)]
    pub struct CSauce {
        volume: f32,
    }

    pub struct Topping {
        pub amount: i32,
    }

    #[repr(C)]
    #[derive(CReprOf, AsRust)]
    #[converted(Topping)]
    pub struct CTopping {
        amount: i32,
    }

    pub struct Dummy {
        pub count: i32,
    }

    #[repr(C)]
    #[derive(CReprOf, AsRust)]
    #[converted(Dummy)]
    pub struct CDummy {
        count: i32,
    }

    #[test]
    fn should_work() {
        let pancakes = Pancake {
            start: 0.0,
            end: 2.0,
            dummy: Dummy { count: 2 },
            sauce: None,
            toppings: vec![Topping { amount: 2 }, Topping { amount: 3 }],
        };

        let _c_pancakes = CPancake::c_repr_of(pancakes).unwrap();
    }
}
