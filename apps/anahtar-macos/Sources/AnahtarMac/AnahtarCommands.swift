import SwiftUI

struct AnahtarCommands: Commands {
    @ObservedObject var model: AppModel

    var body: some Commands {
        CommandGroup(after: .newItem) {
            Button("Open Vault…") { model.chooseVault() }
                .keyboardShortcut("o", modifiers: [.command])
            Button("Lock Vault") { model.lockVault() }
                .keyboardShortcut("l", modifiers: [.command])
            Button("Refresh") { model.refresh() }
                .keyboardShortcut("r", modifiers: [.command])
        }
        CommandGroup(after: .textEditing) {
            Button("Focus Search") { model.focusSearch() }
                .keyboardShortcut("f", modifiers: [.command])
        }
    }
}
