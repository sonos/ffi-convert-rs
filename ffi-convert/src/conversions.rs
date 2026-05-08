use std::ffi::NulError;
use std::mem::MaybeUninit;
use std::str::Utf8Error;

use thiserror::Error;

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
/// of the Rust value `U` and that a `T` can be built from a `U`. The resulting
/// `T` owns any heap memory it allocates, and that memory is reclaimed by the
/// corresponding [`CDrop`] implementation.
///
/// see  [Deriving the traits](crate#deriving-the-traits).
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

/// Releases heap memory referenced by a C-compatible value behind raw pointer
/// fields (typically data that was moved into a `Box` and leaked via
/// [`Box::into_raw`]).
///
/// By default, [`#[derive(CDrop)]`](ffi_convert_derive::CDrop) emits both a
/// [`CDrop`] impl and a matching [`Drop`] impl that calls
/// [`do_drop`](CDrop::do_drop), so dropping the value through Rust's normal
/// path releases its pointer fields. `#[no_drop_impl]` suppresses only the
/// [`Drop`] impl; in that case a handwritten [`Drop`] must call `do_drop`
/// itself, otherwise the pointer fields are leaked.
///
/// see [Deriving the traits](crate#deriving-the-traits).
pub trait CDrop {
    /// Release any Rust-owned memory referenced by `self`. The derived
    /// [`Drop`] impl calls this and discards the result, so errors raised
    /// from a normal drop are not observed.
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
/// `AsRust<U>` takes `&self` and returns a freshly-allocated `U`, copying data
/// out of any pointer field by borrowing through [`RawBorrow`]. The original
/// C-compatible value is left untouched and its allocations are not freed;
/// releasing them is the caller's responsibility.
///
/// This is the recommended entry point for values handed to Rust by C — see
/// the crate-level [Philosophy](crate#philosophy).
pub trait AsRust<T> {
    /// Return a freshly-allocated Rust value equivalent to `self`.
    fn as_rust(&self) -> Result<T, AsRustError>;
}

/// Returned when a raw pointer was expected to be non-null but was null.
#[derive(Error, Debug)]
#[error("Could not use raw pointer: unexpected null pointer")]
pub struct UnexpectedNullPointerError;

/// Moves a Rust value onto the heap and exposes it as a raw pointer suitable
/// for crossing an FFI boundary, then takes it back on the return trip.
///
/// The default impls box the value and leak it via [`Box::into_raw`]. Each
/// pointer produced by `into_raw_pointer` must eventually be passed to
/// [`from_raw_pointer`](RawPointerConverter::from_raw_pointer) or
/// [`drop_raw_pointer`](RawPointerConverter::drop_raw_pointer); otherwise the
/// allocation is leaked. To read the value behind a pointer without taking
/// ownership (e.g. when the C caller retains ownership), use [`RawBorrow`]
/// — see the crate-level [Philosophy](crate#philosophy).
///
/// The `from_raw_pointer` family is `unsafe` because the compiler cannot
/// verify that the pointer originated from `into_raw_pointer`. Passing the
/// same pointer twice frees the same allocation twice.
pub trait RawPointerConverter<T>: Sized {
    /// Leak the value behind a raw pointer. Pair with [`Self::from_raw_pointer`]
    /// or [`Self::drop_raw_pointer`] to release the allocation.
    fn into_raw_pointer(self) -> *const T;
    /// Leak the value behind a mutable raw pointer. Pair with
    /// [`Self::from_raw_pointer_mut`] or [`Self::drop_raw_pointer_mut`] to
    /// release the allocation.
    fn into_raw_pointer_mut(self) -> *mut T;
    /// Take back ownership of a raw pointer previously produced by
    /// [`Self::into_raw_pointer`]. Returns [`UnexpectedNullPointerError`] if
    /// `input` is null.
    /// # Safety
    /// `input` must have been produced by [`Self::into_raw_pointer`] and must
    /// not be used afterwards. Passing the same pointer twice frees the same
    /// allocation twice.
    unsafe fn from_raw_pointer(input: *const T) -> Result<Self, UnexpectedNullPointerError>;
    /// Take back ownership of a raw pointer previously produced by
    /// [`Self::into_raw_pointer_mut`]. Returns [`UnexpectedNullPointerError`]
    /// if `input` is null.
    /// # Safety
    /// `input` must have been produced by [`Self::into_raw_pointer_mut`] and
    /// must not be used afterwards. Passing the same pointer twice frees the
    /// same allocation twice.
    unsafe fn from_raw_pointer_mut(input: *mut T) -> Result<Self, UnexpectedNullPointerError>;

    /// Take back ownership of a pointer produced by [`Self::into_raw_pointer`]
    /// and drop the value.
    /// # Safety
    /// Same requirements as [`Self::from_raw_pointer`].
    unsafe fn drop_raw_pointer(input: *const T) -> Result<(), UnexpectedNullPointerError> {
        unsafe { Self::from_raw_pointer(input) }.map(|_| ())
    }

    /// Take back ownership of a pointer produced by [`Self::into_raw_pointer_mut`]
    /// and drop the value.
    /// # Safety
    /// Same requirements as [`Self::from_raw_pointer_mut`].
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
/// ownership of the allocation — see the crate-level
/// [Philosophy](crate#philosophy). A blanket impl `impl<T> RawBorrow<T> for T`
/// covers every type; [`std::ffi::CStr`] additionally implements
/// `RawBorrow<libc::c_char>`.
pub trait RawBorrow<T> {
    /// Borrow the value behind `input`, or return
    /// [`UnexpectedNullPointerError`] if it is null.
    ///
    /// # Safety
    /// Thin wrapper around `<*const T>::as_ref` with the same requirements:
    /// `input` must point to a valid, properly aligned `T` that lives for at
    /// least `'a`.
    unsafe fn raw_borrow<'a>(input: *const T) -> Result<&'a Self, UnexpectedNullPointerError>;
}

/// Mutable counterpart of [`RawBorrow`].
pub trait RawBorrowMut<T> {
    /// Borrow the value behind `input` mutably, or return
    /// [`UnexpectedNullPointerError`] if it is null.
    ///
    /// # Safety
    /// Thin wrapper around `<*mut T>::as_mut` with the same requirements.
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

macro_rules! impl_noop_c_drop_for {
    ($typ:ty) => {
        impl CDrop for $typ {
            fn do_drop(&mut self) -> Result<(), CDropError> {
                Ok(())
            }
        }
    };
}

impl_noop_c_drop_for!(usize);
impl_noop_c_drop_for!(i8);
impl_noop_c_drop_for!(u8);
impl_noop_c_drop_for!(i16);
impl_noop_c_drop_for!(u16);
impl_noop_c_drop_for!(i32);
impl_noop_c_drop_for!(u32);
impl_noop_c_drop_for!(i64);
impl_noop_c_drop_for!(u64);
impl_noop_c_drop_for!(f32);
impl_noop_c_drop_for!(f64);
impl_noop_c_drop_for!(bool);
impl_noop_c_drop_for!(std::ffi::CString);

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
