use std::ffi::{CStr, CString};

use ffi_convert::{
    AsRust, AsRustError, CDrop, CDropError, CReprOf, CReprOfError, RawBorrow, RawPointerConverter,
};

/// A utility type to represent arrays of string
/// # Example
///
/// ```
/// use ffi_convert::CReprOf;
/// use ffi_convert_extra_ctypes::CStringArray;
/// let pizza_names = vec!["Diavola".to_string(), "Margarita".to_string(), "Regina".to_string()];
/// let c_pizza_names = CStringArray::c_repr_of(pizza_names).expect("could not convert !");
///
/// ```
#[repr(C)]
#[derive(Debug, RawPointerConverter)]
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
            let y = Box::from_raw(std::ptr::slice_from_raw_parts_mut(
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
