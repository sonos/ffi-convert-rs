// using `#[derive(CDrop)]` must emit a deprecation warning

#![deny(deprecated)]

use ffi_convert::CReprOf;

pub struct Owner {
    pub name: String,
}

#[repr(C)]
#[derive(CReprOf, ffi_convert::CDrop)]
#[target_type(Owner)]
pub struct COwner {
    pub name: *const std::ffi::c_char,
}

fn main() {}
