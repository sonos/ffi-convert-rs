//! A collection of utilities (functions, traits, data structures, etc ...) to ease conversion between Rust and C-compatible data structures.
//!
//! Through two **conversion traits**, **`CReprOf`** and **`AsRust`**, this crate provides a framework to convert idiomatic Rust structs to C-compatible structs that can pass through an FFI boundary, and conversely.
//! They ensure that the developer uses best practices when performing the conversion in both directions (ownership-wise).
//!
//! The crate also provides a collection of useful utility functions to perform conversions of types.
//! It goes hand in hand with the `ffi-convert-derive` crate as it provides an **automatic derivation** of the `CReprOf` and `AsRust` trait.
//!
//! # Usage
//! When dealing with an FFI frontier, the general philosophy of the crate is :  
//! - When receiving pointers to structs created by C code, the struct is immediately converted to an owned, idiomatic Rust struct through the use of the `AsRust` trait.
//! - To send an idiomatic, owned Rust struct to C code, the struct is converted to C-compatible representation using the CReprOf trait.
//!
//! ## Example
//!
//! We want to be able to convert a **`Pizza`** Rust struct that has an idiomatic representation to a **`CPizza`** Rust struct that has a C-compatible representation in memory.
//! We start by defining the fields of the `Pizza` struct :
//! ```
//! # struct Topping {};
//! # struct Sauce {};
//! pub struct Pizza {
//!     pub name: String,
//!     pub toppings: Vec<Topping>,
//!     pub base: Option<Sauce>,
//!     pub weight: f32,
//! }
//!```
//!
//! We then create the C-compatible struct by [mapping](#types-representations-mapping) idiomatic Rust types to C-compatible types :
//! ```
//! # use ffi_convert::CArray;
//! # struct CTopping {};
//! # struct CSauce {};
//! #[repr(C)]
//! pub struct CPizza {
//!     pub name: *const libc::c_char,
//!     pub toppings: *const CArray<CTopping>,
//!     pub base: *const CSauce,
//!     pub weight: libc::c_float,
//! }
//! ```
//!
//! This crate provides two traits that are useful for converting between Pizza to CPizza and conversely.
//!
//! ```ignore
//!    CPizza::c_repr_of(pizza)
//!      <=================|
//!
//! CPizza                   Pizza
//!
//!      |=================>
//!       cpizza.as_rust()
//!
//! ```
//! Instead of manually writing the body of the conversion traits, we can derive them :
//!
//! ```
//! # use ffi_convert::{CReprOf, AsRust, CDrop};
//! # use ffi_convert::CArray;
//! # use ffi_convert::RawBorrow;
//! # struct Sauce {};
//! # #[derive(CReprOf, AsRust, CDrop)]
//! # #[target_type(Sauce)]
//! # struct CSauce {};
//! # struct Topping {};
//! # #[derive(CReprOf, AsRust, CDrop)]
//! # #[target_type(Topping)]
//! # struct CTopping {};
//! #
//! # struct Pizza {
//! #     name: String,
//! #     toppings: Vec<Topping>,
//! #     base: Sauce,
//! #     weight: f32
//! # };
//! use libc::{c_char, c_float};
//!
//! #[repr(C)]
//! #[derive(CReprOf, AsRust, CDrop)]
//! #[target_type(Pizza)]
//! pub struct CPizza {
//!     pub name: *const c_char,
//!     pub toppings: *const CArray<CTopping>,
//!     pub base: *const CSauce,
//!     pub weight: c_float,
//! }
//! ```
//!
//! You may have noticed that you have to derive the CDrop trait.
//! The CDrop trait needs to be implemented on every C-compatible struct that require manual resource management.
//! The release of those resources should be done in the drop method of the CDrop trait.
//!
//! You can now pass the `CPizza` struct through your FFI boundary !
//!

//! ## Types representations mapping
//!
//! <table>
//!     <thead>
//!         <tr>
//!             <th>C type</th>
//!             <th>Rust type</th>
//!             <th>C-compatible Rust type</th>
//!         </tr>
//!     </thead>
//!     <tbody>
//!         <tr>
//!             <td><code>const char*</code></td>
//!             <td><code>String</code></td>
//!             <td><code>*const libc::c_char</code></td>
//!         </tr>
//!         <tr>
//!             <td><code>const T</code></td>
//!             <td><code>Option<T></code></td>
//!             <td><code>*const T</code> (with <code>#[nullable]</code> field annotation)</td>
//!         </tr>
//!         <tr>
//!             <td><code>CArrayT</code></td>
//!             <td><code>Vec<T></code></td>
//!             <td><code>CArray<T></code></td>
//!         </tr>
//!     </tbody>
//! </table>
//!
//! ```ignore
//! typedef struct {
//! const T *values; // Pointer to the value of the list
//! int32_t size; // Number of T values in the list
//! } CArrayT;
//! ```

//! ## The CReprOf trait

//! The `CReprOf` trait allows to create a C-compatible representation of the reciprocal idiomatic Rust struct by consuming the latter.

//! ```
//! # use ffi_convert::{Error, CDrop};
//! pub trait CReprOf<T>: Sized + CDrop {
//!     fn c_repr_of(input: T) -> Result<Self, Error>;
//! }
//! ```

//! This shows that the struct implementing it is a `repr(C)` compatible view of the parametrized
//! type and can be created from an object of this type.

//! ## The AsRust trait

//! > When trying to convert a `repr(C)` struct that originated from C, the philosophy is to immediately convert
//! > the struct to an **owned** idiomatic representation of the struct via the AsRust trait.

//! The `AsRust` trait allows to create an idiomatic Rust struct from a C-compatible struct :

//! ```
//! # use ffi_convert::{Error, CDrop};
//! pub trait AsRust<T> {
//!     fn as_rust(&self) -> Result<T, Error>;
//! }
//! ```

//! This shows that the struct implementing it is a `repr(C)` compatible view of the parametrized
//! type and that an instance of the parametrized type can be created form this struct.

//! ## The CDrop trait

//! A Trait showing that the `repr(C)` compatible view implementing it can free up its part of memory that are not
//! managed by Rust drop mechanism.

//! ## Caveats with derivation of CReprOf and AsRust traits
//!
pub use ffi_convert_derive::*;

mod conversions;
mod types;

pub use conversions::*;
pub use types::*;

pub use failure::Error;
