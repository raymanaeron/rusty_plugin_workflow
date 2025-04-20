use std::ffi::CString;
use crate::{HttpMethod, Resource};
use std::sync::Once;

pub fn static_resource(path: &str, methods: &'static [HttpMethod]) -> &'static [Resource] {
    static mut STATIC_SLICE: Option<&'static [Resource]> = None;
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let c_path = CString::new(path).unwrap();
        let path_ptr = Box::leak(c_path.into_boxed_c_str()).as_ptr();

        let resource = Resource::new(path_ptr, methods.as_ptr());
        let boxed = vec![resource].into_boxed_slice();
        unsafe {
            STATIC_SLICE = Some(Box::leak(boxed));
        }
    });

    unsafe { STATIC_SLICE.unwrap_or_else(|| &[]) }
}