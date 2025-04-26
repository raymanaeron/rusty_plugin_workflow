import SwiftUI

struct ContentView: View {
    @EnvironmentObject var serverController: RustServerController
    
    var body: some View {
        ZStack {
            switch serverController.serverState {
            case .notStarted, .starting:
                ProgressView("Starting server...")
                    .progressViewStyle(CircularProgressViewStyle())
                    .scaleEffect(1.5)
                
            case .running:
                WebViewContainer(url: URL(string: "http://localhost:8080")!)
                
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
                }
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
