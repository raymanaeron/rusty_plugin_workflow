import Foundation

// Import the C interface functions from the Rust library
@_cdecl("rust_start_server")
public func rust_start_server() -> Int32

@_cdecl("rust_stop_server")
public func rust_stop_server() -> Int32
