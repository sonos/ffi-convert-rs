use failure::*;

use ffi_utils::*;

pub struct Pancake {
    pub name: String,
    pub description: Option<String>,
    pub start: f32,
    pub end: Option<f32>,
    pub dummy: Dummy,
    pub sauce: Option<Sauce>,
    pub toppings: Vec<Topping>,
    pub layers: Option<Vec<Layer>>
}

#[repr(C)]
#[derive(CReprOf, AsRust)]
#[converted(Pancake)]
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
    layers: *const CArray<CLayer>
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

pub struct Layer {
    pub number: i32,
    pub subtitle: Option<String>
}

#[repr(C)]
#[derive(CReprOf, AsRust)]
#[converted(Layer)]
pub struct CLayer {
    number: i32,
    #[nullable]
    subtitle: *const libc::c_char,
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
            description: None,
            start: 0.0,
            end: Some(2.0),
            dummy: Dummy { count: 2 },
            sauce: None,
            toppings: vec![Topping { amount: 2 }, Topping { amount: 3 }],
            layers: Some(vec![Layer { number: 1, subtitle: Some(String::from("first layer"))}]),
        };

        let output = CPancake::c_repr_of(input).unwrap().as_rust().unwrap();

        assert!(String::from("Here is your pancake").eq(&output.name));
        assert!(match output.description {
            Some(_) => false,
            None => true
        });
        assert_eq!(output.start.clone(), 0.0);
        assert!(match output.end {
            Some(value) => value == 2.0,
            None => false
        });
        assert_eq!(output.dummy.count.clone(), 2);
        assert!(match output.sauce {
            Some(_) => false,
            None => true
        });
        assert_eq!(output.toppings[0].amount, 2);
        assert_eq!(output.toppings[1].amount, 3);
        assert!(match output.layers {
            Some(layer) => {
                (if let Some(s) = &layer[0].subtitle {
                    String::from("first layer").eq(s)
                } else {
                    false
                }) && (layer[0].number == 1)
            },
            None => false
        });
    }
}
