#[macro_export]
macro_rules! generate_error_handling {
    ($get_error_symbol:ident) => {
        use std::cell::RefCell;
        thread_local! {
            static LAST_ERROR: RefCell<Option<String>> = RefCell::new(None);
        }

        fn _get_last_error(
            error: *mut *const libc::c_char,
        ) -> std::result::Result<(), ::failure::Error> {
            LAST_ERROR.with(|msg| {
                let string = msg
                    .borrow_mut()
                    .take()
                    .unwrap_or_else(|| "No error message".to_string());
                $crate::point_to_string(error, string)
            })
        }

        /// Used to retrieve the last error that happened in this thread. A function encountered an
        /// error if its return type is of type SNIPS_RESULT and it returned SNIPS_RESULT_KO
        #[no_mangle]
        pub extern "C" fn $get_error_symbol(
            error: *mut *const ::libc::c_char,
        ) -> $crate::SNIPS_RESULT {
            wrap!(_get_last_error(error))
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

#[cfg(test)]
mod tests {
    #![allow(dead_code)]

    generate_error_handling!(get_last_error);
}
