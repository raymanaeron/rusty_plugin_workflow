import SwiftUI

@main
struct EngineIOSUIApp: App {
    @StateObject private var serverController = RustServerController()
    
    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(serverController)
                .onAppear {
                    serverController.startServer()
                }
        }
    }
}
