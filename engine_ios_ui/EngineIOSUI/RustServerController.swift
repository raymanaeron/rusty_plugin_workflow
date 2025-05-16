import Foundation
import Combine

enum ServerState {
    case notStarted
    case starting
    case running
    case stopping
    case stopped
    case failed(Error)
}

class RustServerController: ObservableObject {
    @Published var serverState: ServerState = .notStarted
    @Published var serverLogs: [String] = []
    @Published var serverUrl: URL? = nil
    private var checkTimer: Timer?
    
    // Default port for the server
    private let serverPort: Int = 8080
    
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
        // Add initial log
        addLog("Starting server availability check...")
        
        // Check every second if the server is responding
        checkTimer = Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true) { [weak self] timer in
            self?.checkServerAvailability()
        }
    }
    
    private func checkServerAvailability() {
        let urlString = "http://localhost:\(serverPort)/health"
        guard let url = URL(string: urlString) else { 
            addLog("ERROR: Invalid URL: \(urlString)")
            return 
        }
        
        let task = URLSession.shared.dataTask(with: url) { [weak self] _, response, error in
            DispatchQueue.main.async {
                if let error = error {
                    self?.addLog("Server check failed: \(error.localizedDescription)")
                    return
                }
                
                if let httpResponse = response as? HTTPURLResponse, httpResponse.statusCode == 200 {
                    self?.serverUrl = URL(string: "http://localhost:\(self?.serverPort ?? 8080)")
                    self?.serverState = .running
                    self?.checkTimer?.invalidate()
                    self?.addLog("✅ Server is now running at http://localhost:\(self?.serverPort ?? 8080)")
                }
            }
        }
        task.resume()
    }
    
    func addLog(_ message: String) {
        let dateFormatter = DateFormatter()
        dateFormatter.dateFormat = "HH:mm:ss"
        let timestamp = dateFormatter.string(from: Date())
        let logEntry = "[\(timestamp)] \(message)"
        
        DispatchQueue.main.async {
            self.serverLogs.append(logEntry)
            // Keep only the most recent 100 logs
            if self.serverLogs.count > 100 {
                self.serverLogs.removeFirst()
            }
        }
    }
    
    func stopServer() {
        addLog("Stopping Rust server...")
        serverState = .stopping
        checkTimer?.invalidate()
        
        // Call into Rust code to stop server
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in            let result = rust_stop_server()
            DispatchQueue.main.async {
                if result != 0 {
                    self?.addLog("⚠️ Warning: Failed to stop Rust server cleanly, error code: \(result)")
                } else {
                    self?.addLog("Server stopped successfully")
                }
                self?.serverState = .stopped
            }
        }
    }
}
