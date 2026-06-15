import SwiftUI

@main
struct AnahtarApp: App {
    @StateObject private var model = AppModel()

    var body: some Scene {
        WindowGroup("Anahtar") {
            RootView()
                .environmentObject(model)
                .frame(minWidth: 900, minHeight: 600)
                .task { model.refreshBackendStatus() }
        }
        .commands {
            AnahtarCommands(model: model)
        }
    }
}
