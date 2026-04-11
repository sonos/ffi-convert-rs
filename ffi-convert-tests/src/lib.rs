use anyhow::{bail, Result};
use ffi_convert::*;
use std::ops::Range;

#[macro_export]
macro_rules! generate_round_trip_rust_c_rust {
    ($func_name:ident, $rust_struct:ty, $c_struct:ty, $builder:block) => {
        #[test]
        fn $func_name() {
            use $crate::round_trip_test_rust_c_rust;
            let item = $builder;
            round_trip_test_rust_c_rust::<$c_struct, $rust_struct>(item)
                .expect("Round trip test failed!");
        }
    };
}

pub fn round_trip_test_rust_c_rust<T, U>(value: U) -> Result<()>
where
    T: AsRust<U> + CReprOf<U>,
    U: Clone + PartialEq,
{
    let value2 = value.clone();
    let intermediate: T = T::c_repr_of(value2)?;
    let value_roundtrip: U = intermediate.as_rust()?;

    if value != value_roundtrip {
        bail!("The value is not the same before and after the roundtrip");
    }

    Ok(())
}

#[derive(Clone, Debug, PartialEq)]
pub struct Pancake {
    pub name: String,
    pub description: Option<String>,
    pub start: f32,
    pub end: Option<f32>,
    pub float_array: [f32; 4],
    pub dummy: Dummy,
    pub sauce: Option<Sauce>,
    pub toppings: Vec<Topping>,
    pub layers: Option<Vec<Layer>>,
    pub base_layers: [Layer; 3],
    pub is_delicious: bool,
    pub range: Range<usize>,
    pub some_futile_info: Option<String>,
    pub flattened_range: Range<i64>,
    pub field_with_specific_rust_name: String,
    pub pancake_data: Option<Vec<u8>>,
    pub extra_ice_cream_flavor: Flavor,
}

#[repr(C)]
#[derive(CReprOf, AsRust, CDrop, RawPointerConverter)]
#[target_type(Pancake)]
#[as_rust_extra_field(some_futile_info = None)]
#[as_rust_extra_field(flattened_range = self.flattened_range_start..self.flattened_range_end)]
pub struct CPancake {
    name: *const libc::c_char,
    #[nullable]
    description: *const libc::c_char,
    start: f32,
    #[nullable]
    end: *const f32,
    float_array: [f32; 4],
    dummy: CDummy,
    #[nullable]
    sauce: *const CSauce,
    toppings: *const CArray<CTopping>,
    #[nullable]
    layers: *const CArray<CLayer>,
    base_layers: [CLayer; 3],
    is_delicious: bool,
    pub range: CRange<i32>,
    #[c_repr_of_convert(input.flattened_range.start)]
    flattened_range_start: i64,
    #[c_repr_of_convert(input.flattened_range.end)]
    flattened_range_end: i64,
    #[target_name(field_with_specific_rust_name)]
    pub field_with_specific_c_name: *const libc::c_char,
    #[nullable]
    pancake_data: *const CArray<u8>,
    extra_ice_cream_flavor: CFlavor,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sauce {
    pub volume: f32,
}

#[repr(C)]
#[derive(CReprOf, AsRust, CDrop, RawPointerConverter)]
#[target_type(Sauce)]
pub struct CSauce {
    volume: f32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Topping {
    pub amount: i32,
}

#[repr(C)]
#[derive(CReprOf, AsRust, CDrop, RawPointerConverter)]
#[target_type(Topping)]
pub struct CTopping {
    amount: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layer {
    pub number: i32,
    pub subtitle: Option<String>,
}

#[repr(C)]
#[derive(CReprOf, AsRust, CDrop, RawPointerConverter)]
#[target_type(Layer)]
pub struct CLayer {
    number: i32,
    #[nullable]
    subtitle: *const libc::c_char,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dummy {
    pub count: i32,
    pub describe: String,
}

#[repr(C)]
#[derive(CReprOf, AsRust, CDrop, RawPointerConverter)]
#[target_type(Dummy)]
pub struct CDummy {
    count: i32,
    describe: *const libc::c_char,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Flavor {
    Vanilla,
    Chocolate,
    Strawberry,
}

#[repr(C)]
#[derive(CReprOf, AsRust, CDrop)]
#[target_type(Flavor)]
pub enum CFlavor {
    Vanilla,
    Chocolate,
    Strawberry,
}

/// # Safety
///
/// `input` must be a valid pointer to a `CPancake`.
#[no_mangle]
pub unsafe extern "C" fn pancake_round_trip(input: *const CPancake) -> *const CPancake {
    let c_pancake = unsafe { &*input };
    let rust_pancake: Pancake = c_pancake.as_rust().expect("Failed to convert to Rust");
    let c_pancake_roundtrip = CPancake::c_repr_of(rust_pancake).expect("Failed to convert to C");
    Box::into_raw(Box::new(c_pancake_roundtrip))
}

/// # Safety
///
/// `pancake` must be a pointer returned by `pancake_round_trip`. It must not be used after this call.
#[no_mangle]
pub unsafe extern "C" fn pancake_free(pancake: *const CPancake) {
    unsafe { drop(Box::from_raw(pancake as *mut CPancake)) }
}

#[cfg(test)]
mod tests {
    use super::*;

    generate_round_trip_rust_c_rust!(round_trip_flavor_vanilla, Flavor, CFlavor, {
        Flavor::Vanilla
    });

    generate_round_trip_rust_c_rust!(round_trip_flavor_chocolate, Flavor, CFlavor, {
        Flavor::Chocolate
    });

    generate_round_trip_rust_c_rust!(round_trip_sauce, Sauce, CSauce, { Sauce { volume: 4.2 } });

    generate_round_trip_rust_c_rust!(round_trip_topping, Topping, CTopping, {
        Topping { amount: 2 }
    });

    generate_round_trip_rust_c_rust!(round_trip_dummy, Dummy, CDummy, {
        Dummy {
            count: 2,
            describe: "yo".to_string(),
        }
    });

    generate_round_trip_rust_c_rust!(round_trip_layer, Layer, CLayer, {
        Layer {
            number: 1,
            subtitle: Some(String::from("first layer")),
        }
    });

    generate_round_trip_rust_c_rust!(round_trip_pancake, Pancake, CPancake, {
        Pancake {
            name: String::from("Here is your pancake"),
            description: Some("I'm delicious ! ".to_string()),
            start: 0.0,
            end: Some(2.0),
            float_array: [1.0, 2.0, 3.0, 4.0],
            dummy: Dummy {
                count: 2,
                describe: "yo".to_string(),
            },
            sauce: Some(Sauce { volume: 32.23 }),
            toppings: vec![Topping { amount: 2 }, Topping { amount: 3 }],
            layers: Some(vec![Layer {
                number: 1,
                subtitle: Some(String::from("first layer")),
            }]),
            base_layers: [
                Layer {
                    number: 0,
                    subtitle: Some(String::from("flour")),
                },
                Layer {
                    number: 1,
                    subtitle: Some(String::from("dough")),
                },
                Layer {
                    number: 2,
                    subtitle: Some(String::from("tomato")),
                },
            ],
            is_delicious: true,
            range: Range { start: 20, end: 30 },
            some_futile_info: None,
            flattened_range: Range { start: 42, end: 64 },
            field_with_specific_rust_name: "renamed field".to_string(),
            pancake_data: Some(vec![1, 2, 3]),
            extra_ice_cream_flavor: Flavor::Chocolate,
        }
    });

    generate_round_trip_rust_c_rust!(round_trip_pancake_2, Pancake, CPancake, {
        Pancake {
            name: String::from("Here is your pancake"),
            description: Some("I'm delicious ! ".to_string()),
            start: 0.0,
            end: None,
            float_array: [8.0, -1.0, f32::INFINITY, -0.0],
            dummy: Dummy {
                count: 2,
                describe: "yo".to_string(),
            },
            sauce: None,
            toppings: vec![],
            layers: Some(vec![]),
            base_layers: [
                Layer {
                    number: 0,
                    subtitle: Some(String::from("flour")),
                },
                Layer {
                    number: 1,
                    subtitle: Some(String::from("dough")),
                },
                Layer {
                    number: 2,
                    subtitle: Some(String::from("cream")),
                },
            ],
            is_delicious: true,
            range: Range {
                start: 50,
                end: 100,
            },
            some_futile_info: None,
            flattened_range: Range { start: 42, end: 64 },
            field_with_specific_rust_name: "renamed field".to_string(),
            pancake_data: None,
            extra_ice_cream_flavor: Flavor::Strawberry,
        }
    });

    #[test]
    fn c_round_trip() {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let workspace_dir = manifest_dir.parent().unwrap();
        let target_dir = workspace_dir.join("target").join("debug");
        let tmp_dir = tempfile::tempdir().expect("Failed to create temp dir");

        // Are we calling cargo from inside the test binary to make sure the cdylib is built ?
        // yes, watch me.
        let cargo_build = std::process::Command::new("cargo")
            .arg("build")
            .arg("-p")
            .arg("ffi-convert-tests")
            .current_dir(workspace_dir)
            .output()
            .expect("Failed to run cargo build");
        assert!(
            cargo_build.status.success(),
            "cargo build failed: {}",
            String::from_utf8_lossy(&cargo_build.stderr)
        );

        // Generate C header with cbindgen
        let header_path = tmp_dir.path().join("ffi_convert_tests.h");
        let mut config = cbindgen::Config::default();
        config.language = cbindgen::Language::C;
        config.parse = cbindgen::ParseConfig {
            parse_deps: true,
            include: Some(vec!["ffi-convert".to_string()]),
            ..Default::default()
        };
        let bindings = cbindgen::Builder::new()
            .with_crate(manifest_dir.to_str().unwrap())
            .with_config(config)
            .generate()
            .expect("Failed to generate C bindings");
        bindings.write_to_file(&header_path);

        // Compile the C test and link against the cdylib.
        // The cc crate expects cargo build-script env vars, so we provide them.
        let rustc_output = std::process::Command::new("rustc")
            .arg("-vV")
            .output()
            .expect("Failed to run rustc");
        let rustc_info = String::from_utf8_lossy(&rustc_output.stdout);
        let host_target = rustc_info
            .lines()
            .find_map(|l| l.strip_prefix("host: "))
            .expect("Could not determine host target from rustc");
        std::env::set_var("TARGET", host_target);
        std::env::set_var("HOST", host_target);
        std::env::set_var("OPT_LEVEL", "0");

        let test_binary = tmp_dir.path().join("test_round_trip");
        let compiler = cc::Build::new()
            .include(tmp_dir.path())
            .opt_level(0)
            .get_compiler();
        let cc_output = compiler
            .to_command()
            .arg(manifest_dir.join("test_round_trip.c"))
            .arg("-fsanitize=address")
            .arg(format!("-L{}", target_dir.display()))
            .arg("-lffi_convert_tests")
            .arg("-o")
            .arg(&test_binary)
            .output()
            .expect("Failed to run C compiler");
        assert!(
            cc_output.status.success(),
            "C compilation failed: {}",
            String::from_utf8_lossy(&cc_output.stderr)
        );

        // Run the C test
        let run = std::process::Command::new(&test_binary)
            .env("LD_LIBRARY_PATH", &target_dir)
            .output()
            .expect("Failed to run C test");
        assert!(
            run.status.success(),
            "C test failed: {}{}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );

        // Run the ASan canary: a deliberate use-after-free that ASan must catch
        let canary = std::process::Command::new(&test_binary)
            .arg("--asan-canary")
            .env("LD_LIBRARY_PATH", &target_dir)
            .output()
            .expect("Failed to run ASan canary");
        assert!(
            !canary.status.success(),
            "ASan canary should have crashed but didn't — is ASan working?"
        );
        let canary_stderr = String::from_utf8_lossy(&canary.stderr);
        assert!(
            canary_stderr.contains("AddressSanitizer"),
            "ASan canary crashed but not from ASan: {}",
            canary_stderr
        );
    }
}
