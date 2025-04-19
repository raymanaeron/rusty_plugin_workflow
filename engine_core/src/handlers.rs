use axum::{
    extract::{Path, State},
};
use axum::body::Bytes;
use axum::response::IntoResponse;
use http::{Method, HeaderMap, StatusCode, HeaderValue};
use std::sync::Arc;
use crate::PluginRegistry;

use std::ffi::{CString, CStr};
use plugin_core::{ApiRequest, HttpMethod, Resource};

pub async fn dispatch_plugin_api(
    State(registry): State<Arc<PluginRegistry>>,
    Path((plugin_name, resource_path)): Path<(String, String)>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    println!("plugin_name = {}", plugin_name);
    println!("resource_path = {}", resource_path);
    println!("registered plugins: {:?}", registry.all().iter().map(|p| &p.name).collect::<Vec<_>>());

    let Some(binding) = registry.get(&plugin_name) else {
        println!("Plugin '{}' not found!", plugin_name);
        return (StatusCode::NOT_FOUND, "Plugin not found").into_response();
    };

    println!("Dispatching to plugin '{}'", plugin_name);
    println!("get_api_resources() = {:p}", binding.get_api_resources as *const ());

    let method_enum = match method.as_str() {
        "GET" => HttpMethod::Get,
        "POST" => HttpMethod::Post,
        "PUT" => HttpMethod::Put,
        "DELETE" => HttpMethod::Delete,
        _ => return (StatusCode::METHOD_NOT_ALLOWED, "Unsupported method").into_response(),
    };

    // ✅ FFI-safe call to plugin.get_api_resources
    let mut count: usize = 0;
    let ptr = (binding.get_api_resources)(&mut count);
    if ptr.is_null() || count == 0 {
        println!("Plugin '{}' returned no resources", plugin_name);
        return (StatusCode::NOT_FOUND, "No API resources found").into_response();
    }

    let supported = unsafe { std::slice::from_raw_parts(ptr, count) };

    // ✅ Search for matching resource
    let Some(resource) = supported.iter().find(|r| {
        let cstr = unsafe { CStr::from_ptr(r.path) };
        let plugin_path = cstr.to_string_lossy();
        println!("Comparing resource: '{}' == '{}'", plugin_path, resource_path);
        plugin_path == resource_path
    }) else {
        println!("Resource '{}' not found in plugin '{}'", resource_path, plugin_name);
        return (StatusCode::NOT_FOUND, "Resource not found").into_response();
    };

    // ✅ Check method support
    let method_supported = {
        let mut i = 0;
        loop {
            let m = unsafe { *resource.supported_methods.add(i) };
            if m == method_enum {
                break true;
            } else if m == HttpMethod::Get && method_enum != HttpMethod::Get {
                i += 1;
            } else {
                break false;
            }
        }
    };

    if !method_supported {
        return (StatusCode::METHOD_NOT_ALLOWED, "Method not allowed").into_response();
    }

    // Convert headers
    let headers_vec = headers
        .iter()
        .map(|(k, v)| {
            let key = CString::new(k.as_str()).unwrap().into_raw();
            let val = CString::new(v.to_str().unwrap_or("")).unwrap().into_raw();
            plugin_core::ApiHeader { key, value: val }
        })
        .collect::<Vec<_>>();

    let path_cstr = CString::new(resource_path).unwrap();
    let content_type_cstr = CString::new("application/json").unwrap();

    let request = ApiRequest {
        path: path_cstr.as_ptr(),
        method: method_enum,
        headers: headers_vec.as_ptr(),
        header_count: headers_vec.len(),
        content_type: content_type_cstr.as_ptr(),
        query: std::ptr::null(),
        body_ptr: body.as_ptr(),
        body_len: body.len(),
    };

    // ✅ Call plugin handler
    let response_ptr = (binding.handle_request)(&request);
    if response_ptr.is_null() {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Plugin error").into_response();
    }

    let response = unsafe { &*response_ptr };
    let body_slice = unsafe { std::slice::from_raw_parts(response.body_ptr, response.body_len) };
    let content_type = unsafe { CStr::from_ptr(response.content_type) }
        .to_str()
        .unwrap_or("application/octet-stream");

    let headers = response.headers;
    let header_count = response.header_count;
    let mut axum_headers = HeaderMap::new();

    if !headers.is_null() && header_count > 0 {
        let header_slice: &[plugin_core::ApiHeader] = unsafe {
            std::slice::from_raw_parts(headers, header_count)
        };

        for h in header_slice {
            let k = unsafe { CStr::from_ptr(h.key) }.to_str().unwrap_or("");
            let v = unsafe { CStr::from_ptr(h.value) }.to_str().unwrap_or("");
            axum_headers.insert(
                k,
                v.parse().unwrap_or_else(|_| HeaderValue::from_static("")),
            );
        }
    }

    let status = StatusCode::from_u16(response.status).unwrap_or(StatusCode::OK);
    let body = body_slice.to_vec();

    (status, [(axum::http::header::CONTENT_TYPE, content_type)], body).into_response()
}
