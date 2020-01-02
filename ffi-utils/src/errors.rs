/// A macro used to bootstrap error handling on an FFI project.
///
/// Errors in C exported functions  are handled by returning a "Result" C enum that has two values
/// (OK and KO). If such a function returns KO, then the next call into the FFI lib from this thread
/// should be to the error retrieval function in order to get a string description of the error.
///
/// This macro creates the error retrieval function and optionally the C error enum and a simple
/// macro to convert a rust Result into the C one (and properly setting the thread local value)
///
/// Main usage is
///
/// ```
/// # fn main() {}
/// # #[macro_use] extern crate ffi_utils;
/// generate_error_handling!(get_error_symbol_name, drop_error_symbol_name);
/// ```
///
/// This will only generate the error handling function that that should be used along with the
/// SNIPS_RESULT enum bundled with this lib and the `wrap!` macro also in this lib
///
/// You can also use the full fledged version generating also the error type and the wrap macro
///
/// ```
/// # fn main() {}
/// # #[macro_use] extern crate ffi_utils;
/// generate_error_handling!(my_get_error_symbol,
///                          my_drop_error_symbol, // used to drop the error messages
///                          MY_RESULT_TYPE,       // error type name
///                          MY_RESULT_TYPE_OK,    // OK variant name
///                          MY_RESULT_TYPE_KO,    // KO variant name
///                          "MY_ERROR_STDERR",    // env var name checked by the wrap macro, if set
///                                                // the error will be printed to stderr
///                          my_wrap               // name of the wrap macro to generate
/// );
///
/// ```
#[macro_export]
macro_rules! generate_error_handling {
    ($get_error_symbol:ident) => {
        $crate::generate_error_handling!($get_error_symbol, [ drop_, $get_error_symbol ],  $crate::SNIPS_RESULT, SNIPS_RESULT_OK, SNIPS_RESULT_KO, $crate::wrap);
    };

    ($get_error_symbol:ident, $drop_error_symbol:ident) => {
        $crate::generate_error_handling!($get_error_symbol, [ $drop_error_symbol ],  $crate::SNIPS_RESULT, SNIPS_RESULT_OK, SNIPS_RESULT_KO, $crate::wrap);
    };

    ($get_error_symbol:ident, [ $($drop_error_symbol:ident),* ], $error_type:ty, $error_ok:ident, $error_ko:ident, $wrap:path) => {
        use std::cell::RefCell;
        thread_local! {
            pub(crate) static LAST_ERROR: RefCell<Option<String>> = RefCell::new(None);
        }

        $crate::document_multiline! {
            " Used to retrieve the last error that happened in this thread. A function encountered an",
            concat!(" error if its return type is of type `",
                    stringify!($error_type),
                    "` and it returned `",
                    stringify!($error_ko),
                    "`"),
            " # Arguments",
            "  - `error`: pointer to a string that will contain the error description, this should",
            concat!(" then be destroyed properly using the `",
                    $(stringify!( $drop_error_symbol ),)*
                    "` function in this lib to prevent leaks"),
            "",
            " # Return type",
            concat!(" Should return `", stringify!($error_ok), "`."),
            "",
            concat!(" If `", stringify!($error_ko), "` is returned, then something very wrong happened in the lib.")

                     =>

            #[no_mangle]
            pub extern "C" fn $get_error_symbol(
                error: *mut *mut ::libc::c_char,
            ) -> $error_type {

                fn _get_last_error(
                    error: *mut *mut libc::c_char,
                ) -> std::result::Result<(), ::failure::Error> {
                    LAST_ERROR.with(|msg| {
                        let string = msg
                            .borrow_mut()
                            .take()
                            .unwrap_or_else(|| "No error message".to_string());
                        $crate::point_to_string_mut(error, string)
                    })
                }

                $wrap!(_get_last_error(error))
            }
        }

        $crate::paste::item! {
            $crate::document_multiline! {
                concat!(" Used to destroy an error string created using the `",
                        stringify!($get_error_symbol),
                        "` function."),
                " # Arguments",
                "  - `ptr`: pointer to th string to destroy",

                " # Return type",
                concat!(" Returns `", stringify!($error_ok), "` if the string was destroyed properly."),
                "",
                concat!(" If `", stringify!($error_ko), "` is returned, you can get more information on the "),
                concat!(" error using the `", stringify!($get_error_symbol), "`function.")

                         =>

                #[no_mangle]
                pub extern "C" fn [< $($drop_error_symbol)* >] (
                    error: *mut ::libc::c_char,
                ) -> $error_type {

                    fn _destroy(error: *mut ::libc::c_char) -> Result<(), failure::Error> {
                        $crate::take_back_c_string!(error);
                        Ok(())
                    }

                    $wrap!(_destroy(error))
                }
            }
        }

    };

    ($get_error_symbol:ident, $drop_error_symbol:ident, $error_type_name:ident, $error_ok:ident, $error_ko:ident, $error_stderr_envvar:expr, $wrap_name:ident) => {

        $crate::document_multiline! {
            " Used as a return type of functions that can encounter errors.",
            concat!(" If the function encountered an error, you can retrieve it using the `",
                    stringify!($get_error_symbol),
                    "` function") =>
            #[repr(C)]
            #[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
            #[allow(non_camel_case_types)]
            pub enum $error_type_name {
                /// The function returned successfully
                $error_ok = 0,

                /// The function returned an error
                $error_ko = 1,
            }

        }

        $crate::generate_wrap!($wrap_name, $error_type_name, $error_ok, $error_ko, $error_stderr_envvar, $crate::ErrorExt);

        $crate::generate_error_handling!($get_error_symbol, [ $drop_error_symbol ] , $error_type_name, $error_ok, $error_ko, $wrap_name);

    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! generate_wrap {
    ($wrap_name:ident, $error_type_name:ident, $error_ok:ident, $error_ko:ident, $error_stderr_envvar:expr, $error_ext:path) => {
        macro_rules! $wrap_name {
            ($e:expr) => {
                match $e {
                    Ok(_) => $error_type_name::$error_ok,
                    Err(e) => {
                        use $error_ext;
                        let msg = e.pretty().to_string();
                        if std::env::var($error_stderr_envvar).is_ok() {
                            eprintln!("{}", msg);
                        }
                        LAST_ERROR.with(|p| *p.borrow_mut() = Some(msg));
                        $error_type_name::$error_ko
                    }
                }
            };
        }
    };
}

#[macro_export]
macro_rules! wrap {
    ($e:expr) => {
        match $e {
            Ok(_) => $crate::SNIPS_RESULT::SNIPS_RESULT_OK,
            Err(e) => {
                use $crate::ErrorExt;
                let msg = e.pretty().to_string();
                if std::env::var("SNIPS_ERROR_STDERR").is_ok() {
                    eprintln!("{}", msg);
                }
                LAST_ERROR.with(|p| *p.borrow_mut() = Some(msg));
                $crate::SNIPS_RESULT::SNIPS_RESULT_KO
            }
        }
    };
}

/// Same principle as the doc_comments crate macro, except with support for proper multiline
#[macro_export]
#[doc(hidden)]
macro_rules! document_multiline {
    ($($doc:expr),* => $($tt:tt)*) => {
        $(#[doc = $doc])*
        $($tt)*
    };
}

#[cfg(test)]
mod tests {
    generate_error_handling!(get_last_error);

    fn foo(input: Result<(), failure::Error>) -> crate::SNIPS_RESULT {
        fn foo_(input: Result<(), failure::Error>) -> Result<(), failure::Error> {
            input
        }

        wrap!(foo_(input))
    }

    #[test]
    fn wrapping_ok_works() {
        assert_eq!(foo(Ok(())), crate::SNIPS_RESULT::SNIPS_RESULT_OK)
    }

    #[test]
    fn wrapping_ko_works() {
        assert_eq!(foo(Err(failure::format_err!("wat?"))), crate::SNIPS_RESULT::SNIPS_RESULT_KO);
        let mut ptr = std::ptr::null_mut();
        get_last_error(&mut ptr);

        assert_eq!(unsafe { std::ffi::CStr::from_ptr(ptr) }.to_str().unwrap(), "wat?\n");

        assert_eq!(drop_get_last_error(ptr), crate::SNIPS_RESULT::SNIPS_RESULT_OK);
    }
}


#[cfg(test)]
mod tests2 {
    generate_error_handling!(get_last_error2, drop_error2, MY_RESULT_TYPE, MY_RESULT_TYPE_OK, MY_RESULT_TYPE_KO, "MY_ERROR_STDERR", mywrap);


    fn foo(input: Result<(), failure::Error>) -> MY_RESULT_TYPE {
        fn foo_(input: Result<(), failure::Error>) -> Result<(), failure::Error> {
            input
        }

        mywrap!(foo_(input))
    }

    #[test]
    fn wrapping_ok_works() {
        assert_eq!(foo(Ok(())), MY_RESULT_TYPE::MY_RESULT_TYPE_OK)
    }

    #[test]
    fn wrapping_ko_works() {
        assert_eq!(foo(Err(failure::format_err!("wat?"))), MY_RESULT_TYPE::MY_RESULT_TYPE_KO);
        let mut ptr = std::ptr::null_mut();
        get_last_error2(&mut ptr);

        assert_eq!(unsafe { std::ffi::CStr::from_ptr(ptr) }.to_str().unwrap(), "wat?\n");

        assert_eq!(drop_error2(ptr), MY_RESULT_TYPE::MY_RESULT_TYPE_OK);
    }
}
