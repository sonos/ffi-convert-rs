use std::ffi::NulError;
use std::mem::MaybeUninit;
use std::str::Utf8Error;

use thiserror::Error;

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

macro_rules! impl_rawpointerconverter_for {
    ($typ:ty) => {
        impl RawPointerConverter<$typ> for $typ {
            fn into_raw_pointer(self) -> *const $typ {
                convert_into_raw_pointer(self)
            }
            fn into_raw_pointer_mut(self) -> *mut $typ {
                convert_into_raw_pointer_mut(self)
            }
            unsafe fn from_raw_pointer(
                input: *const $typ,
            ) -> Result<Self, UnexpectedNullPointerError> {
                unsafe { take_back_from_raw_pointer(input) }
            }
            unsafe fn from_raw_pointer_mut(
                input: *mut $typ,
            ) -> Result<Self, UnexpectedNullPointerError> {
                unsafe { take_back_from_raw_pointer_mut(input) }
            }
        }
    };
}

/// Error returned by [`CReprOf::c_repr_of`].
#[derive(Error, Debug)]
pub enum CReprOfError {
    /// A Rust [`String`] contained an interior `NUL` byte and therefore could
    /// not be converted to a C string.
    #[error("A string contains a nul bit")]
    StringContainsNullBit(#[from] NulError),
    /// Custom error returned by a manual or overridden implementation.
    #[error("An error occurred during conversion to C repr; {}", .0)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

/// Consuming conversion **from** an idiomatic Rust value **to** its
/// `#[repr(C)]` mirror.
///
/// Implementing `CReprOf<U>` for `T` states that `T` is a C-compatible layout
/// of the Rust value `U` and that a `T` can be built from a `U`. The
/// implementation owns any heap memory it allocates, and that memory is
/// reclaimed by the corresponding [`CDrop`] implementation.
///
/// `CReprOf` and [`CDrop`] share an ownership contract — each allocation
/// `c_repr_of` performs must be freeable by `do_drop`, and vice versa. The
/// derives enforce that contract by generating both sides together. If one
/// is derived, the other must be too; mixing a derived impl with a
/// hand-written one is a recipe for double frees or leaks.
pub trait CReprOf<T>: Sized + CDrop {
    /// Consume `input` and return its C-compatible representation.
    fn c_repr_of(input: T) -> Result<Self, CReprOfError>;
}

/// Error returned by [`CDrop::do_drop`].
#[derive(Error, Debug)]
pub enum CDropError {
    /// A non-nullable pointer field was found to be null while dropping.
    #[error("unexpected null pointer")]
    NullPointer(#[from] UnexpectedNullPointerError),
    /// Custom error returned by a manual implementation.
    #[error("An error occurred while dropping C struct: {}", .0)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

/// Releases any heap memory owned by a C-compatible value that is not managed
/// by Rust's regular `Drop` mechanism (typically `Box`-allocated data behind
/// raw pointer fields).
///
/// The [`#[derive(CDrop)]`](ffi_convert_derive::CDrop) macro emits both a
/// [`CDrop`] and a matching [`Drop`] impl that calls
/// [`do_drop`](CDrop::do_drop). The two should always ship together —
/// a [`CDrop`] impl by itself does nothing until something calls `do_drop`,
/// so leaving the value to Rust's regular `drop` leaks every pointer field
/// it owns. Use `#[no_drop_impl]` only when you need to write [`Drop`]
/// yourself, and make sure that manual [`Drop`] calls `do_drop`.
///
/// [`CDrop`] and [`CReprOf`] share an ownership contract: the derived
/// [`CDrop`] assumes every pointer field was produced by the derived
/// [`CReprOf`] (i.e. via `Box::into_raw`). Mixing a derived [`CDrop`] with
/// a hand-written [`CReprOf`] — or vice versa — is how you get double
/// frees or leaks. Derive both or write both.
pub trait CDrop {
    /// Release any Rust-owned memory referenced by `self`. Typically called
    /// from the generated [`Drop`] implementation; in that case errors are
    /// silently ignored.
    fn do_drop(&mut self) -> Result<(), CDropError>;
}

/// Error returned by [`AsRust::as_rust`].
#[derive(Error, Debug)]
pub enum AsRustError {
    /// A non-nullable pointer field was null.
    #[error("unexpected null pointer")]
    NullPointer(#[from] UnexpectedNullPointerError),
    /// A C string field was not valid UTF-8.
    #[error("could not convert string as it is not UTF-8: {}", .0)]
    Utf8Error(#[from] Utf8Error),
    /// Custom error returned by a manual implementation.
    #[error("An error occurred during conversion to Rust: {}", .0)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

/// Non-consuming conversion **from** a `#[repr(C)]` value **back** to an
/// owned, idiomatic Rust value.
///
/// `AsRust<U>` takes `&self` and returns a freshly-allocated `U`, performing a
/// deep copy of any pointer field it owns. After the call the original
/// C-compatible struct is still valid — only the C side is expected to free
/// it.
pub trait AsRust<T> {
    /// Return a freshly-allocated Rust value equivalent to `self`.
    fn as_rust(&self) -> Result<T, AsRustError>;
}

/// Returned whenever a raw pointer was expected to be non-null but was.
#[derive(Error, Debug)]
#[error("Could not use raw pointer: unexpected null pointer")]
pub struct UnexpectedNullPointerError;

/// Boxes a Rust value into a raw pointer suitable for crossing an FFI
/// boundary, and takes it back.
///
/// `into_raw_pointer` leaks the value (via [`Box::into_raw`]) and must be
/// paired with a later [`from_raw_pointer`](RawPointerConverter::from_raw_pointer)
/// or [`drop_raw_pointer`](RawPointerConverter::drop_raw_pointer) call to
/// avoid a leak. If you only need to read the value behind the pointer
/// without taking ownership — because the C caller still owns the allocation
/// — use [`RawBorrow`] instead.
///
/// The `from_raw_pointer` family is unsafe because the compiler cannot verify
/// that the pointer was actually produced by `into_raw_pointer`. Calling it
/// twice on the same pointer is a double free.
pub trait RawPointerConverter<T>: Sized {
    /// Creates a raw pointer from the value and leaks it, you should use [`Self::from_raw_pointer`]
    /// or [`Self::drop_raw_pointer`] to free the value when you're done with it.
    fn into_raw_pointer(self) -> *const T;
    /// Creates a mutable raw pointer from the value and leaks it, you should use
    /// [`Self::from_raw_pointer_mut`] or [`Self::drop_raw_pointer_mut`] to free the value when
    /// you're done with it.
    fn into_raw_pointer_mut(self) -> *mut T;
    /// Takes back control of a raw pointer created by [`Self::into_raw_pointer`].
    /// # Safety
    /// This method is unsafe because passing it a pointer that was not created by
    /// [`Self::into_raw_pointer`] can lead to memory problems. Also note that passing the same pointer
    /// twice to this function will probably result in a double free
    unsafe fn from_raw_pointer(input: *const T) -> Result<Self, UnexpectedNullPointerError>;
    /// Takes back control of a raw pointer created by [`Self::into_raw_pointer_mut`].
    /// # Safety
    /// This method is unsafe because passing it a pointer that was not created by
    /// [`Self::into_raw_pointer_mut`] can lead to memory problems. Also note that passing the same
    /// pointer twice to this function will probably result in a double free
    unsafe fn from_raw_pointer_mut(input: *mut T) -> Result<Self, UnexpectedNullPointerError>;

    /// Takes back control of a raw pointer created by [`Self::into_raw_pointer`] and drop it.
    /// # Safety
    /// This method is unsafe for the same reasons as [`Self::from_raw_pointer`]
    unsafe fn drop_raw_pointer(input: *const T) -> Result<(), UnexpectedNullPointerError> {
        unsafe { Self::from_raw_pointer(input) }.map(|_| ())
    }

    /// Takes back control of a raw pointer created by [`Self::into_raw_pointer_mut`] and drops it.
    /// # Safety
    /// This method is unsafe for the same reasons a [`Self::from_raw_pointer_mut`]
    unsafe fn drop_raw_pointer_mut(input: *mut T) -> Result<(), UnexpectedNullPointerError> {
        unsafe { Self::from_raw_pointer_mut(input) }.map(|_| ())
    }
}

#[doc(hidden)]
pub fn convert_into_raw_pointer<T>(pointee: T) -> *const T {
    Box::into_raw(Box::new(pointee)) as _
}

#[doc(hidden)]
pub fn convert_into_raw_pointer_mut<T>(pointee: T) -> *mut T {
    Box::into_raw(Box::new(pointee))
}

#[doc(hidden)]
pub unsafe fn take_back_from_raw_pointer<T>(
    input: *const T,
) -> Result<T, UnexpectedNullPointerError> {
    unsafe { take_back_from_raw_pointer_mut(input as _) }
}

#[doc(hidden)]
pub unsafe fn take_back_from_raw_pointer_mut<T>(
    input: *mut T,
) -> Result<T, UnexpectedNullPointerError> {
    if input.is_null() {
        Err(UnexpectedNullPointerError)
    } else {
        Ok(*unsafe { Box::from_raw(input) })
    }
}

/// Turn a `*const T` into a borrowed `&T` without taking ownership.
///
/// Use this when the pointer was handed to you by C and the C side retains
/// ownership of the allocation. Blanket-implemented for every `T`; also
/// implemented for [`std::ffi::CStr`] over `*const libc::c_char`.
pub trait RawBorrow<T> {
    /// Borrow the value behind `input`, or return
    /// [`UnexpectedNullPointerError`] if it is null.
    ///
    /// # Safety
    /// This is a thin wrapper around `<*const T>::as_ref` and is unsafe for
    /// the same reasons: `input` must point to a valid, properly aligned
    /// `T` that lives for at least `'a`.
    unsafe fn raw_borrow<'a>(input: *const T) -> Result<&'a Self, UnexpectedNullPointerError>;
}

/// Mutable counterpart of [`RawBorrow`].
pub trait RawBorrowMut<T> {
    /// Borrow the value behind `input` mutably, or return
    /// [`UnexpectedNullPointerError`] if it is null.
    ///
    /// # Safety
    /// This is a thin wrapper around `<*mut T>::as_mut` and is unsafe for the
    /// same reasons.
    unsafe fn raw_borrow_mut<'a>(input: *mut T)
    -> Result<&'a mut Self, UnexpectedNullPointerError>;
}

impl<T> RawBorrow<T> for T {
    unsafe fn raw_borrow<'a>(input: *const T) -> Result<&'a Self, UnexpectedNullPointerError> {
        unsafe { input.as_ref() }.ok_or(UnexpectedNullPointerError)
    }
}

impl<T> RawBorrowMut<T> for T {
    unsafe fn raw_borrow_mut<'a>(
        input: *mut T,
    ) -> Result<&'a mut Self, UnexpectedNullPointerError> {
        unsafe { input.as_mut() }.ok_or(UnexpectedNullPointerError)
    }
}

impl RawPointerConverter<libc::c_void> for std::ffi::CString {
    fn into_raw_pointer(self) -> *const libc::c_void {
        self.into_raw() as _
    }

    fn into_raw_pointer_mut(self) -> *mut libc::c_void {
        self.into_raw() as _
    }

    unsafe fn from_raw_pointer(
        input: *const libc::c_void,
    ) -> Result<Self, UnexpectedNullPointerError> {
        unsafe { Self::from_raw_pointer_mut(input as *mut libc::c_void) }
    }

    unsafe fn from_raw_pointer_mut(
        input: *mut libc::c_void,
    ) -> Result<Self, UnexpectedNullPointerError> {
        if input.is_null() {
            Err(UnexpectedNullPointerError)
        } else {
            Ok(unsafe { std::ffi::CString::from_raw(input as *mut libc::c_char) })
        }
    }
}

impl RawPointerConverter<libc::c_char> for std::ffi::CString {
    fn into_raw_pointer(self) -> *const libc::c_char {
        self.into_raw() as _
    }

    fn into_raw_pointer_mut(self) -> *mut libc::c_char {
        self.into_raw()
    }

    unsafe fn from_raw_pointer(
        input: *const libc::c_char,
    ) -> Result<Self, UnexpectedNullPointerError> {
        unsafe { Self::from_raw_pointer_mut(input as *mut libc::c_char) }
    }

    unsafe fn from_raw_pointer_mut(
        input: *mut libc::c_char,
    ) -> Result<Self, UnexpectedNullPointerError> {
        if input.is_null() {
            Err(UnexpectedNullPointerError)
        } else {
            Ok(unsafe { std::ffi::CString::from_raw(input as *mut libc::c_char) })
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
            Ok(unsafe { Self::from_ptr(input) })
        }
    }
}

impl_c_drop_for!(usize);
impl_c_drop_for!(i8);
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
impl_c_repr_of_for!(i8);
impl_c_repr_of_for!(u8);
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

impl CReprOf<String> for std::ffi::CString {
    fn c_repr_of(input: String) -> Result<Self, CReprOfError> {
        Ok(std::ffi::CString::new(input)?)
    }
}

impl_as_rust_for!(usize);
impl_as_rust_for!(i8);
impl_as_rust_for!(u8);
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

impl AsRust<String> for std::ffi::CStr {
    fn as_rust(&self) -> Result<String, AsRustError> {
        self.to_str().map(|s| s.to_owned()).map_err(|e| e.into())
    }
}

impl_rawpointerconverter_for!(usize);
impl_rawpointerconverter_for!(i16);
impl_rawpointerconverter_for!(u16);
impl_rawpointerconverter_for!(i32);
impl_rawpointerconverter_for!(u32);
impl_rawpointerconverter_for!(i64);
impl_rawpointerconverter_for!(u64);
impl_rawpointerconverter_for!(f32);
impl_rawpointerconverter_for!(f64);
impl_rawpointerconverter_for!(bool);

impl<U, T: CReprOf<U>, const N: usize> CReprOf<[U; N]> for [T; N]
where
    [T; N]: CDrop,
{
    fn c_repr_of(values: [U; N]) -> Result<[T; N], CReprOfError> {
        let mut array: [MaybeUninit<T>; N] = [const { MaybeUninit::uninit() }; N];

        for (n, value) in values.into_iter().enumerate() {
            let item = &mut array[n];

            match T::c_repr_of(value) {
                Ok(value) => {
                    item.write(value);
                }
                Err(err) => {
                    // Drop initialized items
                    for item in &mut array[0..n] {
                        // SAFETY: `item` is certain to be initialized
                        unsafe {
                            let _ = item.assume_init_mut().do_drop();
                        }
                    }

                    return Err(err);
                }
            }
        }

        // SAFETY: `array` is certain to be initialized
        let array = unsafe {
            // TODO: array_assume_init: https://github.com/rust-lang/rust/issues/96097
            (&raw const array).cast::<[T; N]>().read()
        };
        Ok(array)
    }
}

impl<T: CDrop, const N: usize> CDrop for [T; N] {
    fn do_drop(&mut self) -> Result<(), CDropError> {
        let mut result = Ok(());

        for value in self {
            if let Err(err) = value.do_drop()
                && result.is_ok()
            {
                result = Err(err);
            }
        }

        result
    }
}

impl<U: AsRust<T>, T, const N: usize> AsRust<[T; N]> for [U; N] {
    fn as_rust(&self) -> Result<[T; N], AsRustError> {
        let mut array: [MaybeUninit<T>; N] = [const { MaybeUninit::uninit() }; N];

        for (n, value) in self.iter().enumerate() {
            let item = &mut array[n];

            match value.as_rust() {
                Ok(value) => {
                    item.write(value);
                }
                Err(err) => {
                    // Drop initialized items
                    for item in &mut array[0..n] {
                        // SAFETY: `item` is certain to be initialized
                        unsafe {
                            item.assume_init_drop();
                        }
                    }

                    return Err(err);
                }
            }
        }

        // SAFETY: `array` is certain to be initialized
        let array = unsafe {
            // TODO: array_assume_init: https://github.com/rust-lang/rust/issues/96097
            (&raw const array).cast::<[T; N]>().read()
        };
        Ok(array)
    }
}
