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
            Button("Run Audit") { model.runAudit() }
                .keyboardShortcut("a", modifiers: [.command, .shift])
                .disabled(!model.unlocked)
        }
        CommandGroup(after: .textEditing) {
            Button("Focus Search") { model.focusSearch() }
                .keyboardShortcut("f", modifiers: [.command])
        }
        SidebarCommands()
    }
}
