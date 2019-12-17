use failure::{ensure, format_err, Error, ResultExt};

use std::ptr::null;

#[macro_export]
macro_rules! convert_to_c_string {
    ($string:expr) => {
        $crate::convert_to_c_string_result!($string)?
    };
}

#[macro_export]
macro_rules! convert_to_c_string_result {
    ($string:expr) => {
        std::ffi::CString::c_repr_of($string).map(|s| s.into_raw_pointer() as *const libc::c_char)
    };
}

#[macro_export]
macro_rules! convert_to_c_string_array {
    ($string_vec:expr) => {{
        $crate::CStringArray::c_repr_of($string_vec)?.into_raw_pointer()
    }};
}

#[macro_export]
macro_rules! convert_to_nullable_c_string_array {
    ($opt:expr) => {
        if let Some(s) = $opt {
            $crate::convert_to_c_string_array!(s)
        } else {
            null()
        }
    };
}

#[macro_export]
macro_rules! convert_to_nullable_c_string {
    ($opt:expr) => {
        if let Some(s) = $opt {
            $crate::convert_to_c_string!(s)
        } else {
            null()
        }
    };
}

#[macro_export]
macro_rules! take_back_c_string {
    ($pointer:expr) => {{
        let _ = unsafe { std::ffi::CString::from_raw_pointer($pointer) };
    }};
}

#[macro_export]
macro_rules! take_back_nullable_c_string {
    ($pointer:expr) => {
        if !$pointer.is_null() {
            $crate::take_back_c_string!($pointer)
        }
    };
}

#[macro_export]
macro_rules! take_back_c_string_array {
    ($pointer:expr) => {{
        let _ = unsafe { $crate::CStringArray::from_raw_pointer($pointer) };
    }};
}

#[macro_export]
macro_rules! take_back_nullable_c_string_array {
    ($pointer:expr) => {
        if !$pointer.is_null() {
            $crate::take_back_c_string_array!($pointer)
        }
    };
}

#[macro_export]
macro_rules! create_rust_string_from {
    ($pointer:expr) => {{
        use $crate::RawBorrow;
        unsafe { std::ffi::CStr::raw_borrow($pointer) }?
            .to_str()
            .context("Could not convert pointer to rust str")?
            .to_owned()
    }};
}

#[macro_export]
macro_rules! create_optional_rust_string_from {
    ($pointer:expr) => {
        match unsafe { $pointer.as_ref() } {
            Some(thing) => Some($crate::create_rust_string_from!(thing)),
            None => None,
        }
    };
}

#[macro_export]
macro_rules! create_rust_vec_string_from {
    ($pointer:expr) => {
        unsafe { $crate::CStringArray::raw_borrow($pointer) }?.as_rust()?
    };
}

#[macro_export]
macro_rules! create_optional_rust_vec_string_from {
    ($pointer:expr) => {
        match unsafe { $pointer.as_ref() } {
            Some(thing) => Some($crate::create_rust_vec_string_from!(thing)),
            None => None,
        }
    };
}

pub fn point_to_string(pointer: *mut *const libc::c_char, string: String) -> Result<(), Error> {
    unsafe { *pointer = std::ffi::CString::c_repr_of(string)?.into_raw_pointer() }
    Ok(())
}

/// Trait showing that the struct implementing it is a `repr(C)` compatible view of the parametrized
/// type that can be created from an object of this type.
pub trait CReprOf<T>: Sized {
    fn c_repr_of(input: T) -> Result<Self, Error>;
}

/// Trait showing that the struct implementing it is a `repr(C)` compatible view of the parametrized
/// type and that an instance of the parametrized type can be created form this struct
pub trait AsRust<T> {
    fn as_rust(&self) -> Result<T, Error>;
}

/// Trait representing the creation of a raw pointer from a struct and the recovery of said pointer.
///
/// The `from_raw_pointer` function should be used only on pointers obtained thought the
/// `into_raw_pointer` method (and is thus unsafe as we don't have any way to get insurance of that
/// from the compiler).
///
/// The `from_raw_pointer` effectively takes back ownership of the pointer. If you didn't create the
/// pointer yourself, please use the `as_ref` method on the raw pointer to borrow it
///
/// A generic implementation of this trait exist for every struct, it will use a `Box` to create the
/// pointer. There is also a special implementation available in order to create a
/// `*const libc::c_char` from a CString.
pub trait RawPointerConverter<T>: Sized {
    fn into_raw_pointer(self) -> *const T;
    unsafe fn from_raw_pointer(input: *const T) -> Result<Self, Error>;

    unsafe fn drop_raw_pointer(input: *const T) -> Result<(), Error> {
        T::from_raw_pointer(input).map(|_| ())
    }
}

pub trait RawBorrow<T> {
    unsafe fn raw_borrow<'a>(input: *const T) -> Result<&'a Self, Error>;
}

pub trait RawBorrowMut<T> {
    unsafe fn raw_borrow_mut<'a>(input: *mut T) -> Result<&'a mut Self, Error>;
}

/// TODO custom derive instead of generic impl, this would prevent CString from having 2 impls...
impl<T> RawPointerConverter<T> for T {
    fn into_raw_pointer(self) -> *const T {
        Box::into_raw(Box::new(self)) as _
    }

    unsafe fn from_raw_pointer(input: *const T) -> Result<T, Error> {
        ensure!(
            !input.is_null(),
            "could not take raw pointer, unexpected null pointer"
        );
        Ok(*Box::from_raw(input as *mut T))
    }
}

impl<T> RawBorrow<T> for T {
    unsafe fn raw_borrow<'a>(input: *const T) -> Result<&'a Self, Error> {
        input
            .as_ref()
            .ok_or_else(|| format_err!("could not borrow, unexpected null pointer"))
    }
}

impl<T> RawBorrowMut<T> for T {
    unsafe fn raw_borrow_mut<'a>(input: *mut T) -> Result<&'a mut Self, Error> {
        input
            .as_mut()
            .ok_or_else(|| format_err!("could not borrow, unexpected null pointer"))
    }
}

impl RawPointerConverter<libc::c_void> for std::ffi::CString {
    fn into_raw_pointer(self) -> *const libc::c_void {
        self.into_raw() as _
    }

    unsafe fn from_raw_pointer(input: *const libc::c_void) -> Result<Self, Error> {
        ensure!(
            !input.is_null(),
            "could not take raw pointer, unexpected null pointer"
        );
        Ok(std::ffi::CString::from_raw(input as *mut libc::c_char))
    }
}

impl RawPointerConverter<libc::c_char> for std::ffi::CString {
    fn into_raw_pointer(self) -> *const libc::c_char {
        self.into_raw() as _
    }

    unsafe fn from_raw_pointer(input: *const libc::c_char) -> Result<Self, Error> {
        ensure!(
            !input.is_null(),
            "could not take raw pointer, unexpected null pointer"
        );
        Ok(std::ffi::CString::from_raw(input as *mut libc::c_char))
    }
}

impl RawBorrow<libc::c_char> for std::ffi::CStr {
    unsafe fn raw_borrow<'a>(input: *const libc::c_char) -> Result<&'a Self, Error> {
        ensure!(
            !input.is_null(),
            "could not borrow, unexpected null pointer"
        );
        Ok(Self::from_ptr(input))
    }
}

impl CReprOf<String> for std::ffi::CString {
    fn c_repr_of(input: String) -> Result<Self, Error> {
        std::ffi::CString::new(input)
            .context("Could not convert String to C Repr")
            .map_err(|e| e.into())
    }
}

impl CReprOf<f32> for f32 {
    fn c_repr_of(input: f32) -> Result<f32, Error> {
        Ok(input)
    }
}

impl CReprOf<i32> for i32 {
    fn c_repr_of(input: i32) -> Result<i32, Error> {
        Ok(input)
    }
}

impl CReprOf<usize> for i32 {
    fn c_repr_of(input: usize) -> Result<i32, Error> {
        Ok(input as i32)
    }
}

pub type RawPointerTo<T> = *const T;

impl<U: CReprOf<V>, V> CReprOf<Option<V>> for RawPointerTo<U> {
    fn c_repr_of(input: Option<V>) -> Result<Self, Error> {
        Ok(if let Some(inp) = input {
            U::c_repr_of(inp)?.into_raw_pointer()
        } else {
            null() as *const _
        })
    }
}

impl CReprOf<String> for RawPointerTo<libc::c_char> {
    fn c_repr_of(input: String) -> Result<Self, Error> {
        convert_to_c_string_result!(input)
    }
}

impl AsRust<String> for std::ffi::CStr {
    fn as_rust(&self) -> Result<String, Error> {
        self.to_str().map(|s| s.to_owned()).map_err(|e| e.into())
    }
}

impl AsRust<i32> for i32 {
    fn as_rust(&self) -> Result<i32, Error> {
        Ok(*self)
    }
}

impl AsRust<f32> for f32 {
    fn as_rust(&self) -> Result<f32, Error> {
        Ok(*self)
    }
}

impl<U: AsRust<V>, V> AsRust<Option<V>> for RawPointerTo<U> {
    fn as_rust(&self) -> Result<Option<V>, Error> {
        Ok(if *self != null() {
            Some(unsafe { U::as_rust(&U::from_raw_pointer(*self)?)? })
        } else {
            None
        })
    }
}

impl AsRust<String> for RawPointerTo<libc::c_char> {
    fn as_rust(&self) -> Result<String, Error> {
        Ok(create_rust_string_from!(*self))
    }
}

impl AsRust<usize> for i32 {
    fn as_rust(&self) -> Result<usize, Error> {
        Ok(*self as usize)
    }
}