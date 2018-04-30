#[macro_export]
macro_rules! generate_error_handling {
    ($get_error_symbol:ident) => {
        lazy_static! {
            static ref LAST_ERROR: ::std::sync::Mutex<String> = ::std::sync::Mutex::new("".to_string());
        }

        fn get_last_error(error: *mut *const libc::c_char) -> ::std::result::Result<(), ::failure::Error> {
            $crate::point_to_string(error, LAST_ERROR.lock().map_err(|_| format_err!("poison lock"))?.clone())
        }

        #[no_mangle]
        pub extern "C" fn $get_error_symbol(error: *mut *const ::libc::c_char) -> $crate::SNIPS_RESULT {
            wrap!(get_last_error(error))
        }
    }
}

#[macro_export]
macro_rules! wrap {
    ($e:expr) => {
        match $e {
            Ok(_) => $crate::SNIPS_RESULT::SNIPS_RESULT_OK,
            Err(e) => {
                use $crate::ErrorExt;
                let msg = e.pretty().to_string();
                eprintln!("{}", msg);
                match LAST_ERROR.lock() {
                    Ok(mut guard) => *guard = msg,
                    Err(_) => (), /* curl up and cry */
                }
                $crate::SNIPS_RESULT::SNIPS_RESULT_KO
            }
        }
    };
}
