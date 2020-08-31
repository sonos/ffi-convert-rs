use std::ffi::NulError;
use std::str::Utf8Error;
use thiserror::Error;

/// A macro to convert a `std::String` to a C-compatible representation : a raw pointer to libc::c_char.
/// After calling this function, the caller is responsible for releasing the memory.
/// The [`take_back_c_string!`] macro can be used for releasing the memory.
#[macro_export]
macro_rules! convert_to_c_string {
    ($string:expr) => {
        $crate::convert_to_c_string_result!($string)?
    };
}

/// A macro to convert a `std::String` to a C-compatible representation a raw pointer to libc::c_char
/// wrapped in a Result enum.
/// After calling this function, the caller is responsible for releasing the memory.
/// The [`take_back_c_string!`] macro can be used for releasing the memory.  
#[macro_export]
macro_rules! convert_to_c_string_result {
    ($string:expr) => {
        std::ffi::CString::c_repr_of($string).map(|s| {
            use $crate::RawPointerConverter;
            s.into_raw_pointer() as *const libc::c_char
        })
    };
}

/// A macro to convert a `Vec<String>` to a C-compatible representation : a raw pointer to a CStringArray
/// After calling this function, the caller is responsible for releasing the memory.
/// The [`take_back_c_string_array!`] macro can be used for releasing the memory.
#[macro_export]
macro_rules! convert_to_c_string_array {
    ($string_vec:expr) => {{
        use $crate::RawPointerConverter;
        $crate::CStringArray::c_repr_of($string_vec)?.into_raw_pointer()
    }};
}

/// A macro to convert a `Vec<String>` to a C-compatible representation : a raw pointer to a CStringArray
/// After calling this function, the caller is responsible for releasing the memory.
/// The [`take_back_c_string_array!`] macro can be used for releasing the memory.
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

/// A macro to convert an `Option<String>` to a C-compatible representation : a raw pointer to libc::c_char if the Option enum is of variant Some,
/// or a null pointer if the Option enum is of variant None.  
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

/// Retakes the ownership of the memory pointed to by a raw pointer to a libc::c_char
#[macro_export]
macro_rules! take_back_c_string {
    ($pointer:expr) => {{
        use $crate::RawPointerConverter;
        let _ = unsafe { std::ffi::CString::from_raw_pointer($pointer) };
    }};
}

/// Retakes the ownership of the memory pointed to by a raw pointer to a libc::c_char, checking first if the pointer is not null.
#[macro_export]
macro_rules! take_back_nullable_c_string {
    ($pointer:expr) => {
        if !$pointer.is_null() {
            $crate::take_back_c_string!($pointer)
        }
    };
}

/// Retakes the ownership of the memory storing an array of C-compatible strings
#[macro_export]
macro_rules! take_back_c_string_array {
    ($pointer:expr) => {{
        use $crate::RawPointerConverter;
        let _ = unsafe { $crate::CStringArray::from_raw_pointer($pointer) };
    }};
}

/// Retakes the ownership of the memory storing an array of C-compatible strings, checking first if the provided pointer is not null.
#[macro_export]
macro_rules! take_back_nullable_c_string_array {
    ($pointer:expr) => {
        if !$pointer.is_null() {
            $crate::take_back_c_string_array!($pointer)
        }
    };
}

/// Unsafely creates an owned string from a pointer to a nul-terminated array of bytes.
#[macro_export]
macro_rules! create_rust_string_from {
    ($pointer:expr) => {{
        use $crate::RawBorrow;
        unsafe { std::ffi::CStr::raw_borrow($pointer) }?.as_rust()?
    }};
}

/// Unsafely creates an optional owned string from a pointer to a nul-terminated array of bytes.
#[macro_export]
macro_rules! create_optional_rust_string_from {
    ($pointer:expr) => {
        match unsafe { $pointer.as_ref() } {
            Some(thing) => Some($crate::create_rust_string_from!(thing)),
            None => None,
        }
    };
}

/// Unsafely creates an array of owned string from a pointer to a CStringArray.
#[macro_export]
macro_rules! create_rust_vec_string_from {
    ($pointer:expr) => {{
        use $crate::RawBorrow;
        unsafe { $crate::CStringArray::raw_borrow($pointer) }?.as_rust()?
    }};
}

/// Unsafely creates an optional array of owned string from a pointer to a CStringArray.
#[macro_export]
macro_rules! create_optional_rust_vec_string_from {
    ($pointer:expr) => {
        match unsafe { $pointer.as_ref() } {
            Some(thing) => Some($crate::create_rust_vec_string_from!(thing)),
            None => None,
        }
    };
}

macro_rules! impl_c_repr_of_for {
    ($typ:ty) => {
        impl CReprOf<$typ> for $typ {
            fn c_repr_of(input: $typ) -> Result<$typ, CReprOfError> {
                Ok(input)
            }
        }
    };

    ($from_typ:ty, $to_typ:ty) => {
        impl CReprOf<$from_typ> for $to_typ {
            fn c_repr_of(input: $from_typ) -> Result<$to_typ, CReprOfError> {
                Ok(input as $to_typ)
            }
        }
    };
}

/// implements a noop implementation of the CDrop trait for a given type.
macro_rules! impl_c_drop_for {
    ($typ:ty) => {
        impl CDrop for $typ {
            fn do_drop(&mut self) -> Result<(), CDropError> {
                Ok(())
            }
        }
    };
}

macro_rules! impl_as_rust_for {
    ($typ:ty) => {
        impl AsRust<$typ> for $typ {
            fn as_rust(&self) -> Result<$typ, AsRustError> {
                Ok(*self)
            }
        }
    };

    ($from_typ:ty, $to_typ:ty) => {
        impl AsRust<$to_typ> for $from_typ {
            fn as_rust(&self) -> Result<$to_typ, AsRustError> {
                Ok(*self as $to_typ)
            }
        }
    };
}

pub fn point_to_string(
    pointer: *mut *const libc::c_char,
    string: String,
) -> Result<(), CReprOfError> {
    unsafe { *pointer = std::ffi::CString::c_repr_of(string)?.into_raw_pointer() }
    Ok(())
}

#[derive(Error, Debug)]
pub enum CReprOfError {
    #[error("A string contains a nul bit")]
    StringContainsNullBit(#[from] NulError),
    #[error("An error occurred during conversion to C repr; {}", .0)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

/// Trait showing that the struct implementing it is a `repr(C)` compatible view of the parametrized
/// type that can be created from an object of this type.
pub trait CReprOf<T>: Sized + CDrop {
    fn c_repr_of(input: T) -> Result<Self, CReprOfError>;
}

#[derive(Error, Debug)]
pub enum CDropError {
    #[error("unexpected null pointer")]
    NullPointer(#[from] UnexpectedNullPointerError),
    #[error("An error occurred while dropping C struct: {}", .0)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

/// Trait showing that the C-like struct implementing it can free up its part of memory that are not
/// managed by Rust.
pub trait CDrop {
    fn do_drop(&mut self) -> Result<(), CDropError>;
}

#[derive(Error, Debug)]
pub enum AsRustError {
    #[error("unexpected null pointer")]
    NullPointer(#[from] UnexpectedNullPointerError),

    #[error("could not convert string as it is not UTF-8: {}", .0)]
    Utf8Error(#[from] Utf8Error),
    #[error("An error occurred during conversion to Rust: {}", .0)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}
/// Trait showing that the struct implementing it is a `repr(C)` compatible view of the parametrized
/// type and that an instance of the parametrized type can be created form this struct
pub trait AsRust<T> {
    fn as_rust(&self) -> Result<T, AsRustError>;
}

#[derive(Error, Debug)]
#[error("Could not use raw pointer: unexpected null pointer")]
pub struct UnexpectedNullPointerError;

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
    unsafe fn from_raw_pointer(input: *const T) -> Result<Self, UnexpectedNullPointerError>;

    unsafe fn drop_raw_pointer(input: *const T) -> Result<(), UnexpectedNullPointerError> {
        T::from_raw_pointer(input).map(|_| ())
    }
}

/// Trait to create borrowed references to type T, from a raw pointer to a T
pub trait RawBorrow<T> {
    unsafe fn raw_borrow<'a>(input: *const T) -> Result<&'a Self, UnexpectedNullPointerError>;
}

/// Trait to create mutable borrowed references to type T, from a raw pointer to a T
pub trait RawBorrowMut<T> {
    unsafe fn raw_borrow_mut<'a>(input: *mut T)
        -> Result<&'a mut Self, UnexpectedNullPointerError>;
}

/// TODO custom derive instead of generic impl, this would prevent CString from having 2 impls...
/// Trait representing conversion operations from and to owned type T to a raw pointer to T
impl<T> RawPointerConverter<T> for T {
    fn into_raw_pointer(self) -> *const T {
        Box::into_raw(Box::new(self)) as _
    }

    unsafe fn from_raw_pointer(input: *const T) -> Result<T, UnexpectedNullPointerError> {
        if input.is_null() {
            Err(UnexpectedNullPointerError)
        } else {
            Ok(*Box::from_raw(input as *mut T))
        }
    }
}

/// Trait that allows obtaining a borrowed reference to a type T from a raw pointer to T
impl<T> RawBorrow<T> for T {
    unsafe fn raw_borrow<'a>(input: *const T) -> Result<&'a Self, UnexpectedNullPointerError> {
        input.as_ref().ok_or_else(|| UnexpectedNullPointerError)
    }
}

/// Trait that allows obtaining a mutable borrowed reference to a type T from a raw pointer to T
impl<T> RawBorrowMut<T> for T {
    unsafe fn raw_borrow_mut<'a>(
        input: *mut T,
    ) -> Result<&'a mut Self, UnexpectedNullPointerError> {
        input.as_mut().ok_or_else(|| UnexpectedNullPointerError)
    }
}

impl RawPointerConverter<libc::c_void> for std::ffi::CString {
    fn into_raw_pointer(self) -> *const libc::c_void {
        self.into_raw() as _
    }

    unsafe fn from_raw_pointer(
        input: *const libc::c_void,
    ) -> Result<Self, UnexpectedNullPointerError> {
        if input.is_null() {
            Err(UnexpectedNullPointerError)
        } else {
            Ok(std::ffi::CString::from_raw(input as *mut libc::c_char))
        }
    }
}

impl RawPointerConverter<libc::c_char> for std::ffi::CString {
    fn into_raw_pointer(self) -> *const libc::c_char {
        self.into_raw() as _
    }

    unsafe fn from_raw_pointer(
        input: *const libc::c_char,
    ) -> Result<Self, UnexpectedNullPointerError> {
        if input.is_null() {
            Err(UnexpectedNullPointerError)
        } else {
            Ok(std::ffi::CString::from_raw(input as *mut libc::c_char))
        }
    }
}

impl RawBorrow<libc::c_char> for std::ffi::CStr {
    unsafe fn raw_borrow<'a>(
        input: *const libc::c_char,
    ) -> Result<&'a Self, UnexpectedNullPointerError> {
        if input.is_null() {
            Err(UnexpectedNullPointerError)
        } else {
            Ok(Self::from_ptr(input))
        }
    }
}

impl_c_drop_for!(usize);
impl_c_drop_for!(u8);
impl_c_drop_for!(i16);
impl_c_drop_for!(u16);
impl_c_drop_for!(i32);
impl_c_drop_for!(u32);
impl_c_drop_for!(i64);
impl_c_drop_for!(u64);
impl_c_drop_for!(f32);
impl_c_drop_for!(f64);
impl_c_drop_for!(bool);
impl_c_drop_for!(std::ffi::CString);

impl_c_repr_of_for!(usize);
impl_c_repr_of_for!(i16);
impl_c_repr_of_for!(u16);
impl_c_repr_of_for!(i32);
impl_c_repr_of_for!(u32);
impl_c_repr_of_for!(i64);
impl_c_repr_of_for!(u64);
impl_c_repr_of_for!(f32);
impl_c_repr_of_for!(f64);
impl_c_repr_of_for!(bool);

impl_c_repr_of_for!(usize, i32);

impl CReprOf<bool> for u8 {
    fn c_repr_of(input: bool) -> Result<u8, CReprOfError> {
        Ok(if input { 1 } else { 0 })
    }
}

impl CReprOf<String> for std::ffi::CString {
    fn c_repr_of(input: String) -> Result<Self, CReprOfError> {
        Ok(std::ffi::CString::new(input)?)
    }
}

impl_as_rust_for!(usize);
impl_as_rust_for!(i16);
impl_as_rust_for!(u16);
impl_as_rust_for!(i32);
impl_as_rust_for!(u32);
impl_as_rust_for!(i64);
impl_as_rust_for!(u64);
impl_as_rust_for!(f32);
impl_as_rust_for!(f64);
impl_as_rust_for!(bool);

impl_as_rust_for!(i32, usize);

impl AsRust<bool> for u8 {
    fn as_rust(&self) -> Result<bool, AsRustError> {
        Ok((*self) != 0)
    }
}

impl AsRust<String> for std::ffi::CStr {
    fn as_rust(&self) -> Result<String, AsRustError> {
        self.to_str().map(|s| s.to_owned()).map_err(|e| e.into())
    }
}
