# ffi-convert

![ffi-convert](https://github.com/sonos/ffi-convert-rs/workflows/Rust/badge.svg?branch=main&event=push)
[![ffi-convert on crates.io](https://img.shields.io/crates/v/ffi-convert.svg)](https://crates.io/crates/ffi-convert)
[![ffi-convert documentation](https://docs.rs/ffi-convert/badge.svg)](https://docs.rs/ffi-convert/)

**Convert between idiomatic Rust values and C-compatible data structures with
a minimum of unsafe ceremony.**

`ffi-convert` provides two conversion traits — `CReprOf` (Rust → C) and
`AsRust` (C → Rust) — plus `CDrop` and `RawPointerConverter` to handle
ownership of pointer fields, and derive macros that take care of the
boilerplate.

```rust
use ffi_convert::{AsRust, CDrop, CReprOf};
use libc::{c_char, c_float};

pub struct Pizza {
    pub name: String,
    pub weight: f32,
}

#[repr(C)]
#[derive(CReprOf, AsRust, CDrop)]
#[target_type(Pizza)]
pub struct CPizza {
    pub name: *const c_char,
    pub weight: c_float,
}

let pizza = Pizza { name: "Margarita".to_owned(), weight: 450.0 };
let c_pizza = CPizza::c_repr_of(pizza).unwrap();        // Rust -> C
let again: Pizza = c_pizza.as_rust().unwrap();          // C    -> Rust
```

## Workspace layout

| Crate                                                                      | What's in it                                                                                           |
|----------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------|
| [`ffi-convert`](./ffi-convert)                                             | The conversion traits (`CReprOf`, `AsRust`, `CDrop`, `RawPointerConverter`, `RawBorrow`).              |
| [`ffi-convert-derive`](./ffi-convert-derive)                               | `#[derive(...)]` macros for all four conversion traits. Re-exported from `ffi-convert`.                |
| [`ffi-convert-extra-ctypes`](./ffi-convert-extra-ctypes)                   | Optional C-compatible containers: `CArray<T>` (`Vec<U>`), `CStringArray` (`Vec<String>`), `CRange<T>`. |
| [`ffi-convert-tests`](./ffi-convert-tests)                                 | Workspace tests, including C round-trip tests with AddressSanitizer / MemorySanitizer.                 |

Full documentation lives on [docs.rs/ffi-convert](https://docs.rs/ffi-convert),
including the type-mapping table, attribute reference, and caveats that apply
to the derives.

More on open source projects at Sonos
[here](https://tech-blog.sonos.com/posts/three-open-source-sonos-projects-in-rust/).

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
