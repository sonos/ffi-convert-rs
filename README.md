# snips-utils-rs

This crate provides a framework to convert idiomatic Rust structs to C-like structs that can pass through an FFI boundary, and conversely.

This framework is made up of two **conversion traits**, **`CReprOf`** and **`AsRust`**.
They ensure that the developper uses best practices when performing the conversion in both directions (ownership-wise).

The crate also provides a collection of useful utility functions to perform conversions of types.
It goes hand in hand with the `ffi-utils-derive` as it provides an **automatic derivation** of the `CReprOf` and `AsRust` trait.

## Usage

We want to be able to convert a **`Pizza`** Rust struct that has an idiomatic representation to a **`CPizza`** Rust struct that
has a C-compatible representation in memory.

We start by definining the fields of the Pizza struct :

```
pub struct Pizza {
    pub name: String,
    pub toppings: Vec<Topping>,
    pub base: Option<Sauce>,
    pub weight: f32,
}
```

We then create the C-like struct by mapping idiomatic Rust types to C compatible types :

```
#[repr(C)]
pub struct CPizza {
    pub name: *const libc::c_char,
    pub toppings: *const CArray<CTopping>,
    pub base: *const CSauce,
    pub weight: libc::c_float,
}
```

This crate provides two traits that are useful for converting between Pizza to CPizza and conversely.

```
    CPizza::c_repr_of(pizza)
      <=================|

CPizza                   Pizza

      |=================>
       cpizza.as_rust()

```

Instead of manually writing the body of the conversion traits, we can derive them :

```
#[repr(C)]
#[derive(CReprOf, AsRust, CDrop)]
#[target_type(Pancake)]
pub struct CPizza {
    pub name: *const libc::c_char,
    pub toppings: *const CArray<CTopping>,
    pub base: *const CSauce,
    pub weight: libc::c_float,
}
```

You can now pass the `CPizza` struct through your FFI boundary !

## Example
TODO : Provide an example that you can copy/paste

## Types representations mapping

See definitions below this table.

|    C type   | Rust type |            C-compatible Rust type            |
|:-----------:|:---------:|:--------------------------------------------:|
| const char* |   String  |              *const libc::c_char             |
|   const T*  | Option<T> | *const T (with #[nullable] field annotation) |
|   CArrayT   |   Vec<T>  |                   CArray<T>                  |


```
typedef struct {
  const CSlotValue *slot_values; // Pointer to the first slot value of the list
  int32_t size; // Number of T values in the list
} CArrayT;

```

## The CReprOf trait

The `CReprOf` trait allows to create a C-compatible representation of the reciprocal idiomatic Rust struct by consuming the latter.

```
pub trait CReprOf<T>: Sized + CDrop {
    fn c_repr_of(input: T) -> Result<Self, Error>;
}
```

This shows that the struct implementing it is a `repr(C)` compatible view of the parametrized
type and can be created from an object of this type.

## The AsRust trait

> When trying to convert a `repr(C)` struct that originated from C, the philosophy is to immediately convert 
> the struct to an **owned** idiomatic representation of the struct via the AsRust trait. 

The `AsRust` trait allows to create an idiomatic Rust struct from a C-compatible struct :

```
pub trait AsRust<T> {
    fn as_rust(&self) -> Result<T, Error>;
}
```

This shows that the struct implementing it is a `repr(C)` compatible view of the parametrized
type and that an instance of the parametrized type can be created form this struct.


## The CDrop trait

A Trait showing that the `repr(C)` compatible view implementing it can free up its part of memory that are not
managed by Rust.


## Caveats with derivation of CReprOf and AsRust traits
TBD

## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
