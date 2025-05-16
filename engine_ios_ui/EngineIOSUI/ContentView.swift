import SwiftUI

struct ContentView: View {
    @EnvironmentObject var serverController: RustServerController
    @State private var showLogs = false
    
    var body: some View {
        NavigationView {
            ZStack {
                switch serverController.serverState {
                case .notStarted:
                    VStack(spacing: 20) {
                        Image(systemName: "server.rack")
                            .font(.system(size: 70))
                            .foregroundColor(.blue)
                        
                        Text("Rusty Plugin Workflow")
                            .font(.largeTitle)
                            .fontWeight(.bold)
                        
                        Text("iOS Integration Demo")
                            .font(.title2)
                            .foregroundColor(.gray)
                        
                        Button(action: {
                            serverController.startServer()
                        }) {
                            HStack {
                                Image(systemName: "play.fill")
                                Text("Start Server")
                            }
                            .frame(minWidth: 200)
                            .padding()
                            .background(Color.blue)
                            .foregroundColor(.white)
                            .cornerRadius(10)
                        }
                        .padding(.top, 30)
                    }
                    
                case .starting:
                    VStack {
                        ProgressView("Starting Rust server...")
                            .progressViewStyle(CircularProgressViewStyle())
                            .scaleEffect(1.5)
                        
                        Text("Initializing plugin system...")
                            .font(.caption)
                            .foregroundColor(.gray)
                            .padding(.top, 20)
                    }
                
                case .running:
                    VStack {
                        if let url = serverController.serverUrl {
                            WebViewContainer(url: url)
                        } else {
                            Text("Server URL not available")
                                .foregroundColor(.orange)
                        }
                    }
                    
                case .stopping:
                    ProgressView("Stopping server...")
                        .progressViewStyle(CircularProgressViewStyle())
                
                case .stopped:
                    VStack {
                        Image(systemName: "stop.circle")
                            .font(.system(size: 60))
                            .foregroundColor(.orange)
                        
                        Text("Server stopped")
                            .font(.headline)
                            .padding()
                        
                        Button("Restart Server") {
                            serverController.startServer()
                        }
                        .buttonStyle(.borderedProminent)
                    }
                    
                case .failed(let error):
                    VStack {
                        Image(systemName: "exclamationmark.triangle.fill")
                            .foregroundColor(.red)
                            .font(.system(size: 50))
                            .padding()
                        
                        Text("Failed to start server")
                            .font(.headline)
                        
                        Text(error.localizedDescription)
                            .font(.subheadline)
                            .multilineTextAlignment(.center)
                            .padding()
                        
                        Button("Retry") {
                            serverController.startServer()
                        }
                        .buttonStyle(.borderedProminent)
                        .padding()
                .padding()
            }
        }
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
            .environmentObject(RustServerController())
    }
}
