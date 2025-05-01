/*
 * Procedural macros for enhanced logging capabilities
 *
 * This module provides procedural macros that can be applied to functions
 * for various logging, monitoring, and instrumentation purposes.
 * 
 * These macros work with the liblogger crate to provide automatic context
 * capturing, timing measurements, and other advanced logging features.
 */

 extern crate proc_macro;

 // Import our utils module
 mod macro_utils;
 
 use proc_macro::TokenStream;
 use quote::{quote, format_ident};
 use syn::{parse_macro_input, parse_quote, ItemFn};
 
 // Import helpers from our utils module
 use crate::macro_utils::{get_fn_name, IdList, MacroArgs, define_helper_functions};
 
 /// Initialization macro that must be called at the module level to enable attribute macros
 ///
 /// This macro defines helper functions needed by the attribute macros, such as
 /// error extraction, success checking, trace ID management, and feature flag checking.
 ///
 /// # Example
 /// ```
 /// use liblogger_macros::*;
 ///
 /// // Call at module level (usually at the top of your file)
 /// initialize_logger_attributes!();
 /// ```
 #[proc_macro]
 pub fn initialize_logger_attributes(_input: TokenStream) -> TokenStream {
     TokenStream::from(define_helper_functions())
 }
 
 /// Logs function entry and exit points to track execution flow
 ///
 /// Automatically adds INFO level logs at the start and end of the function.
 /// Useful for tracing code execution paths during debugging and in production.
 ///
 /// # Example
 /// ```
 /// #[log_entry_exit]
 /// fn process_data(user_id: &str) {
 ///     // Function implementation
 /// }
 /// ```
 ///
 /// # Generated logs
 /// - "ENTRY: process_data"
 /// - "EXIT: process_data"
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
             Ok(inner_result) => {
                 // Use pattern matching to handle Result types
                 match &inner_result {
                     Ok(_) => {},  // Success case, no logging needed
                     Err(err) => {
                         // Error case, log the error
                         liblogger::log_error!(&format!("{} returned error: {:?}", #fn_name, err), None);
                     }
                 }
                 inner_result
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
             
             // Use pattern matching to determine success or failure
             match &result {
                 Ok(_) => {
                     // Success case
                     if attempts > 1 {
                         liblogger::log_info!(
                             &format!("{} succeeded after {} attempts", #fn_name, attempts), 
                             None
                         );
                     }
                     return result;
                 },
                 Err(err) => {
                     // Error case
                     if attempts >= #max_attempts {
                         liblogger::log_error!(
                             &format!("{} failed after {} attempts: {:?}", #fn_name, attempts, err), 
                             None
                         );
                         return result;
                     }
                     
                     liblogger::log_warn!(
                         &format!("{} attempt {} failed: {:?}", #fn_name, attempts, err), 
                         None
                     );
                     // Continue to next retry iteration
                 }
             }
         }
     }));
     
     TokenStream::from(quote!(#input_fn))
 }
 
 /// Create detailed audit logs
 #[proc_macro_attribute]
 pub fn audit_log(_args: TokenStream, input: TokenStream) -> TokenStream {
     let mut input_fn = parse_macro_input!(input as ItemFn);
     let fn_name = get_fn_name(&input_fn);
     let orig_block = input_fn.block.clone();
     
     input_fn.block = Box::new(parse_quote!({
         let user_id = get_thread_local_value("user_id").unwrap_or_else(|| "unknown".to_string());
         liblogger::log_info!(&format!("AUDIT: {} called", #fn_name), Some(format!("user_id={}", user_id)));
         
         let start_time = std::time::Instant::now();
         let result = #orig_block;
         let duration = start_time.elapsed();
         
         // Use pattern matching on result
         match &result {
             () => {
                 // Unit return type
                 liblogger::log_info!(
                     &format!("AUDIT: {} completed in {} ms", #fn_name, duration.as_millis()),
                     Some(format!("user_id={}", user_id))
                 );
             },
             _ => {
                 // Any other return type
                 liblogger::log_info!(
                     &format!("AUDIT: {} completed in {} ms with result: {:?}", 
                         #fn_name, duration.as_millis(), result),
                     Some(format!("user_id={}", user_id))
                 );
             }
         }
         
         result
     }));
     
     TokenStream::from(quote!(#input_fn))
 }
 
 /// Circuit breaker pattern with logging
 #[proc_macro_attribute]
 pub fn circuit_breaker(args: TokenStream, input: TokenStream) -> TokenStream {
     let args = parse_macro_input!(args as MacroArgs);
     let threshold = args.failure_threshold.unwrap_or(3);
     
     let mut input_fn = parse_macro_input!(input as ItemFn);
     let fn_name = get_fn_name(&input_fn);
     let orig_block = input_fn.block.clone();
     
     input_fn.block = Box::new(parse_quote!({
         use std::sync::atomic::{AtomicU32, Ordering};
         use std::sync::Mutex;
         use std::time::{Instant, Duration};
         
         // Thread-safe failure counters
         static FAILURE_COUNT: AtomicU32 = AtomicU32::new(0);
         static LAST_SUCCESS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
         
         // Reset failure count after 30 seconds of success
         let now = Instant::now();
         let last_success_time = LAST_SUCCESS.load(Ordering::Relaxed);
         
         if last_success_time > 0 {
             let elapsed = now.duration_since(Instant::now() - Duration::from_secs(last_success_time));
             if elapsed > Duration::from_secs(30) {
                 FAILURE_COUNT.store(0, Ordering::Relaxed);
             }
         }
         
         // Check if circuit is open (too many failures)
         let failures = FAILURE_COUNT.load(Ordering::Relaxed);
         if failures >= #threshold {
             liblogger::log_error!(
                 &format!("Circuit breaker open for {}: {} failures exceeded threshold {}", 
                     #fn_name, failures, #threshold),
                 None
             );
             return Err(format!("Circuit breaker open for {}", #fn_name).into());
         }
         
         // Call the function and track success/failure
         let result = #orig_block;
         
         // Use pattern matching for Result
         match &result {
             Ok(_) => {
                 // Reset failure count on success
                 FAILURE_COUNT.store(0, Ordering::Relaxed);
                 LAST_SUCCESS.store(now.elapsed().as_secs(), Ordering::Relaxed);
             },
             Err(_) => {
                 // Increment failure count
                 FAILURE_COUNT.fetch_add(1, Ordering::Relaxed);
                 let new_count = FAILURE_COUNT.load(Ordering::Relaxed);
                 
                 liblogger::log_warn!(&format!(
                     "Circuit breaker: {} failed ({}/{} failures)", 
                     #fn_name, new_count, #threshold
                 ), None);
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
     let rate = args.rate.unwrap_or(5);
     
     let mut input_fn = parse_macro_input!(input as ItemFn);
     let fn_name = get_fn_name(&input_fn);
     let orig_block = input_fn.block.clone();
     
     input_fn.block = Box::new(parse_quote!({
         use std::sync::atomic::{AtomicUsize, Ordering};
         use std::time::{SystemTime, UNIX_EPOCH};
         
         static COUNTER: AtomicUsize = AtomicUsize::new(0);
         static LAST_MINUTE: AtomicUsize = AtomicUsize::new(0);
         static SKIPPED_COUNT: AtomicUsize = AtomicUsize::new(0);
         
         // Get current minute for rate limiting window
         let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
         let current_minute = (now.as_secs() / 60) as usize;
         
         // Check if we're in a new minute or still in the rate limit
         let should_log = {
             let last_minute = LAST_MINUTE.load(Ordering::SeqCst);
             if last_minute != current_minute {
                 // New minute, reset counter and log a summary of skipped messages
                 LAST_MINUTE.store(current_minute, Ordering::SeqCst);
                 let skipped = SKIPPED_COUNT.swap(0, Ordering::SeqCst);
                 if skipped > 0 {
                     liblogger::log_info!(
                         &format!("Throttled logs for {}: skipped {} logs in previous minute", 
                             #fn_name, skipped),
                         None
                     );
                 }
                 COUNTER.store(1, Ordering::SeqCst);
                 true
             } else {
                 // Same minute, check counter
                 let count = COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
                 if count <= #rate as usize {
                     true
                 } else {
                     SKIPPED_COUNT.fetch_add(1, Ordering::SeqCst);
                     false
                 }
             }
         };
         
         let result = #orig_block;
         
         // Only log if within rate limits
         if should_log {
             // Simple logging without trying to match on the result type
             liblogger::log_info!(&format!("{} executed", #fn_name), None);
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
         
         // Use pattern matching to handle different result types
         match &result {
             Ok(_) => {
                 liblogger::log_info!(&format!("Dependency call to {} completed in {} ms", #target, duration_ms), None);
             },
             Err(err) => {
                 liblogger::log_error!(
                     &format!("Dependency call to {} failed after {} ms with error: {:?}",
                         #target, duration_ms, err),
                     None
                 );
             },
             _ => {
                 // For non-Result types
                 liblogger::log_info!(&format!("Dependency call to {} completed in {} ms", #target, duration_ms), None);
             }
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
         liblogger::log_debug!(&format!("{} returned: {:?}", #fn_name, result), None);
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
         
         // Use pattern matching to determine success or failure
         match &result {
             Ok(_) => {
                 liblogger::log_info!(
                     &format!("Health check {} passed in {} ms", #fn_name, duration.as_millis()),
                     None
                 );
             },
             Err(err) => {
                 liblogger::log_error!(
                     &format!("Health check {} failed in {} ms: {:?}", 
                         #fn_name, duration.as_millis(), err),
                     None
                 );
             }
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
         
         // Use pattern matching to handle the Result
         match &result {
             Ok(val) => {
                 // Success case with different log levels
                 let level = #success_level_str;
                 if level == "debug" {
                     liblogger::log_debug!(&format!("{} succeeded with result: {:?}", #fn_name, val), None);
                 } else if level == "warn" {
                     liblogger::log_warn!(&format!("{} succeeded with result: {:?}", #fn_name, val), None);
                 } else if level == "error" {
                     liblogger::log_error!(&format!("{} succeeded with result: {:?}", #fn_name, val), None);
                 } else {
                     liblogger::log_info!(&format!("{} succeeded with result: {:?}", #fn_name, val), None);
                 }
             },
             Err(err) => {
                 // Error case with different log levels
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
         }
         
         result
     }));
     
     TokenStream::from(quote!(#input_fn))
 }
 