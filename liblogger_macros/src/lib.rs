/*
 * Procedural macros for enhanced logging capabilities
 *
 * This module provides procedural macros that can be applied to functions
 * for various logging, monitoring, and instrumentation purposes.
 */

extern crate proc_macro;

// Import our utils module
mod macro_utils;

use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, parse_quote, ItemFn};

// Import helpers from our utils module
use crate::macro_utils::{get_fn_name, IdList, MacroArgs, define_helper_functions};

/// Helper function to extract error from Result and other utility functions
#[proc_macro]
pub fn initialize_logger_attributes(_input: TokenStream) -> TokenStream {
    TokenStream::from(define_helper_functions())
}

/// Log entry and exit of a function
#[proc_macro_attribute]
pub fn log_entry_exit(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        liblogger::log_info!(&format!("ENTRY: {}", #fn_name));
        
        let result = (|| #orig_block)();
        
        liblogger::log_info!(&format!("EXIT: {}", #fn_name));
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Log errors and panics
#[proc_macro_attribute]
pub fn log_errors(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        use std::panic::{catch_unwind, AssertUnwindSafe};
        
        let result = catch_unwind(AssertUnwindSafe(|| #orig_block));
        
        match result {
            Ok(output) => {
                // Check if output is a Result and has an Err variant
                if let Some(err) = extract_error(&output) {
                    liblogger::log_error!(&format!("{} returned error: {:?}", #fn_name, err), None);
                }
                output
            },
            Err(panic_err) => {
                let panic_msg = if let Some(s) = panic_err.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic_err.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown panic".to_string()
                };
                
                liblogger::log_error!(&format!("{} panicked: {}", #fn_name, panic_msg), None);
                std::panic::resume_unwind(panic_err);
            }
        }
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Measure execution time of a function
#[proc_macro_attribute]
pub fn measure_time(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        use std::time::Instant;
        use std::panic::{catch_unwind, AssertUnwindSafe};
        
        let start_time = Instant::now();
        
        let result = catch_unwind(AssertUnwindSafe(|| #orig_block));
        
        let duration = start_time.elapsed();
        let duration_ms = duration.as_millis();
        
        match result {
            Ok(output) => {
                liblogger::log_info!(&format!("{} completed in {} ms ", #fn_name, duration_ms), None);
                output
            },
            Err(panic_err) => {
                liblogger::log_error!(
                    &format!("{} panicked after {} ms ", #fn_name, duration_ms), 
                    None
                );
                std::panic::resume_unwind(panic_err);
            }
        }
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Log specified function arguments
#[proc_macro_attribute]
pub fn log_args(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as IdList);
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let arg_names = args.ids;
    let mut log_stmts = Vec::new();
    
    for arg_name in &arg_names {
        let arg_str = arg_name.to_string();
        log_stmts.push(quote! {
            let arg_value = format!("{:?}", #arg_name);
            args_str.push_str(&format!("{} = {}, ", #arg_str, arg_value));
        });
    }
    
    input_fn.block = Box::new(parse_quote!({
        use std::time::Instant;
        let start_time = Instant::now();
        let mut args_str = String::new();
        #(#log_stmts)*
        // Remove trailing comma and space
        if !args_str.is_empty() {
            args_str.truncate(args_str.len() - 2);
        }
        liblogger::log_info!(&format!("Entering {} with args: {}", #fn_name, args_str), None);
        #orig_block
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Log and implement retry logic
#[proc_macro_attribute]
pub fn log_retries(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let max_attempts = args.max_attempts.unwrap_or(3);
    
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        let mut attempts = 0u32;
        loop {
            attempts += 1;
            if attempts > 1 {
                liblogger::log_info!(
                    &format!("Retry attempt {} of {} for {}", attempts, #max_attempts, #fn_name), 
                    None
                );
                // Simple exponential backoff
                std::thread::sleep(std::time::Duration::from_millis((2u64.pow(attempts - 1) * 50) as u64));
            }
            
            let result = (|| #orig_block)();
            
            if is_success(&result) || attempts >= #max_attempts {
                if attempts >= #max_attempts && !is_success(&result) {
                    liblogger::log_error!(
                        &format!("{} failed after {} attempts ", #fn_name, attempts), 
                        None
                    );
                } else if attempts > 1 {
                    liblogger::log_info!(
                        &format!("{} succeeded after {} attempts ", #fn_name, attempts), 
                        None
                    );
                }
                return result;
            }
            
            if let Some(err) = extract_error(&result) {
                liblogger::log_warn!(
                    &format!("{} attempt {} failed: {:?}", #fn_name, attempts, err), 
                    None
                );
            }
        }
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Create audit logs for security-critical operations
#[proc_macro_attribute]
pub fn audit_log(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let category = args.category.unwrap_or_else(|| "general".to_string());
    
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        liblogger::log_info!(
            &format!("AUDIT: [{}] Operation {} started", #category, #fn_name), 
            None
        );
        let result = #orig_block;
        liblogger::log_info!(
            &format!("AUDIT: [{}] Operation {} completed", #category, #fn_name), 
            Some(format!("result_type={}", if is_success(&result) { "success" } else { "failure" }))
        );
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Implement a circuit breaker patter
#[proc_macro_attribute]
pub fn circuit_breaker(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let failure_threshold = args.failure_threshold.unwrap_or(5);
        
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let counter_var = format_ident!("CB_FAILURES_{}", fn_name.to_uppercase());
    let circuit_open_var = format_ident!("CB_OPEN_{}", fn_name.to_uppercase());
    let last_failure_var = format_ident!("CB_LAST_FAILURE_{}", fn_name.to_uppercase());
    
    input_fn.block = Box::new(parse_quote!({
        use std::sync::atomic::{AtomicU32, AtomicBool, AtomicI64, Ordering};
        use std::time::{SystemTime, UNIX_EPOCH};
        // Create static counters if they don't exist
        static #counter_var: AtomicU32 = AtomicU32::new(0);
        static #circuit_open_var: AtomicBool = AtomicBool::new(false);
        static #last_failure_var: AtomicI64 = AtomicI64::new(0);
        
        // Check if circuit is open
        if #circuit_open_var.load(Ordering::SeqCst) {
            // Check if we should try to reset (after 30 seconds)
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
            let last_failure = #last_failure_var.load(Ordering::SeqCst);
            
            if now - last_failure >= 30 {
                // Allow one request through to test if the system has recovered
                #circuit_open_var.store(false, Ordering::SeqCst);
                #counter_var.store(0, Ordering::SeqCst);
                liblogger::log_info!(&format!("Circuit breaker for {} is attempting to reset", #fn_name), None);
            } else {
                liblogger::log_warn!(&format!("Circuit breaker for {} is open, fast-failing request", #fn_name), None);
                return Err(format!("Service {} is unavailable (circuit open)", #fn_name).into());
            }
        }
        
        // Execute the function
        let result = #orig_block;
        
        // Update circuit breaker state
        if is_success(&result) {
            if #counter_var.load(Ordering::SeqCst) > 0 {
                #counter_var.store(0, Ordering::SeqCst);
                liblogger::log_info!(&format!("Circuit breaker for {} reset after success", #fn_name), None);
            }
        } else {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
            #last_failure_var.store(now, Ordering::SeqCst);
            let failures = #counter_var.fetch_add(1, Ordering::SeqCst) + 1;
            if failures >= #failure_threshold && !#circuit_open_var.load(Ordering::SeqCst) {
                #circuit_open_var.store(true, Ordering::SeqCst);
                liblogger::log_error!(
                    &format!("Circuit breaker for {} opened after {} consecutive failures", 
                        #fn_name, failures), 
                    None
                );
            }
        }
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Throttle logs to avoid flooding during incidents
#[proc_macro_attribute]
pub fn throttle_log(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let rate = args.rate.unwrap_or(10);
    
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let counter_var = format_ident!("LOG_THROTTLE_{}", fn_name.to_uppercase());
    let minute_var = format_ident!("LOG_THROTTLE_MINUTE_{}", fn_name.to_uppercase());
    let skipped_var = format_ident!("LOG_THROTTLE_SKIPPED_{}", fn_name.to_uppercase());
    
    input_fn.block = Box::new(parse_quote!({
        use std::sync::atomic::{AtomicU32, AtomicI64, Ordering};
        use std::time::{SystemTime, UNIX_EPOCH};
        // Create static counters if they don't exist
        static #counter_var: AtomicU32 = AtomicU32::new(0);
        static #minute_var: AtomicI64 = AtomicI64::new(0);
        static #skipped_var: AtomicU32 = AtomicU32::new(0);
        
        // Check throttle limits
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() / 60;
        let current_minute = #minute_var.load(Ordering::SeqCst);
        
        let should_log = if current_minute != now as i64 {
            // New minute, reset counters
            #minute_var.store(now as i64, Ordering::SeqCst);
            let skipped = #skipped_var.swap(0, Ordering::SeqCst);
            if skipped > 0 {
                liblogger::log_info!(
                    &format!("Throttled logs for {}: skipped {} logs in previous minute", 
                        #fn_name, skipped),
                    None
                );
            }
            #counter_var.store(1, Ordering::SeqCst);
            true
        } else {
            // Same minute, check counter
            let count = #counter_var.fetch_add(1, Ordering::SeqCst) + 1;
            if count <= #rate {
                true
            } else {
                #skipped_var.fetch_add(1, Ordering::SeqCst);
                false
            }
        };
        
        let result = #orig_block;
        
        // Only log if within rate limits
        if should_log {
            if is_success(&result) {
                liblogger::log_info!(&format!("{} succeeded", #fn_name), None);
            } else if let Some(err) = extract_error(&result) {
                liblogger::log_error!(&format!("{} failed: {:?}", #fn_name, err), None);
            }
        }
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Measure latency to external dependencies
#[proc_macro_attribute]
pub fn dependency_latency(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let target = args.target.unwrap_or_else(|| "unknown".to_string());
    
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        use std::time::Instant;
        liblogger::log_info!(
            &format!("Dependency call to {} started for {}", #target, #fn_name),
            None
        );
        let start_time = Instant::now();
        let result = #orig_block;
        let duration_ms = start_time.elapsed().as_millis();
        
        // Only log if within rate limits
        if is_success(&result) {
            liblogger::log_info!(&format!("Dependency call to {} completed in {} ms", #target, duration_ms), None);
        } else if let Some(err) = extract_error(&result) {
            liblogger::log_error!(&format!("Dependency call to {} failed after {} ms: {:?}", 
                #target, duration_ms, err), None);
        }
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Log the returned value from a function
#[proc_macro_attribute]
pub fn log_response(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        let result = #orig_block;
        if is_success(&result) {
            liblogger::log_debug!(&format!("{} returned: {:?}", #fn_name, result), None);
        }
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Track concurrent invocations of a function
#[proc_macro_attribute]
pub fn log_concurrency(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let counter_var = format_ident!("CONCURRENCY_{}", fn_name.to_uppercase());
    
    input_fn.block = Box::new(parse_quote!({
        use std::sync::atomic::{AtomicU32, Ordering};
        static #counter_var: AtomicU32 = AtomicU32::new(0);
        
        let current = #counter_var.fetch_add(1, Ordering::SeqCst) + 1;
        liblogger::log_debug!(
            &format!("{} concurrent invocations: {}", #fn_name, current),
            None
        );
        
        let result = #orig_block;
        
        let after = #counter_var.fetch_sub(1, Ordering::SeqCst) - 1;
        liblogger::log_debug!(
            &format!("{} concurrent invocations after exit: {}", #fn_name, after),
            None
        );
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Create and propagate a trace ID for request flow tracking
#[proc_macro_attribute]
pub fn trace_span(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        use uuid::Uuid;
        // Generate or reuse trace ID
        let trace_id = if let Some(existing_id) = get_trace_id() {
            existing_id
        } else {
            let new_id = Uuid::new_v4().to_string();
            set_trace_id(&new_id);
            new_id
        };
        
        liblogger::log_info!(
            &format!("[TraceID: {}] {} started", trace_id, #fn_name),
            None
        );
        
        let result = #orig_block;
        
        liblogger::log_info!(
            &format!("[TraceID: {}] {} completed", trace_id, #fn_name),
            None
        );
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Log feature flag state
#[proc_macro_attribute]
pub fn feature_flag(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let flag_name = args.flag_name.unwrap_or_else(|| "unknown".to_string());
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        // Check feature flag (placeholder function)
        let is_enabled = is_feature_enabled(#flag_name);
        
        liblogger::log_info!(
            &format!("{} called with feature flag {} = {}", 
                #fn_name, #flag_name, is_enabled),
            None
        );
        
        let result = #orig_block;
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Increment a metrics counter for function calls
#[proc_macro_attribute]
pub fn metrics_counter(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let counter_name = args.counter_name.unwrap_or_else(|| "function_calls".to_string());
        
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        // Increment counter using Prometheus if available
        #[cfg(feature = "prometheus")]
        {
            use prometheus::{Counter, register_counter};
            use std::sync::Once;
            static INIT: Once = Once::new();
            static mut COUNTER: Option<Counter> = None;
            
            INIT.call_once(|| {
                let counter = register_counter!(#counter_name, "Function call counter").unwrap();
                unsafe {
                    COUNTER = Some(counter);
                }
            });
            
            if let Some(counter) = unsafe { COUNTER.as_ref() } {
                counter.inc();
            }
        }
        
        let result = #orig_block;
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Log memory usage during function execution
#[proc_macro_attribute]
pub fn log_memory_usage(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        #[cfg(feature = "memory_usage")]
        let (start_rss, start_vms) = {
            use psutil::process::Process;
            let process = Process::current().unwrap();
            let memory = process.memory_info().unwrap();
            (memory.rss(), memory.vms())
        };
        
        let result = #orig_block;
        
        #[cfg(feature = "memory_usage")]
        {
            use psutil::process::Process;
            let process = Process::current().unwrap();
            let memory = process.memory_info().unwrap();
            let end_rss = memory.rss();
            let end_vms = memory.vms();
            
            liblogger::log_info!(
                &format!("{} starting memory usage - RSS: {} bytes, VMS: {} bytes", 
                    #fn_name, start_rss, start_vms),
                None
            );
            liblogger::log_info!(
                &format!("{} ending memory usage - RSS: {} bytes (delta: {} bytes), VMS: {} bytes (delta: {} bytes)", 
                    #fn_name, end_rss, end_rss as i64 - start_rss as i64, 
                    end_vms, end_vms as i64 - start_vms as i64),
                None
            );
        }
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Log CPU time used during function execution
#[proc_macro_attribute]
pub fn log_cpu_time(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        use std::time::Instant;
        let wall_time_start = Instant::now();
        
        // There's no direct CPU time measurement in standard Rust
        // This is just a placeholder that measures wall time
        let result = #orig_block;
        let wall_time = wall_time_start.elapsed();
        
        liblogger::log_info!(
            &format!("{} used CPU time: approx {} ms (wall time)", 
                #fn_name, wall_time.as_millis()),
            None
        );
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Include version information in logs
#[proc_macro_attribute]
pub fn version_tag(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        let version = std::env::var("BUILD_VERSION").unwrap_or_else(|_| "unknown".to_string());
        liblogger::log_info!(
            &format!("[Version: {}] {} called", version, #fn_name),
            None
        );
        
        let result = #orig_block;
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Attach request context to logs
#[proc_macro_attribute]
pub fn request_context(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        // Get context from thread-local storage (placeholder)
        let user_id = get_thread_local_value("user_id");
        let session_id = get_thread_local_value("session_id");
        let request_id = get_thread_local_value("request_id");
        
        let mut context_parts = Vec::new();
        if let Some(id) = user_id {
            context_parts.push(format!("user_id={}", id));
        }
        if let Some(id) = session_id {
            context_parts.push(format!("session_id={}", id));
        }
        if let Some(id) = request_id {
            context_parts.push(format!("request_id={}", id));
        }
        
        let context_str = if !context_parts.is_empty() {
            context_parts.join(", ")
        } else {
            "No context available".to_string()
        };
        
        liblogger::log_info!(
            &format!("{} called", #fn_name),
            Some(context_str)
        );
        
        let result = #orig_block;
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Catch and log panics but don't crash
#[proc_macro_attribute]
pub fn catch_panic(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    // Determine return type
    let returns_result = if let syn::ReturnType::Type(_, ty) = &input_fn.sig.output {
        if let syn::Type::Path(type_path) = ty.as_ref() {
            let last_segment = type_path.path.segments.last().unwrap();
            last_segment.ident.to_string() == "Result"
        } else {
            false
        }
    } else {
        false
    };
    
    input_fn.block = if returns_result {
        Box::new(parse_quote!({
            use std::panic::{catch_unwind, AssertUnwindSafe};
            
            match catch_unwind(AssertUnwindSafe(|| #orig_block)) {
                Ok(result) => result,
                Err(panic_err) => {
                    let panic_msg = if let Some(s) = panic_err.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = panic_err.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "Unknown panic ".to_string()
                    };
                    
                    liblogger::log_error!(&format!("{} caught panic: {}", #fn_name, panic_msg), None);
                    Err(format!("Panic in {}: {}", #fn_name, panic_msg).into())
                }
            }
        }))
    } else {
        Box::new(parse_quote!({
            use std::panic::{catch_unwind, AssertUnwindSafe};
            
            match catch_unwind(AssertUnwindSafe(|| #orig_block)) {
                Ok(result) => result,
                Err(panic_err) => {
                    let panic_msg = if let Some(s) = panic_err.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = panic_err.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "Unknown panic ".to_string()
                    };
                    
                    liblogger::log_error!(&format!("{} caught panic: {}", #fn_name, panic_msg), None);
                    // Return default value as fallback
                    Default::default()
                }
            }
        }))
    };
    
    TokenStream::from(quote!(#input_fn))
}

/// Log health check results
#[proc_macro_attribute]
pub fn health_check(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        use std::time::Instant;
        
        let start_time = Instant::now();
        let result = #orig_block;
        let duration = start_time.elapsed();
        
        if is_success(&result) {
            liblogger::log_info!(
                &format!("Health check {} passed in {} ms ", #fn_name, duration.as_millis()),
                None
            );
        } else if let Some(err) = extract_error(&result) {
            liblogger::log_info!(
                &format!("Health check {} failed in {} ms: {:?}", 
                    #fn_name, duration.as_millis(), err),
                None
            );
        }
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Log function result with different levels for success/error
#[proc_macro_attribute] 
pub fn log_result(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let success_level = args.success_level.unwrap_or_else(|| "info".to_string());
    let error_level = args.error_level.unwrap_or_else(|| "error".to_string());
    
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    // Create string literals for the different log levels to avoid str_as_str
    let success_level_str = success_level.clone();
    let error_level_str = error_level.clone();
    
    input_fn.block = Box::new(parse_quote!({
        let result = #orig_block;
        
        if is_success(&result) {
            // Replace the match with if-else to avoid str_as_str
            let level = #success_level_str;
            if level == "debug" {
                liblogger::log_debug!(&format!("{} succeeded with result: {:?}", #fn_name, result), None);
            } else if level == "warn" {
                liblogger::log_warn!(&format!("{} succeeded with result: {:?}", #fn_name, result), None);
            } else if level == "error" {
                liblogger::log_error!(&format!("{} succeeded with result: {:?}", #fn_name, result), None);
            } else {
                liblogger::log_info!(&format!("{} succeeded with result: {:?}", #fn_name, result), None);
            }
        } else if let Some(err) = extract_error(&result) {
            // Replace the match with if-else to avoid str_as_str
            let level = #error_level_str;
            if level == "debug" {
                liblogger::log_debug!(&format!("{} failed with error: {:?}", #fn_name, err), None);
            } else if level == "info" {
                liblogger::log_info!(&format!("{} failed with error: {:?}", #fn_name, err), None);
            } else if level == "warn" {
                liblogger::log_warn!(&format!("{} failed with error: {:?}", #fn_name, err), None);
            } else {
                liblogger::log_error!(&format!("{} failed with error: {:?}", #fn_name, err), None);
            }
        }
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}
