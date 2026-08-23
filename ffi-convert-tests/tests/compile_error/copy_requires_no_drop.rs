// a struct with a pointer field gets a generated `Drop` impl, which is
// incompatible with `Copy`

use ffi_convert::CReprOf;

pub struct Owner {
    pub name: String,
}

#[repr(C)]
#[derive(Clone, Copy, CReprOf)]
#[target_type(Owner)]
pub struct COwner {
    pub name: *const std::ffi::c_char,
}

fn main() {}
