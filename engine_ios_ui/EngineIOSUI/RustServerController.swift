import Foundation
import Combine

enum ServerState {
    case notStarted
    case starting
    case running
    case failed(Error)
}

class RustServerController: ObservableObject {
    @Published var serverState: ServerState = .notStarted
    private var checkTimer: Timer?
    
    deinit {
        stopServer()
    }
    
    func startServer() {
        serverState = .starting
        
        // Call into Rust code via FFI
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            do {
                let result = rust_start_server()
                if result != 0 {
                    throw NSError(domain: "RustServerError", code: Int(result), userInfo: [
                        NSLocalizedDescriptionKey: "Failed to start Rust server with error code \(result)"
                    ])
                }
                
                // Start checking if the server is available
                DispatchQueue.main.async {
                    self?.startServerAvailabilityCheck()
                }
            } catch {
                DispatchQueue.main.async {
                    self?.serverState = .failed(error)
                }
            }
        }
    }
    
    private func startServerAvailabilityCheck() {
        // Check every second if the server is responding
        checkTimer = Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true) { [weak self] timer in
            self?.checkServerAvailability()
        }
    }
    
    private func checkServerAvailability() {
        guard let url = URL(string: "http://localhost:8080/health") else { return }
        
        let task = URLSession.shared.dataTask(with: url) { [weak self] _, response, error in
            DispatchQueue.main.async {
                if let error = error {
                    print("Server check failed: \(error.localizedDescription)")
                    return
                }
                
                if let httpResponse = response as? HTTPURLResponse, httpResponse.statusCode == 200 {
                    self?.serverState = .running
                    self?.checkTimer?.invalidate()
                }
            }
        }
        task.resume()
    }
    
    func stopServer() {
        checkTimer?.invalidate()
        
        // Call into Rust code to stop server
        let result = rust_stop_server()
        if result != 0 {
            print("Warning: Failed to stop Rust server cleanly, error code: \(result)")
        }
    }
}
