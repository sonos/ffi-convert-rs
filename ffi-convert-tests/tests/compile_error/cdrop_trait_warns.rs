// importing or referencing the `CDrop` trait must emit a deprecation warning

#![deny(deprecated)]

use ffi_convert::CDrop;

fn assert_impls_cdrop<T: CDrop>() {}

fn main() {}
