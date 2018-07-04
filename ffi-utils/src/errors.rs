#[macro_export]
macro_rules! generate_error_handling {
    ($get_error_symbol:ident) => {
        use std::cell::RefCell;
        thread_local! {
            static LAST_ERROR: RefCell<Option<String>> = RefCell::new(None);
        }

        fn get_last_error(error: *mut *const libc::c_char) -> ::std::result::Result<(), ::failure::Error> {
            LAST_ERROR.with(|msg| {
                let string = msg.borrow_mut().take().unwrap_or_else(||
                    "No error message".to_string()
                );
                $crate::point_to_string(error, string)
            })
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
                LAST_ERROR.with(|p| { *p.borrow_mut() = Some(msg) } );
                $crate::SNIPS_RESULT::SNIPS_RESULT_KO
            }
        }
    };
}
