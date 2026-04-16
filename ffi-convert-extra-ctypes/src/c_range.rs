use std::ops::Range;

use ffi_convert::{AsRust, AsRustError, CDrop, CDropError, CReprOf, CReprOfError};

/// A utility type to represent range.
/// Note that the parametrized type T should have `CReprOf` and `AsRust` trait implemented.
///
/// # Example
///
/// ```
/// use ffi_convert::{CReprOf, AsRust, CDrop};
/// use ffi_convert_extra_ctypes::CRange;
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
