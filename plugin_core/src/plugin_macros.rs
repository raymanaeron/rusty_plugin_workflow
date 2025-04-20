#[macro_export]
macro_rules! declare_plugin {
    ( $name:expr, $route:expr, $run_fn:ident, $static_fn:ident, $resources_fn:ident, $handle_fn:ident, $cleanup_fn:ident ) => {
        #[no_mangle]
        pub extern "C" fn name() -> *const ::std::os::raw::c_char {
            ::std::ffi::CString::new($name).unwrap().into_raw()
        }

        #[no_mangle]
        pub extern "C" fn plugin_route() -> *const ::std::os::raw::c_char {
            ::std::ffi::CString::new($route).unwrap().into_raw()
        }

        #[no_mangle]
        pub extern "C" fn create_plugin() -> *const $crate::Plugin {
            &$crate::Plugin {
                name,
                plugin_route,
                run: $run_fn,
                get_static_content_path: $static_fn,
                get_api_resources: $resources_fn,
                handle_request: $handle_fn,
                cleanup: $cleanup_fn,
                run_workflow: None,
                on_progress: None,
                on_complete: None,
            }
        }
    };
}
