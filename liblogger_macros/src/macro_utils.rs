use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Expr, Ident, ItemFn, Lit, Meta,
    parse::{Parse, ParseStream}, punctuated::Punctuated, token::Comma
};

/// Helper function to get function name as string
pub fn get_fn_name(func: &ItemFn) -> String {
    func.sig.ident.to_string()
}

/// Parse a list of identifiers from attribute args
pub struct IdList {
    pub ids: Vec<Ident>,
}

impl Parse for IdList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args = Punctuated::<Ident, Comma>::parse_terminated(input)?;
        Ok(IdList {
            ids: args.into_iter().collect(),
        })
    }
}

/// For parsing macro attributes in format #[macro_name(name=value)]
#[derive(Default)]
pub struct MacroArgs {
    pub max_attempts: Option<u32>,
    pub rate: Option<u32>,
    pub failure_threshold: Option<u32>,
    pub category: Option<String>,
    pub flag_name: Option<String>,
    pub target: Option<String>,
    pub counter_name: Option<String>,
    pub success_level: Option<String>,
    pub error_level: Option<String>,
}

impl Parse for MacroArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = MacroArgs::default();
        
        // If input is empty, return default values
        if input.is_empty() {
            return Ok(args);
        }
        
        // Handle two different formats:
        // 1. Simple literals like #[audit_log("user_action")]
        // 2. Name-value pairs like #[log_retries(max_attempts=3)]
        
        if input.peek(Lit) {
            // Format 1: Simple literals
            let lit: Lit = input.parse()?;
            if let Lit::Str(lit_str) = lit {
                let value = lit_str.value();
                
                // Assign to the first relevant field
                if args.category.is_none() {
                    args.category = Some(value.clone());
                }
                if args.flag_name.is_none() {
                    args.flag_name = Some(value.clone());
                }
                if args.target.is_none() {
                    args.target = Some(value.clone());
                }
                if args.counter_name.is_none() {
                    args.counter_name = Some(value.clone()); 
                }
                if args.success_level.is_none() {
                    args.success_level = Some(value.clone());
                }
                
                // Handle second argument if present (for log_result)
                if input.peek(Comma) {
                    input.parse::<Comma>()?; // consume the comma
                    let second_lit: Lit = input.parse()?;
                    if let Lit::Str(lit_str) = second_lit {
                        args.error_level = Some(lit_str.value());
                    }
                }
            }
        } else {
            // Format 2: Name-value pairs
            let nested = Punctuated::<Meta, Comma>::parse_terminated(input)?;
            
            for meta in nested {
                match meta {
                    Meta::NameValue(nv) => {
                        let ident = nv.path.get_ident().map(|i| i.to_string()).unwrap_or_default();
                        
                        match ident.as_str() {
                            "max_attempts" => {
                                if let Expr::Lit(expr_lit) = &nv.value {
                                    if let Lit::Int(lit) = &expr_lit.lit {
                                        args.max_attempts = lit.base10_parse().ok();
                                    }
                                }
                            },
                            "rate" => {
                                if let Expr::Lit(expr_lit) = &nv.value {
                                    if let Lit::Int(lit) = &expr_lit.lit {
                                        args.rate = lit.base10_parse().ok();
                                    }
                                }
                            },
                            "failure_threshold" => {
                                if let Expr::Lit(expr_lit) = &nv.value {
                                    if let Lit::Int(lit) = &expr_lit.lit {
                                        args.failure_threshold = lit.base10_parse().ok();
                                    }
                                }
                            },
                            _ => {}
                        }
                    },
                    Meta::Path(path) => {
                        // Handle simple path arguments
                        if let Some(ident) = path.get_ident() {
                            let name = ident.to_string();
                            if args.category.is_none() {
                                args.category = Some(name);
                            }
                        }
                    },
                    Meta::List(list) => {
                        // Handle list arguments
                        if let Some(ident) = list.path.get_ident() {
                            let name = ident.to_string();
                            // Could handle more complex nested arguments if needed
                            if args.category.is_none() {
                                args.category = Some(name);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(args)
    }
}

/// Helper function to generate the helper functions code
pub fn define_helper_functions() -> TokenStream2 {
    quote!(
        // Helper function to check if a result is successful
        pub fn is_success<T: std::fmt::Debug + 'static>(result: &T) -> bool {
            use std::any::Any;
            
            // Using reflection to check if this is a Result type
            let _type_id = std::any::TypeId::of::<T>();
            let _result_type_id = std::any::TypeId::of::<Result<(), ()>>();
            
            // If it's a Result type, we can check for Ok/Err
            if let Some(result_any) = (result as &dyn Any).downcast_ref::<Result<(), ()>>() {
                result_any.is_ok()
            } else {
                // For non-Result types, assume success
                true
            }
        }
        
        // Helper function to extract error from Result
        pub fn extract_error<T: std::fmt::Debug + 'static>(result: &T) -> Option<String> {
            use std::any::Any;
            use std::fmt::Debug;
            
            // Using reflection with type name
            let type_name = std::any::type_name::<T>();
            if type_name.starts_with("core::result::Result") || 
               type_name.starts_with("std::result::Result") {
                // This is best-effort extraction of Error
                if let Some(err) = (result as &dyn Any).downcast_ref::<Result<(), String>>() {
                    if let Err(e) = err {
                        return Some(e.clone());
                    }
                }
                
                // Generic fallback
                if !is_success(result) {
                    return Some(format!("{:?}", result));
                }
            }
            
            None
        }
        
        // Helper functions for trace ID management
        thread_local! {
            static TRACE_ID: std::cell::RefCell<Option<String>> = std::cell::RefCell::new(None);
        }
        
        fn set_trace_id(id: &str) {
            TRACE_ID.with(|cell| {
                *cell.borrow_mut() = Some(id.to_string());
            });
        }
        
        fn get_trace_id() -> Option<String> {
            TRACE_ID.with(|cell| {
                cell.borrow().clone()
            })
        }
        
        // Placeholder for feature flag checking
        fn is_feature_enabled(feature: &str) -> bool {
            // In a real application, this would check a feature flag system
            match feature {
                "experimental" => false,
                "new_ui" => true,
                _ => false,
            }
        }
        
        // Placeholder for thread local context values
        fn get_thread_local_value(key: &str) -> Option<String> {
            // In a real application, this would retrieve values from thread local storage
            match key {
                "user_id" => Some("12345".to_string()),
                "session_id" => Some("abcd-1234-xyz".to_string()),
                "request_id" => Some("req-789".to_string()),
                _ => None,
            }
        }
    )
}
