// an invalid field conversion must be reported on the offending field, not
// on the whole struct

use ffi_convert::CReprOf;

pub struct Owner {
    pub name: String,
    pub count: u8,
}

#[repr(C)]
#[derive(CReprOf)]
#[target_type(Owner)]
pub struct COwner {
    pub name: *const std::ffi::c_char,
    pub count: i64,
}

fn main() {}
