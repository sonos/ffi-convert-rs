use std::any::TypeId;
use std::ptr;

use ffi_convert::{
    AsRust, AsRustError, CDrop, CDropError, CReprOf, CReprOfError, RawPointerConverter,
    UnexpectedNullPointerError, convert_into_raw_pointer, convert_into_raw_pointer_mut,
    take_back_from_raw_pointer, take_back_from_raw_pointer_mut,
};

/// A utility type to represent arrays of the parametrized type.
/// Note that the parametrized type should have a C-compatible representation.
///
/// # Example
///
/// ```
/// use ffi_convert::{CReprOf, AsRust, CDrop};
/// use ffi_convert_extra_ctypes::CArray;
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
#[derive(Debug)]
pub struct CArray<T> {
    /// Pointer to the first element of the array
    pub data_ptr: *const T,
    /// Number of elements in the array
    pub size: usize,
}

impl<U: AsRust<V> + 'static, V> AsRust<Vec<V>> for CArray<U> {
    fn as_rust(&self) -> Result<Vec<V>, AsRustError> {
        let mut vec = Vec::with_capacity(self.size);

        if self.size > 0 {
            let values =
                unsafe { std::slice::from_raw_parts_mut(self.data_ptr as *mut U, self.size) };

            if is_primitive(TypeId::of::<U>()) {
                unsafe {
                    ptr::copy(values.as_ptr() as *const V, vec.as_mut_ptr(), self.size);
                    vec.set_len(self.size);
                }
            } else {
                for value in values {
                    vec.push(value.as_rust()?);
                }
            }
        }
        Ok(vec)
    }
}

impl<U: CReprOf<V> + CDrop, V: 'static> CReprOf<Vec<V>> for CArray<U> {
    fn c_repr_of(input: Vec<V>) -> Result<Self, CReprOfError> {
        let input_size = input.len();
        let mut output: CArray<U> = CArray {
            data_ptr: ptr::null(),
            size: input_size,
        };

        if input_size > 0 {
            if is_primitive(TypeId::of::<V>()) {
                output.data_ptr = Box::into_raw(input.into_boxed_slice()) as *const U;
            } else {
                output.data_ptr = Box::into_raw(
                    input
                        .into_iter()
                        .map(U::c_repr_of)
                        .collect::<Result<Vec<_>, CReprOfError>>()
                        .expect("Could not convert to C representation")
                        .into_boxed_slice(),
                ) as *const U;
            }
        } else {
            output.data_ptr = ptr::null();
        }
        Ok(output)
    }
}

impl<T> CDrop for CArray<T> {
    fn do_drop(&mut self) -> Result<(), CDropError> {
        if !self.data_ptr.is_null() {
            let _ = unsafe {
                Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                    self.data_ptr as *mut T,
                    self.size,
                ))
            };
        }
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
        unsafe { take_back_from_raw_pointer(input) }
    }

    unsafe fn from_raw_pointer_mut(
        input: *mut CArray<T>,
    ) -> Result<Self, UnexpectedNullPointerError> {
        unsafe { take_back_from_raw_pointer_mut(input) }
    }
}

fn is_primitive(id: TypeId) -> bool {
    id == TypeId::of::<u8>()
        || id == TypeId::of::<i8>()
        || id == TypeId::of::<u16>()
        || id == TypeId::of::<i16>()
        || id == TypeId::of::<u32>()
        || id == TypeId::of::<i32>()
        || id == TypeId::of::<f32>()
        || id == TypeId::of::<f64>()
}
