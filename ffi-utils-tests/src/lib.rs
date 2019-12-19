use failure::*;

use ffi_utils::*;

pub struct Pancake {
    pub name: String,
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
    name: *const libc::c_char,
    start: f32,
    end: f32,
    dummy: CDummy,
    #[nullable]
    sauce: *const CSauce,
    toppings: *const CArray<CTopping>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_work() {
        let input = Pancake {
            name: String::from("Here is your pancake"),
            start: 0.0,
            end: 2.0,
            dummy: Dummy { count: 2 },
            sauce: None,
            toppings: vec![Topping { amount: 2 }, Topping { amount: 3 }],
        };

        let output = CPancake::c_repr_of(input).unwrap().as_rust().unwrap();

        assert!(String::from("Here is your pancake").eq(&output.name));
        assert_eq!(output.start.clone(), 0.0);
        assert_eq!(output.end.clone(), 2.0);
        assert_eq!(output.dummy.count.clone(), 2);
        assert!(match output.sauce {
            Some(_) => false,
            None => true
        });
        assert_eq!(output.toppings[0].amount, 2);
        assert_eq!(output.toppings[1].amount, 3);
    }
}
