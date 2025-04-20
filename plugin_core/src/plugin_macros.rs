#[macro_export]
macro_rules! declare_plugin {
    // 1. UI or minimal plugin (6 args, no workflow support)
    (
        $name:expr,
        $route:expr,
        $run_fn:ident,
        $static_fn:ident,
        $resources_fn:ident,
        $handle_fn:ident,
        $cleanup_fn:ident
    ) => {
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

    // 2. Headless plugin with workflow support (9 args)
    (
        $name:expr,
        $route:expr,
        $run_fn:ident,
        $static_fn:ident,
        $resources_fn:ident,
        $handle_fn:ident,
        $cleanup_fn:ident,
        $run_workflow_fn:ident,
        $on_progress_fn:ident,
        $on_complete_fn:ident
    ) => {
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
                run_workflow: Some($run_workflow_fn as extern "C" fn(*const $crate::ApiRequest) -> *mut $crate::ApiResponse),
                on_progress: Some($on_progress_fn as extern "C" fn() -> *mut $crate::ApiResponse),
                on_complete: Some($on_complete_fn as extern "C" fn() -> *mut $crate::ApiResponse),
            }
        }
    };
}
