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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pancake_round_trip(input: *const CPancake) -> *const CPancake {
    let c_pancake = unsafe { &*input };
    let rust_pancake: Pancake = c_pancake.as_rust().expect("Failed to convert to Rust");
    let c_pancake_roundtrip = CPancake::c_repr_of(rust_pancake).expect("Failed to convert to C");
    Box::into_raw(Box::new(c_pancake_roundtrip))
}

/// # Safety
///
/// `pancake` must be a pointer returned by `pancake_round_trip`. It must not be used after this call.
#[unsafe(no_mangle)]
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

    fn compile_c_test(
        work_dir: &std::path::Path,
        lib_dir: &std::path::Path,
        host: &str,
        sanitizer_flag: Option<&str>,
        compiler_override: Option<&str>,
        output: &std::path::Path,
    ) {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));

        // Generate the C header with cbindgen
        let header_path = work_dir.join("ffi_convert_tests.h");
        let mut config = cbindgen::Config::default();
        config.language = cbindgen::Language::C;
        config.parse = cbindgen::ParseConfig {
            parse_deps: true,
            include: Some(vec!["ffi-convert".to_string()]),
            ..Default::default()
        };
        let bindings = cbindgen::Builder::new()
            .with_crate(manifest_dir)
            .with_config(config)
            .generate()
            .expect("Failed to generate C bindings");
        bindings.write_to_file(&header_path);

        // Compile the C test
        let mut build = cc::Build::new();
        build.include(work_dir).opt_level(0).target(host).host(host);
        if let Some(cc) = compiler_override {
            build.compiler(cc);
        }
        let compiler = build.get_compiler();
        let mut cmd = compiler.to_command();
        cmd.arg(manifest_dir.join("test_round_trip.c"));
        if let Some(flag) = sanitizer_flag {
            cmd.arg(flag);
        }
        let cc_output = cmd
            .arg(format!("-L{}", lib_dir.display()))
            .arg("-lffi_convert_tests")
            .arg("-o")
            .arg(output)
            .output()
            .expect("Failed to run C compiler");
        assert!(
            cc_output.status.success(),
            "C compilation failed: {}",
            String::from_utf8_lossy(&cc_output.stderr)
        );
    }

    fn run_c_test(binary: &std::path::Path, lib_dir: &std::path::Path) {
        let run = std::process::Command::new(binary)
            .env("LD_LIBRARY_PATH", lib_dir)
            .output()
            .expect("Failed to run C test");
        assert!(
            run.status.success(),
            "C test failed: {}{}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
    }

    #[cfg(any(feature = "asan", feature = "msan"))]
    const SANITIZER_EXIT_CODE: i32 = 42;

    #[cfg(any(feature = "asan", feature = "msan"))]
    fn run_sanitizer_canary(
        binary: &std::path::Path,
        lib_dir: &std::path::Path,
        canary_flag: &str,
        sanitizer_options_env: &str,
    ) {
        let canary = std::process::Command::new(binary)
            .arg(canary_flag)
            .env("LD_LIBRARY_PATH", lib_dir)
            .env(
                sanitizer_options_env,
                format!("exitcode={SANITIZER_EXIT_CODE}"),
            )
            .output()
            .expect("Failed to run sanitizer canary");
        // The canary binary returns 0 if it ran to completion (sanitizer didn't
        // fire). We expect the sanitizer to kill it with our configured exit code.
        assert_eq!(
            canary.status.code(),
            Some(SANITIZER_EXIT_CODE),
            "Sanitizer canary ({}) exited with unexpected code (expected {}):\nstdout: {}\nstderr: {}",
            canary_flag,
            SANITIZER_EXIT_CODE,
            String::from_utf8_lossy(&canary.stdout),
            String::from_utf8_lossy(&canary.stderr),
        );
    }

    fn work_dir(name: &str) -> std::path::PathBuf {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let dir = manifest_dir.parent().unwrap().join("target").join(name);
        std::fs::create_dir_all(&dir).expect("Failed to create work dir");
        dir
    }

    /// Build the cdylib, optionally with a sanitizer (requires nightly + rust-src).
    /// Cargo artifacts go in `work_dir`. Returns `(lib_dir, host_target)`.
    fn build_cdylib(
        work_dir: &std::path::Path,
        sanitizer: Option<&str>,
    ) -> (std::path::PathBuf, String) {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let workspace_dir = manifest_dir.parent().unwrap();

        let output = std::process::Command::new("rustc")
            .arg("-vV")
            .output()
            .expect("Failed to run rustc");
        let info = String::from_utf8_lossy(&output.stdout);
        let host = info
            .lines()
            .find_map(|l| l.strip_prefix("host: "))
            .expect("Could not determine host target from rustc")
            .to_string();

        // Are we calling cargo from inside the test binary to make sure the cdylib is built ?
        // yes, watch me.
        let mut cmd = std::process::Command::new("cargo");
        cmd.current_dir(workspace_dir)
            .env("CARGO_TARGET_DIR", work_dir);

        let lib_dir = if let Some(san) = sanitizer {
            cmd.args([
                "+nightly",
                "build",
                "-p",
                "ffi-convert-tests",
                "-Zbuild-std",
            ])
            .arg(format!("--target={host}"))
            .env("RUSTFLAGS", format!("-Zsanitizer={san}"));
            work_dir.join(&host).join("debug")
        } else {
            // Clear RUSTFLAGS to avoid inheriting sanitizer flags from the
            // parent process
            cmd.args(["build", "-p", "ffi-convert-tests"])
                .env("RUSTFLAGS", "");
            work_dir.join("debug")
        };

        let output = cmd.output().expect("Failed to run cargo build");
        assert!(
            output.status.success(),
            "cargo build failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        (lib_dir, host)
    }

    #[test]
    /// Plain C round-trip test without sanitizers (works on stable).
    fn c_round_trip() {
        let work_dir = work_dir("c-test");
        let (lib_dir, host) = build_cdylib(&work_dir, None);

        let test_binary = work_dir.join("test_round_trip");
        compile_c_test(&work_dir, &lib_dir, &host, None, None, &test_binary);
        run_c_test(&test_binary, &lib_dir);
    }

    #[test]
    #[cfg(feature = "asan")]
    /// C round-trip test with AddressSanitizer on both Rust and C (requires nightly + rust-src).
    fn c_round_trip_asan() {
        let work_dir = work_dir("c-test-asan");
        let (lib_dir, host) = build_cdylib(&work_dir, Some("address"));

        let test_binary = work_dir.join("test_round_trip");
        compile_c_test(
            &work_dir,
            &lib_dir,
            &host,
            Some("-fsanitize=address"),
            None,
            &test_binary,
        );
        run_c_test(&test_binary, &lib_dir);
        run_sanitizer_canary(&test_binary, &lib_dir, "--asan-canary", "ASAN_OPTIONS");
    }

    #[test]
    #[cfg(feature = "msan")]
    /// C round-trip test with MemorySanitizer on both Rust and C (requires nightly + rust-src).
    fn c_round_trip_msan() {
        let work_dir = work_dir("c-test-msan");
        let (lib_dir, host) = build_cdylib(&work_dir, Some("memory"));

        let test_binary = work_dir.join("test_round_trip_msan");
        // MSan requires clang — gcc doesn't support it
        compile_c_test(
            &work_dir,
            &lib_dir,
            &host,
            Some("-fsanitize=memory"),
            Some("clang"),
            &test_binary,
        );
        run_c_test(&test_binary, &lib_dir);
        run_sanitizer_canary(&test_binary, &lib_dir, "--msan-canary", "MSAN_OPTIONS");
    }
}
