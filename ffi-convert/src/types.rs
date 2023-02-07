//! This module contains definitions of utility types that implement the [`CReprOf`], [`AsRust`], and [`CDrop`] traits.
//!

use std::ffi::{CStr, CString};
use std::ops::Range;
use std::ptr::null;

use crate::conversions::*;

/// A utility type to represent arrays of string
/// # Example
///
/// ```
/// use ffi_convert::{CReprOf, CStringArray};
/// let pizza_names = vec!["Diavola".to_string(), "Margarita".to_string(), "Regina".to_string()];
/// let c_pizza_names = CStringArray::c_repr_of(pizza_names).expect("could not convert !");
///
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct CStringArray {
    /// Pointer to the first element of the array
    pub data: *const *const libc::c_char,
    /// Number of elements in the array
    pub size: usize,
}

unsafe impl Sync for CStringArray {}

impl AsRust<Vec<String>> for CStringArray {
    fn as_rust(&self) -> Result<Vec<String>, AsRustError> {
        let mut result = vec![];

        let strings = unsafe {
            std::slice::from_raw_parts_mut(self.data as *mut *mut libc::c_char, self.size)
        };

        for s in strings {
            result.push(unsafe { CStr::raw_borrow(*s) }?.as_rust()?)
        }

        Ok(result)
    }
}

impl CReprOf<Vec<String>> for CStringArray {
    fn c_repr_of(input: Vec<String>) -> Result<Self, CReprOfError> {
        Ok(Self {
            size: input.len(),
            data: Box::into_raw(
                input
                    .into_iter()
                    .map::<Result<*const libc::c_char, CReprOfError>, _>(|s| {
                        Ok(CString::c_repr_of(s)?.into_raw_pointer())
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .into_boxed_slice(),
            ) as *const *const libc::c_char,
        })
    }
}

impl CDrop for CStringArray {
    fn do_drop(&mut self) -> Result<(), CDropError> {
        unsafe {
            let y = Box::from_raw(std::slice::from_raw_parts_mut(
                self.data as *mut *mut libc::c_char,
                self.size,
            ));
            for p in y.iter() {
                let _ = CString::from_raw_pointer(*p)?; // let's not panic if we fail here
            }
        }
        Ok(())
    }
}

impl Drop for CStringArray {
    fn drop(&mut self) {
        let _ = self.do_drop();
    }
}

/// A utility type to represent arrays of the parametrized type.
/// Note that the parametrized type should have a C-compatible representation.
///
/// # Example
///
/// ```
/// use ffi_convert::{CReprOf, AsRust, CDrop, CArray};
/// use libc::c_char;
///
/// pub struct PizzaTopping {
///     pub ingredient: String,
/// }
///
/// #[derive(CDrop, CReprOf, AsRust)]
/// #[target_type(PizzaTopping)]
/// pub struct CPizzaTopping {
///     pub ingredient: *const c_char
/// }
///
/// let toppings = vec![
///         PizzaTopping { ingredient: "Cheese".to_string() },
///         PizzaTopping { ingredient: "Ham".to_string() } ];
///
/// let ctoppings = CArray::<CPizzaTopping>::c_repr_of(toppings);
///
/// ```
#[repr(C)]
#[derive(Debug, Clone)]
pub struct CArray<T> {
    data_ptr: *const T,
    size: usize,
}

impl<U: AsRust<V>, V> AsRust<Vec<V>> for CArray<U> {
    fn as_rust(&self) -> Result<Vec<V>, AsRustError> {
        let mut vec = Vec::with_capacity(self.size);
        if self.size > 0 {
            let values =
                unsafe { std::slice::from_raw_parts_mut(self.data_ptr as *mut U, self.size) };
            for value in values {
                vec.push(value.as_rust()?);
            }
        }
        Ok(vec)
    }
}

impl<U: CReprOf<V> + CDrop, V> CReprOf<Vec<V>> for CArray<U> {
    fn c_repr_of(input: Vec<V>) -> Result<Self, CReprOfError> {
        let input_size = input.len();
        Ok(Self {
            data_ptr: if input_size > 0 {
                Box::into_raw(
                    input
                        .into_iter()
                        .map(U::c_repr_of)
                        .collect::<Result<Vec<_>, CReprOfError>>()
                        .expect("Could not convert to C representation")
                        .into_boxed_slice(),
                ) as *const U
            } else {
                null() as *const U
            },
            size: input_size,
        })
    }
}

impl<T> CDrop for CArray<T> {
    fn do_drop(&mut self) -> Result<(), CDropError> {
        let _ = unsafe {
            Box::from_raw(std::slice::from_raw_parts_mut(
                self.data_ptr as *mut T,
                self.size,
            ))
        };
        Ok(())
    }
}

impl<T> Drop for CArray<T> {
    fn drop(&mut self) {
        let _ = self.do_drop();
    }
}

impl<T> RawPointerConverter<CArray<T>> for CArray<T> {
    fn into_raw_pointer(self) -> *const CArray<T> {
        convert_into_raw_pointer(self)
    }

    fn into_raw_pointer_mut(self) -> *mut CArray<T> {
        convert_into_raw_pointer_mut(self)
    }

    unsafe fn from_raw_pointer(
        input: *const CArray<T>,
    ) -> Result<Self, UnexpectedNullPointerError> {
        take_back_from_raw_pointer(input)
    }

    unsafe fn from_raw_pointer_mut(
        input: *mut CArray<T>,
    ) -> Result<Self, UnexpectedNullPointerError> {
        take_back_from_raw_pointer_mut(input)
    }
}

/// A utility type to represent range.
/// Note that the parametrized type T should have have `CReprOf` and `AsRust` trait implementated.
///
/// # Example
///
/// ```
/// use ffi_convert::{CReprOf, AsRust, CDrop, CRange};
/// use std::ops::Range;
///
/// #[derive(Clone, Debug, PartialEq)]
/// pub struct Foo {
///     pub range: Range<i32>
/// }
///
/// #[derive(AsRust, CDrop, CReprOf, Debug, PartialEq)]
/// #[target_type(Foo)]
/// pub struct CFoo {
///     pub range: CRange<i32>
/// }
///
/// let foo = Foo {
///     range: Range {
///         start: 20,
///         end: 30,
///     }
/// };
///
/// let c_foo = CFoo {
///     range: CRange {
///         start: 20,
///         end: 30,
///     }
/// };
///
/// let c_foo_converted = CFoo::c_repr_of(foo.clone()).unwrap();
/// assert_eq!(c_foo, c_foo_converted);
///
/// let foo_converted = c_foo.as_rust().unwrap();
/// assert_eq!(foo_converted, foo);
/// ```
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CRange<T> {
    pub start: T,
    pub end: T,
}

impl<U: AsRust<V>, V: PartialOrd + PartialEq> AsRust<Range<V>> for CRange<U> {
    fn as_rust(&self) -> Result<Range<V>, AsRustError> {
        Ok(Range {
            start: self.start.as_rust()?,
            end: self.end.as_rust()?,
        })
    }
}

impl<U: CReprOf<V> + CDrop, V: PartialOrd + PartialEq> CReprOf<Range<V>> for CRange<U> {
    fn c_repr_of(input: Range<V>) -> Result<Self, CReprOfError> {
        Ok(Self {
            start: U::c_repr_of(input.start)?,
            end: U::c_repr_of(input.end)?,
        })
    }
}

impl<T> CDrop for CRange<T> {
    fn do_drop(&mut self) -> Result<(), CDropError> {
        Ok(())
    }
}

impl<T> Drop for CRange<T> {
    fn drop(&mut self) {
        let _ = self.do_drop();
    }
}
