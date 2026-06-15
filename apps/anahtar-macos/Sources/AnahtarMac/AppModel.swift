import Foundation

@MainActor
final class AppModel: ObservableObject {
    @Published var statusMessage = "Native macOS scaffold ready."
    @Published var selectedGroup: String? = nil
    @Published var selectedEntryID: String? = nil

    let placeholderGroups = ["All Entries", "General", "General/Email", "General/Web"]
    let placeholderEntries = ["Github Test", "Email Example", "TOTP Example"]

    func openVault() {
        statusMessage = "Open Vault will use NSOpenPanel in the next slice."
    }

    func focusSearch() {
        statusMessage = "Search focus shortcut received."
    }

    func lockVault() {
        selectedEntryID = nil
        statusMessage = "Locked. In-memory session cleared."
    }

    func refresh() {
        statusMessage = "Refresh requested."
    }
}
