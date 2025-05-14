## 1. Problem Statement

Every developer has faced those moments: a production issue occurs, and you're left digging through sparse logs trying to piece together what happened. Or worse, finding no relevant logs at all. 

### Why Do We Need Logging?

Logging serves multiple critical functions in software development:

1. **Debugging**: When things inevitably go wrong, logs are often your only window into what happened, especially in production environments where you can't attach a debugger.

2. **Understanding Flow**: Logs help trace the execution path through your code, showing which functions were called, in what order, and with what parameters.

3. **Monitoring System Health**: Regular status logs help identify performance bottlenecks, resource limitations, or unexpected behaviors before they become critical failures.

4. **Auditing**: For many applications, especially those handling sensitive data or financial transactions, maintaining an audit trail is not just helpful—it's often a regulatory requirement.

### Common Pain Points

Despite its importance, implementing effective logging remains challenging:

#### For Developers:
- **Inconsistency**: Logging becomes inconsistent across a codebase without a standardized approach, especially with multiple contributors.
- **Verbosity vs. Signal**: Too little logging leaves you blind; too much creates noise that obscures essential information.
- **Context Loss**: Logs without sufficient context (like function parameters, return values, or system state) are often useless for debugging.
- **Repetitive Boilerplate**: Adding proper context, error handling, and log formatting requires repetitive code that clutters business logic.
- **Performance Concerns**: Developers often avoid comprehensive logging due to fears about performance impact, especially on hot paths.

#### For Operations and SRE Teams:
- **Inconsistent Formats**: Varying log formats make automated parsing and alerting difficult.
- **Missing Correlation IDs**: Tracing request flows becomes nearly impossible without identifiers linking related logs across distributed systems.
- **Insufficient Detail**: Logs that lack timing information, component identifiers, or error specifics hinder incident response.
- **Log Loss**: Critical log messages may be lost before being persisted under high load or during crashes.
- **Noisy Alerts**: Poorly configured logging can trigger unnecessary alerts, leading to alert fatigue.

### Technical Challenges in Rust

Beyond these common issues, Rust developers face additional challenges:

- **Rust's Ownership Model**: Traditional logging patterns can clash with Rust's ownership and borrowing rules, requiring awkward workarounds.
- **Generic Error Handling**: Working with different error types across a codebase often leads to brittle, reflection-based logging that breaks type safety.
- **Asynchronous Code**: Ensuring logs maintain proper context and order in async code requires special consideration.

### Production-Grade Concerns

As applications scale and mature, additional concerns emerge:

- **Dual-State Logging**: Synchronous and asynchronous logging paths can lead to file corruption when both write to the same file without coordination.
- **Buffer Flushing**: Logs may remain in memory buffers if not reliably flushed during unexpected terminations.
- **Shutdown Procedures**: Inadequate shutdown can result in lost messages when an application exits normally.
- **Backpressure Handling**: High-volume logging can overwhelm message channels, leading to silent log loss without visibility.
- **Type Flexibility**: Logging macros that rely on reflection for error handling often break when faced with custom error types.

The liblogger Framework addresses these challenges by providing a structured, consistent logging approach that integrates seamlessly with Rust's idioms. It eliminates boilerplate through procedural macros, ensures reliable log delivery with proper backpressure handling, and maintains performance through asynchronous processing—all while providing the context and format consistency needed for effective debugging and monitoring.

---

## 2. Architecture Overview

### How liblogger Addresses Your Challenges

liblogger was designed to solve the specific challenges developers face with logging:

| Your Challenge | Our Solution |
|---------------|--------------|
| **Inconsistent logging across teams** | Standardized API and configurable formatting |
| **Cluttered business logic** | Procedural macros capture context without manual code |
| **Performance concerns** | Asynchronous processing with minimal overhead |
| **Lost logs at shutdown** | Safe shutdown protocol with message confirmation |
| **File corruption** | Unified file handles between sync and async paths |
| **Log loss during crashes** | Configurable immediate flush guarantees |
| **Custom error type handling** | Pattern matching instead of reflection |
| **Backpressure saturation** | Automatic fallback with dropped message tracking |

### Core Components

liblogger is structured around four primary components that work together to provide a robust logging experience:

#### 1. The Logger Facade

At the API level, the `Logger` struct provides a simple, consistent interface through static methods and macros:

```rust
// Core logging macros that handle context capture automatically
log_debug!("Connection initialized with timeout: {}", timeout_ms);
log_info!("User profile updated", Some(format!("user_id={}", user.id)));
log_warn!("Database connection pool running low");
log_error!("Payment processing failed: {}", error);
```

This facade implements a thread-safe singleton pattern (using `OnceCell` and `Arc<Mutex<>>`) that requires just a single initialization call, after which any component in your application can log without maintaining state.

#### 2. The Configuration System

Rather than hard-coding behavior, Rusty Logger separates configuration from implementation:

```toml
[logging]
type = "file"                           # Output destination
threshold = "info"                      # Minimum severity level
file_path = "application.log"           # Log file name
force_flush = true                      # For critical data
```

Configuration can be loaded from TOML files or programmatically constructed, making it adaptable to different environments (development, testing, production) without code changes.

#### 3. The Output System

Based on your configuration, Rusty Logger dynamically creates the appropriate output handler:

- **Console Output**: Writes formatted logs to stdout
- **File Output**: Writes to files with proper directory creation and path handling
- **HTTP Output**: Sends logs to remote endpoints for centralized collection

All outputs implement a common trait that ensures consistent behavior while allowing specialized handling for each destination type.

#### 4. The Processing Pipeline

Behind the scenes, Rusty Logger implements two processing paths:

**Asynchronous Path**:
1. Log calls are converted to messages and sent to a Tokio channel
2. A background task processes these messages without blocking your application
3. Messages are formatted and written to the configured destination
4. During shutdown, a special command ensures all pending messages are processed

**Synchronous Fallback**:
1. If the async channel is full or disabled, logs fall back to synchronous processing
2. The system tracks dropped messages to provide visibility into backpressure
3. Critical paths can force immediate flushing when required

### Advanced Features

Building on this foundation, liblogger includes production-grade capabilities:

#### Procedural Macros for Automatic Context

```rust
#[log_entry_exit]
#[measure_time]
fn process_payment(payment_id: &str) -> Result<Receipt, PaymentError> {
    // Your code here
}
```

These macros eliminate boilerplate by automatically handling common logging patterns, capturing metadata like file names, line numbers, and function names without manual coding.

#### Unified File Writing

The file output system uses a shared file handle between synchronous and asynchronous paths, coordinated through `Arc<Mutex<File>>`. This eliminates the dual-state problem where competing log paths might corrupt each other's output.

#### Controlled Flush Behavior

The `force_flush` configuration option determines whether logs immediately persist to disk after writing. This gives you control over the performance vs. reliability tradeoff based on the criticality of your logs.

#### Safe Shutdown Protocol

The logger implements a command-based shutdown protocol that:
1. Sends a special `Shutdown` command through the async channel
2. Waits for confirmation that all pending logs are processed
3. Uses a timeout to prevent indefinite blocking
4. Reports any dropped messages during the application's lifetime

#### Type-Safe Error Handling

Rather than using reflection (which is brittle in Rust), our procedural macros use pattern matching:

```rust
match &result {
    Ok(value) => log_info!("Operation succeeded: {:?}", value),
    Err(error) => log_error!("Operation failed: {:?}", error)
}
```

This approach works with any `Result<T, E>` type without compromising type safety or requiring specific trait implementations.

#### Backpressure Awareness

The logger tracks messages that couldn't be processed asynchronously using atomic counters, providing visibility into potential log loss and helping you tune your logging volume or channel capacity.

By combining these components and features, liblogger provides a logging framework that's both easy to use and production-ready, addressing the full spectrum of logging challenges faced by Rust developers.

---

## 3. Quick Start Guide

### Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
liblogger = { path = "../path/to/liblogger" }
liblogger_macros = { path = "../path/to/liblogger_macros" }
```

### Basic Usage in 3 Steps

1. **Create a configuration file** named `app_config.toml` in your project root:

```toml
[logging]
type = "console"  # Options: console, file, http
threshold = "debug"  # Options: debug, info, warn, error
file_path = "application.log"  # Used when type = "file"
log_folder = "logs"  # Directory where logs are stored 
max_file_size_mb = 10  # Rotation size when using file logging
http_endpoint = "https://logs.example.com"  # Used when type = "http"
http_timeout_seconds = 5  # HTTP request timeout
force_flush = false  # Set to true for immediate flushing after each write
```

2. **Initialize the logger** in your application's entry point:

```rust
use liblogger::{Logger, log_info, log_error, shutdown_logger};

fn main() {
    // Initialize from config file
    match Logger::init_with_config_file("app_config.toml") {
        Ok(_) => log_info!("Logger initialized successfully"),
        Err(e) => {
            println!("Failed to initialize logger: {}", e);
            // Fall back to console logging
            Logger::init();
        }
    }
    
    // Your application code here
    
    // Ensure proper shutdown with all pending logs written
    match shutdown_logger() {
        Ok(_) => println!("Logger shutdown successfully"),
        Err(e) => eprintln!("Error during logger shutdown: {}", e)
    }
}
```

3. **Start logging** throughout your codebase:

```rust
use liblogger::{Logger, log_info, log_debug, log_warn, log_error};

fn process_order(order_id: &str) -> Result<(), String> {
    log_debug!("Starting order processing");
    log_info!(&format!("Processing order {}", order_id));
    
    if order_id.is_empty() {
        log_warn!("Received empty order ID");
        return Err("Empty order ID".into());
    }
    
    // Processing logic...
    
    // Check if there have been any dropped log messages due to backpressure
    let dropped_count = Logger::get_dropped_log_count();
    if dropped_count > 0 {
        log_warn!(&format!("Warning: {} log messages were dropped due to backpressure", dropped_count));
    }
    
    Ok(())
}
```

---

## 4. Configuration Details

### Configuration Options

| Parameter | Description | Default |
|-----------|-------------|---------|
| `type` | Output destination (`console`, `file`, `http`) | `console` |
| `threshold` | Minimum log level to record (`debug`, `info`, `warn`, `error`) | `info` |
| `file_path` | Log file name | `app.log` |
| `log_folder` | Directory for log files | `logs` |
| `max_file_size_mb` | Maximum file size before rotation | `10` |
| `http_endpoint` | URL for HTTP logging | `http://localhost:8080/logs` |
| `http_timeout_seconds` | HTTP request timeout | `5` |
| `force_flush` | Whether to flush logs after every write | `false` |

### Sample Configurations

#### Console Logging
```toml
[logging]
type = "console"
threshold = "debug"
```

#### File Logging with Rotation and Reliable Flushing
```toml
[logging]
type = "file"
threshold = "info"
file_path = "application.log"
log_folder = "logs"
max_file_size_mb = 5
force_flush = true  # Guarantee immediate persistence
```

#### Remote HTTP Logging
```toml
[logging]
type = "http"
threshold = "warn"
http_endpoint = "https://logging-service.example.com/ingest"
http_timeout_seconds = 3
```

---

## 5. Writing Logs

### Basic Logging Macros

The library provides four logging macros corresponding to different severity levels:

```rust
// Debug information useful during development
log_debug!("Connection pool initialized with 10 connections");

// Regular operational information
log_info!("User profile updated successfully");

// Warning conditions that should be addressed
log_warn!("Database connection pool running low (10% remaining)");

// Error conditions requiring attention
log_error!("Failed to process payment: timeout");
```

### Adding Context to Logs

You can add contextual information by providing an optional second parameter:

```rust
// With context as String
log_info!("User login successful", Some(format!("user_id={}", user_id)));

// With context as Option<String>
let context = if is_premium { Some("account=premium".to_string()) } else { None };
log_info!("Feature accessed", context);
```

### Logging in Asynchronous Code

liblogger seamlessly supports asynchronous code:

```rust
async fn process_data(user_id: &str) -> Result<(), Error> {
    log_info!(&format!("Starting data processing for user {}", user_id));
    
    let result = fetch_user_data(user_id).await;
    
    // Uses pattern matching internally to handle any Result type
    match result {
        Ok(data) => {
            log_info!("Data processing complete");
            Ok(())
        },
        Err(e) => {
            log_error!(&format!("Data processing failed: {}", e));
            Err(e)
        }
    }
}
```

### Monitoring Backpressure

Track potential log message loss due to channel saturation:

```rust
fn check_logger_health() {
    let dropped_count = Logger::get_dropped_log_count();
    if dropped_count > 0 {
        log_warn!(&format!("{} log messages were dropped due to backpressure", dropped_count));
        
        // Potential mitigations
        // - Increase channel capacity in your config
        // - Reduce logging frequency
        // - Switch to synchronous logging for critical sections
    }
}
```

---

## 6. Using Procedural Macros

### Step 1: Import and Initialize Macro Support

At the top of your source file, add:

```rust
use liblogger_macros::*;

// This brings all procedural macros into scope (must be at module level)
initialize_logger_attributes!();
```

### Step 2: Apply Macros to Functions

Annotate functions with the desired logging behaviors:

```rust
// Log function entry and exit points
#[log_entry_exit]
fn process_payment(payment_id: &str) {
    // Function implementation
}

// Measure and log execution time
#[measure_time]
fn generate_report() -> Report {
    // Time-consuming operation
}

// Log errors returned by the function - works with any Result<T, E> type
#[log_errors]
fn validate_input(data: &str) -> Result<(), ValidationError> {
    // Implementation that might return errors
}
```

### Common Macro Examples

#### Logging Entry and Exit
```rust
#[log_entry_exit]
fn update_user_profile(user_id: &str, profile_data: &ProfileData) {
    // Function implementation
}
// Produces logs like:
// "ENTRY: update_user_profile"
// "EXIT: update_user_profile"
```

#### Measuring Execution Time
```rust
#[measure_time]
fn process_large_dataset(data: &[DataPoint]) -> Analysis {
    // Time-consuming data processing
}
// Produces logs like:
// "process_large_dataset completed in 1250 ms"
```

#### Logging Function Arguments
```rust
#[log_args(user_id, action)]
fn audit_user_action(user_id: &str, action: &str, details: &ActionDetails) {
    // Only user_id and action will be logged
}
// Produces logs like:
// "Entering audit_user_action with args: user_id = "12345", action = "delete_account""
```

#### Retry Logic with Logging
```rust
#[log_retries(max_attempts=3)]
fn connect_to_database() -> Result<Connection, DbError> {
    // The function will be retried up to 3 times if it fails
    // Works with any Result<T, E> type through pattern matching
}
// Produces logs like:
// "Retry attempt 1 for connect_to_database failed: connection refused"
// "Retry attempt 2 for connect_to_database succeeded"
```

#### Creating Audit Logs
```rust
#[audit_log]
fn change_permissions(user_id: &str, new_role: Role) {
    // Security-sensitive operation
}
// Produces logs like:
// "AUDIT: [general] Operation change_permissions started"
// "AUDIT: [general] Operation change_permissions completed | Context: result_type=success"
```

#### Error Handling and Logging
```rust
#[log_errors]
fn validate_transaction(transaction: &Transaction) -> Result<(), TransactionError> {
    // Function that might fail - works with any Result<T, E>
}
// Produces logs when errors occur:
// "validate_transaction returned error: "insufficient funds""
```

#### Log Results with Custom Levels
```rust
#[log_result(success_level="debug", error_level="error")]
fn process_batch() -> Result<BatchStats, ProcessError> {
    // Implementation
}
// Will log successes at DEBUG level and failures at ERROR level
// Uses pattern matching for any Result<T, E> type
```

---

## 7. Advanced Usage

### Combining Multiple Macros

You can stack macros to combine their functionality:

```rust
#[log_entry_exit]
#[measure_time]
#[log_errors]
fn critical_operation() -> Result<OperationResult, OperationError> {
    // Implementation
}
```

### Using Request Context

Track request flow across multiple functions:

```rust
#[trace_span]
fn handle_api_request(request: &Request) -> Response {
    // Will generate a trace ID
    process_request_data(request);
}

#[trace_span]
fn process_request_data(request: &Request) {
    // Will use the same trace ID as the parent function
}

// Produces logs like:
// "[TraceID: 748405dd-ce44-48bd-9f1a-86fdb5eae237] handle_api_request started"
// "[TraceID: 748405dd-ce44-48bd-9f1a-86fdb5eae237] process_request_data started"
```

### Graceful Shutdown

To ensure all pending logs are processed before your application exits:

```rust
fn main() {
    // Initialize logger
    Logger::init_with_config_file("app_config.toml").unwrap();
    
    // Application code...
    
    // Ensure all logs are flushed before exit
    // The shutdown process will:
    // 1. Send a shutdown command through the async channel
    // 2. Wait for confirmation that all messages are processed
    // 3. Timeout after 5 seconds if confirmation isn't received
    liblogger::shutdown_logger().unwrap();
}
```

### Controlling File Flush Behavior

For critical logs that must be immediately persisted:

```toml
# In app_config.toml
[logging]
type = "file"
file_path = "critical_logs.log"
force_flush = true
```

This guarantees that logs are flushed to disk immediately after each write, preventing loss in case of sudden application termination.

---

## 8. Performance Considerations

- **Asynchronous Operation**: By default, logs are processed asynchronously to avoid blocking your application.
- **Log Level Filtering**: Log messages below the configured threshold are filtered early to minimize overhead.
- **Channel Buffering**: The async logger uses a buffered channel (1024 messages) to handle bursts of log activity.
- **Fallback Mechanism**: If the async channel is full, the logger falls back to synchronous logging.
- **Controlled Flushing**: The `force_flush` option balances performance (deferred flushing) and reliability (immediate flushing).
- **Backpressure Monitoring**: The system tracks and reports when log messages are dropped due to channel saturation.
- **Unified File Writer**: Both sync and async operations share a single file handle for improved efficiency and consistency.

---

## 9. More Examples

### Basic Initialization Pattern

```rust
match Logger::init_with_config_file("app_config.toml") {
    Ok(_) => log_info!("Logger successfully initialized from config file"),
    Err(e) => {
        println!("Error initializing logger from config: {}", e);
        Logger::init(); // Fall back to default console logging
        log_error!("Failed to initialize file logger, falling back to console");
    }
}
```

### Asynchronous Logging Example

```rust
pub fn test_async_logger() {
    // Initialize from config file
    match Logger::init_with_config_file("app_config.toml") {
        Ok(_) => println!("Async logger initialized successfully"),
        Err(e) => {
            eprintln!("Failed to initialize async logger: {}", e);
            return;
        }
    }
    
    // Generate log messages rapidly
    for i in 0..1000 {
        log_info!(&format!("Async test message {}", i));
        
        if i % 100 == 0 {
            log_warn!(&format!("Warning message at {}", i));
        }
    }
    
    // Check for any dropped messages due to backpressure
    let dropped = Logger::get_dropped_log_count();
    if dropped > 0 {
        println!("Warning: {} log messages were dropped", dropped);
    }
    
    // Ensure all logs are processed before shutdown
    // This uses the safe shutdown protocol with confirmation
    shutdown_logger().unwrap();
}
```

### Error Handling with Results Example

```rust
#[log_errors]
fn process_payment(payment: &Payment) -> Result<Receipt, PaymentError> {
    // Implementation that might fail
    // The log_errors macro works with any Result<T, E> type
    if payment.amount <= 0.0 {
        return Err(PaymentError::InvalidAmount);
    }
    
    // Processing logic
    Ok(Receipt { id: "12345".into(), amount: payment.amount })
}

// You can also use a custom macro for both success and error cases
#[log_result(success_level="info", error_level="error")]
fn verify_user(id: &str) -> Result<UserProfile, VerificationError> {
    // Implementation
    if id.is_empty() {
        return Err(VerificationError::InvalidId);
    }
    
    Ok(UserProfile { id: id.to_string(), verified: true })
}
```

### Reliable File Logging Example

```rust
// In app_config.toml:
// [logging]
// type = "file"
// file_path = "financial_transactions.log"
// force_flush = true

fn record_financial_transaction(transaction: &Transaction) -> Result<(), String> {
    // This log will be immediately flushed to disk due to the force_flush setting
    log_info!(&format!("Processing transaction {}", transaction.id), 
             Some(format!("amount={}, type={}", transaction.amount, transaction.type)));
             
    // Process transaction...
    
    // This confirmation log will also be immediately persisted
    log_info!(&format!("Transaction {} completed successfully", transaction.id));
    
    Ok(())
}
```

---

## 10. Troubleshooting

### Common Issues

1. **Missing Logs**: Check if the log level is below your threshold in the config.
2. **File Permissions**: For file output, ensure your application has write permissions.
3. **Log Directory**: The library will try to create the log directory, but check permissions if this fails.
4. **Macro Errors**: Make sure you've called `initialize_logger_attributes!()` at the module level.
5. **Shutdown Issues**: If logs are missing at program exit, ensure you call `shutdown_logger()`.
6. **Dropped Messages**: If `Logger::get_dropped_log_count()` reports dropped messages, consider increasing the channel capacity or reducing log volume.
7. **File Flushing**: For critical logs that must survive crashes, set `force_flush = true` in your configuration.

### Getting Help

If you encounter issues not covered here, please open an issue on the GitHub repository with:
- Your configuration settings
- Code samples demonstrating the issue
- Expected vs. actual behavior
