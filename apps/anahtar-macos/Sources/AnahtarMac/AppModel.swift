import AppKit
import Foundation
import UniformTypeIdentifiers

struct RecentVault: Codable, Identifiable, Hashable {
    var id: String { path }
    let path: String
    let keyFilePath: String?

    var displayName: String {
        URL(fileURLWithPath: path).lastPathComponent
    }
}

enum GroupSelection: Hashable {
    case allEntries
    case group(String)
}

@MainActor
final class AppModel: ObservableObject {
    @Published var statusMessage = "Native macOS scaffold ready." {
        didSet {
            if statusMessage != oldValue {
                showToast(statusMessage)
            }
        }
    }
    @Published var toastMessage: String? = nil
    @Published var vaultPath = ""
    @Published var keyFilePath = ""
    @Published var masterPassword = ""
    @Published var recentVaults: [RecentVault] = []
    @Published var unlocked = false
    @Published var selectedGroupSelection = GroupSelection.allEntries
    var selectedGroup: String? {
        get {
            if case let .group(path) = selectedGroupSelection {
                return path
            }
            return nil
        }
        set {
            selectedGroupSelection = newValue.map(GroupSelection.group) ?? .allEntries
        }
    }
    @Published var focusedEntryID: String? = nil
    @Published var selectedEntryID: String? = nil {
        didSet {
            if selectedEntryID != oldValue {
                loadSelectedDetail(revealPassword: false)
            }
        }
    }
    @Published var entries: [EntrySummary] = []
    @Published var visibleEntries: [EntrySummary] = []
    @Published var groups: [GroupSummary] = []
    @Published var selectedDetail: EntryDetail? = nil
    @Published var detailRevealed = false
    @Published var searchQuery = ""
    @Published var searchFocusRequest = 0
    @Published var detailFocusRequest = 0
    @Published var showAddEntrySheet = false
    @Published var showEditEntrySheet = false
    @Published var newEntryGroup = "General/Web"
    @Published var newEntryTitle = ""
    @Published var newEntryUsername = ""
    @Published var newEntryPassword = ""
    @Published var newEntryURL = ""
    @Published var newEntryNotes = ""
    @Published var editEntryGroup = ""
    @Published var editEntryTitle = ""
    @Published var editEntryUsername = ""
    @Published var editEntryPassword = ""
    @Published var editEntryURL = ""
    @Published var editEntryNotes = ""
    @Published var auditFindings: [AuditFinding] = []
    @Published var showAuditWindow = false

    private static let recentVaultsKey = "AnahtarRecentVaults"
    private let backend = BackendBridge()
    private var sessionPassword = ""
    private var toastDismissTask: Task<Void, Never>?

    init() {
        loadRecentVaults()
        if let recent = recentVaults.first {
            vaultPath = recent.path
            keyFilePath = recent.keyFilePath ?? ""
            statusMessage = "Recent vault selected. Enter the master password to unlock."
        }
    }
    private var clipboardClearTimer: Timer?
    private var ownedClipboardValue: String?

    var filteredEntries: [EntrySummary] {
        visibleEntries.filter { entry in
            guard let selectedGroup else { return true }
            let entryPath = normalizeGroupPath(entry.group_path)
            return entryPath == selectedGroup || entryPath.hasPrefix("\(selectedGroup)/")
        }
    }


    private func showToast(_ message: String) {
        let trimmed = message.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }
        toastDismissTask?.cancel()
        toastMessage = trimmed
        toastDismissTask = Task { [weak self] in
            try? await Task.sleep(nanoseconds: 3_000_000_000)
            guard !Task.isCancelled else { return }
            await MainActor.run {
                if self?.toastMessage == trimmed {
                    self?.toastMessage = nil
                }
            }
        }
    }

    func refreshBackendStatus() {
        do {
            let status = try backend.backendStatus()
            statusMessage = "Rust backend: \(status.status) · \(status.service) \(status.version)"
        } catch {
            statusMessage = error.localizedDescription
        }
    }


    func prepareAddEntry() {
        newEntryGroup = selectedGroup ?? "General/Web"
        newEntryTitle = ""
        newEntryUsername = ""
        newEntryPassword = ""
        newEntryURL = ""
        newEntryNotes = ""
        showAddEntrySheet = true
    }

    func saveNewEntry() {
        guard unlocked else { return }
        let title = newEntryTitle.trimmingCharacters(in: .whitespacesAndNewlines)
        let group = newEntryGroup.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !title.isEmpty, !group.isEmpty else {
            statusMessage = "Enter both a group path and a title."
            return
        }
        do {
            let report = try backend.addEntry(AddEntryFfiRequest(
                path: vaultPath,
                password: currentPasswordForSession(),
                key_file: optionalKeyFilePath(),
                entry: AddEntryInput(
                    group_path: group,
                    title: title,
                    username: emptyToNil(newEntryUsername),
                    password: newEntryPassword.isEmpty ? nil : newEntryPassword,
                    url: emptyToNil(newEntryURL),
                    notes: emptyToNil(newEntryNotes)
                ),
                backup_dir: nil
            ))
            showAddEntrySheet = false
            refreshAfterWrite(selecting: report.changed_entry_id)
            statusMessage = writeStatus(report)
        } catch {
            statusMessage = error.localizedDescription
        }
    }

    func deleteSelectedEntry() {
        guard unlocked, let selectedEntryID else { return }
        guard confirm(title: "Delete Entry", message: "Delete the selected entry? A backup will be created before the vault is replaced.") else { return }
        do {
            let report = try backend.deleteEntry(EntryIdFfiRequest(
                path: vaultPath,
                password: currentPasswordForSession(),
                key_file: optionalKeyFilePath(),
                entry_id: selectedEntryID,
                backup_dir: nil
            ))
            refreshAfterWrite(selecting: nil)
            statusMessage = writeStatus(report)
        } catch {
            statusMessage = error.localizedDescription
        }
    }

    func addGroupPrompt() {
        guard unlocked, let value = prompt(title: "New Group", message: "Enter the full group path:", defaultValue: selectedGroup.map { "\($0)/" } ?? "") else { return }
        do {
            let report = try backend.addGroup(GroupFfiRequest(path: vaultPath, password: currentPasswordForSession(), key_file: optionalKeyFilePath(), group_path: value, backup_dir: nil))
            refreshAfterWrite(selecting: nil)
            selectedGroup = value
            statusMessage = writeStatus(report)
        } catch {
            statusMessage = error.localizedDescription
        }
    }

    func renameSelectedGroupPrompt() {
        guard unlocked, let selectedGroup, let value = prompt(title: "Rename Group", message: "Enter the new group name:", defaultValue: selectedGroup.split(separator: "/").last.map(String.init) ?? selectedGroup) else { return }
        do {
            let report = try backend.renameGroup(RenameGroupFfiRequest(path: vaultPath, password: currentPasswordForSession(), key_file: optionalKeyFilePath(), group_path: selectedGroup, new_name: value, backup_dir: nil))
            refreshAfterWrite(selecting: nil)
            statusMessage = writeStatus(report)
        } catch {
            statusMessage = error.localizedDescription
        }
    }

    func deleteSelectedGroup() {
        guard unlocked, let selectedGroup else { return }
        guard confirm(title: "Delete Group", message: "Delete empty group \"\(selectedGroup)\"? This cannot be undone.") else { return }
        do {
            let report = try backend.deleteGroup(GroupFfiRequest(path: vaultPath, password: currentPasswordForSession(), key_file: optionalKeyFilePath(), group_path: selectedGroup, backup_dir: nil))
            self.selectedGroup = nil
            refreshAfterWrite(selecting: nil)
            statusMessage = writeStatus(report)
        } catch {
            statusMessage = error.localizedDescription
        }
    }

    func runAudit() {
        guard unlocked else { return }
        do {
            let report = try backend.auditVault(vaultRequest())
            auditFindings = report.findings
            showAuditWindow = true
            statusMessage = "Audit found \(report.findings.count) findings."
        } catch {
            statusMessage = error.localizedDescription
        }
    }

    private func refreshAfterWrite(selecting entryID: String?) {
        do {
            let request = vaultRequest()
            entries = try backend.unlockVault(request)
            visibleEntries = entries
            groups = try backend.listGroups(request)
            focusedEntryID = entryID
            selectedEntryID = entryID
            selectedDetail = nil
            detailRevealed = false
            if entryID != nil {
                loadSelectedDetail(revealPassword: false)
            }
        } catch {
            statusMessage = error.localizedDescription
        }
    }


    func prepareEditEntry() {
        guard let detail = selectedDetail else {
            statusMessage = "Select an entry first."
            return
        }
        editEntryGroup = normalizeGroupPath(detail.group_path)
        editEntryTitle = detail.title ?? ""
        editEntryUsername = detail.username ?? ""
        editEntryPassword = ""
        editEntryURL = detail.url ?? ""
        editEntryNotes = detail.notes ?? ""
        showEditEntrySheet = true
    }

    func saveEditedEntry() {
        guard unlocked, let selectedEntryID, let original = selectedDetail else { return }
        let title = editEntryTitle.trimmingCharacters(in: .whitespacesAndNewlines)
        let group = editEntryGroup.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !title.isEmpty, !group.isEmpty else {
            statusMessage = "Enter both a group path and a title."
            return
        }
        do {
            var report = try backend.editEntry(EditEntryFfiRequest(
                path: vaultPath,
                password: currentPasswordForSession(),
                key_file: optionalKeyFilePath(),
                entry_id: selectedEntryID,
                entry: EditEntryInput(
                    title: title,
                    username: emptyToNil(editEntryUsername),
                    password: editEntryPassword.isEmpty ? nil : editEntryPassword,
                    url: emptyToNil(editEntryURL),
                    notes: emptyToNil(editEntryNotes)
                ),
                backup_dir: nil
            ))
            let originalGroup = normalizeGroupPath(original.group_path)
            if group != originalGroup {
                report = try backend.moveEntry(MoveEntryFfiRequest(
                    path: vaultPath,
                    password: currentPasswordForSession(),
                    key_file: optionalKeyFilePath(),
                    entry_id: selectedEntryID,
                    group_path: group,
                    backup_dir: nil
                ))
            }
            showEditEntrySheet = false
            refreshAfterWrite(selecting: selectedEntryID)
            statusMessage = writeStatus(report)
        } catch {
            statusMessage = error.localizedDescription
        }
    }


    func selectRecentVault(_ recent: RecentVault) {
        vaultPath = recent.path
        keyFilePath = recent.keyFilePath ?? ""
        masterPassword = ""
        statusMessage = "Recent vault selected. Enter the master password to unlock."
    }

    func clearRecentVaults() {
        recentVaults = []
        UserDefaults.standard.removeObject(forKey: Self.recentVaultsKey)
        statusMessage = "Recent vaults cleared."
    }

    private func loadRecentVaults() {
        guard let data = UserDefaults.standard.data(forKey: Self.recentVaultsKey),
              let decoded = try? JSONDecoder().decode([RecentVault].self, from: data) else {
            recentVaults = []
            return
        }
        recentVaults = decoded.filter { !$0.path.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty }
    }

    private func rememberCurrentVault() {
        let path = vaultPath.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !path.isEmpty else { return }
        let keyFile = optionalKeyFilePath()
        let current = RecentVault(path: path, keyFilePath: keyFile)
        var updated = [current]
        updated.append(contentsOf: recentVaults.filter { $0.path != path })
        recentVaults = Array(updated.prefix(10))
        if let data = try? JSONEncoder().encode(recentVaults) {
            UserDefaults.standard.set(data, forKey: Self.recentVaultsKey)
        }
    }

    func chooseVault() {
        if let url = openFilePanel(title: "Choose KDBX Vault", allowedExtensions: ["kdbx", "kdb"]) {
            vaultPath = url.path
            statusMessage = "Selected vault: \(url.lastPathComponent)"
        }
    }

    func chooseKeyFile() {
        if let url = openFilePanel(title: "Choose Key File", allowedExtensions: nil) {
            keyFilePath = url.path
            statusMessage = "Selected key file: \(url.lastPathComponent)"
        }
    }

    func unlockVault() {
        guard !vaultPath.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else {
            statusMessage = "Choose a vault file first."
            return
        }
        guard !masterPassword.isEmpty else {
            statusMessage = "Enter the master password to unlock this vault."
            return
        }

        do {
            let request = vaultRequest()
            let loadedEntries = try backend.unlockVault(request)
            let loadedGroups = try backend.listGroups(request)
            entries = loadedEntries
            visibleEntries = loadedEntries
            groups = loadedGroups
            selectedGroup = nil
            focusedEntryID = nil
            selectedEntryID = nil
            selectedDetail = nil
            detailRevealed = false
            unlocked = true
            sessionPassword = masterPassword
            rememberCurrentVault()
            masterPassword = ""
            statusMessage = "Unlocked \(loadedEntries.count) entries."
        } catch {
            statusMessage = error.localizedDescription
        }
    }

    func search() {
        updateSearchResults(showStatus: true)
    }

    func updateSearchResults(showStatus: Bool = false) {
        let query = searchQuery.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        if query.isEmpty {
            visibleEntries = entries
        } else {
            visibleEntries = entries.filter { entry in
                [entry.title, entry.username, entry.url, Optional(entry.group_path)]
                    .compactMap { $0?.lowercased() }
                    .contains { $0.contains(query) }
            }
        }
        focusedEntryID = nil
        selectedEntryID = nil
        selectedDetail = nil
        detailRevealed = false
        if showStatus {
            statusMessage = query.isEmpty ? "Showing \(visibleEntries.count) entries." : "Search returned \(visibleEntries.count) entries."
        }
    }

    func resetList() {
        searchQuery = ""
        visibleEntries = entries
        focusedEntryID = nil
        selectedEntryID = nil
        selectedDetail = nil
        detailRevealed = false
        statusMessage = "Showing \(entries.count) entries."
    }

    func toggleRevealPassword() {
        loadSelectedDetail(revealPassword: !detailRevealed)
    }

    func copyUsername() {
        guard let value = selectedDetail?.username, !value.isEmpty else { return }
        copyWithOwnedClear(value, label: "username")
    }

    func copyURL() {
        guard let value = selectedDetail?.url, !value.isEmpty else { return }
        copyWithOwnedClear(value, label: "URL")
    }

    func copyPassword() {
        guard unlocked, let selectedEntryID else { return }
        do {
            let request = ShowEntryRequest(
                path: vaultPath,
                password: currentPasswordForSession(),
                key_file: optionalKeyFilePath(),
                selector_kind: "id",
                selector_value: selectedEntryID,
                reveal_password: true
            )
            let detail = try backend.showEntry(request)
            guard let password = detail.password, !password.isEmpty else { return }
            copyWithOwnedClear(password, label: "password")
        } catch {
            statusMessage = error.localizedDescription
        }
    }

    func copyTotp() {
        guard unlocked, let selectedEntryID, selectedDetail?.has_totp == true else { return }
        do {
            let request = TotpRequest(
                path: vaultPath,
                password: currentPasswordForSession(),
                key_file: optionalKeyFilePath(),
                selector_kind: "id",
                selector_value: selectedEntryID
            )
            let code = try backend.totpCode(request)
            copyWithOwnedClear(code.code, label: "TOTP code valid for \(code.valid_for_seconds)s")
        } catch {
            statusMessage = error.localizedDescription
        }
    }

    func loadSelectedDetail(revealPassword: Bool) {
        guard unlocked, let selectedEntryID else { return }
        do {
            let request = ShowEntryRequest(
                path: vaultPath,
                password: currentPasswordForSession(),
                key_file: optionalKeyFilePath(),
                selector_kind: "id",
                selector_value: selectedEntryID,
                reveal_password: revealPassword
            )
            selectedDetail = try backend.showEntry(request)
            detailRevealed = revealPassword
            statusMessage = "Loaded entry detail."
        } catch {
            statusMessage = error.localizedDescription
        }
    }


    func focusEntry(_ entryID: String) {
        focusedEntryID = entryID
    }

    func selectEntry(_ entryID: String) {
        focusedEntryID = entryID
        selectedEntryID = entryID
    }

    func selectAdjacentEntry(delta: Int) {
        let candidates = filteredEntries
        guard !candidates.isEmpty else { return }
        let currentID = focusedEntryID ?? selectedEntryID
        let currentIndex = currentID.flatMap { id in candidates.firstIndex { $0.id == id } }
        let nextIndex: Int
        if let currentIndex {
            nextIndex = min(max(currentIndex + delta, 0), candidates.count - 1)
        } else {
            nextIndex = delta < 0 ? candidates.count - 1 : 0
        }
        focusedEntryID = candidates[nextIndex].id
    }

    func openSelectedEntryDetail() {
        guard let entryID = focusedEntryID ?? selectedEntryID else { return }
        let previousEntryID = selectedEntryID
        selectedEntryID = entryID
        if previousEntryID == entryID {
            loadSelectedDetail(revealPassword: false)
        }
    }

    func copyValue(_ value: String, label: String) {
        guard !value.isEmpty else { return }
        copyWithOwnedClear(value, label: label)
    }

    func focusDetailAttribute() {
        detailFocusRequest += 1
    }

    func focusSearch() {
        guard unlocked else { return }
        searchFocusRequest += 1
    }

    func lockVault() {
        unlocked = false
        masterPassword = ""
        sessionPassword = ""
        entries = []
        visibleEntries = []
        groups = []
        searchQuery = ""
        selectedGroup = nil
        focusedEntryID = nil
        selectedEntryID = nil
        selectedDetail = nil
        detailRevealed = false
        showAuditWindow = false
        clearClipboardTimer()
        toastDismissTask?.cancel()
        toastDismissTask = nil
        statusMessage = "Locked. In-memory session cleared."
    }

    func refresh() {
        guard unlocked else {
            refreshBackendStatus()
            return
        }
        do {
            let request = vaultRequest()
            entries = try backend.unlockVault(request)
            groups = try backend.listGroups(request)
            visibleEntries = entries
            focusedEntryID = nil
            selectedEntryID = nil
            selectedDetail = nil
            detailRevealed = false
            statusMessage = "Refreshed \(entries.count) entries."
        } catch {
            statusMessage = error.localizedDescription
        }
    }

    private func copyWithOwnedClear(_ value: String, label: String) {
        let pasteboard = NSPasteboard.general
        pasteboard.clearContents()
        pasteboard.setString(value, forType: .string)
        ownedClipboardValue = value
        clipboardClearTimer?.invalidate()
        clipboardClearTimer = Timer.scheduledTimer(withTimeInterval: 30, repeats: false) { [weak self] _ in
            Task { @MainActor in
                self?.clearClipboardIfOwned()
            }
        }
        statusMessage = "Copied \(label). Clipboard will clear in 30 seconds if unchanged."
    }

    private func clearClipboardIfOwned() {
        guard let ownedClipboardValue else { return }
        let pasteboard = NSPasteboard.general
        if pasteboard.string(forType: .string) == ownedClipboardValue {
            pasteboard.clearContents()
            statusMessage = "Clipboard cleared."
        }
        self.ownedClipboardValue = nil
        clipboardClearTimer = nil
    }

    private func clearClipboardTimer() {
        clipboardClearTimer?.invalidate()
        clipboardClearTimer = nil
        ownedClipboardValue = nil
    }


    private func emptyToNil(_ value: String) -> String? {
        let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
        return trimmed.isEmpty ? nil : trimmed
    }

    private func writeStatus(_ report: WriteReport) -> String {
        let backup = report.backup_path.map { " Backup: \($0)." } ?? ""
        let target = report.final_target_path.map { " Saved to \($0)." } ?? ""
        return "Saved.\(backup)\(target)"
    }

    private func confirm(title: String, message: String) -> Bool {
        let alert = NSAlert()
        alert.messageText = title
        alert.informativeText = message
        alert.alertStyle = .warning
        alert.addButton(withTitle: "Continue")
        alert.addButton(withTitle: "Cancel")
        return alert.runModal() == .alertFirstButtonReturn
    }

    private func prompt(title: String, message: String, defaultValue: String) -> String? {
        let alert = NSAlert()
        alert.messageText = title
        alert.informativeText = message
        alert.addButton(withTitle: "Save")
        alert.addButton(withTitle: "Cancel")
        let field = NSTextField(string: defaultValue)
        field.frame = NSRect(x: 0, y: 0, width: 320, height: 24)
        alert.accessoryView = field
        guard alert.runModal() == .alertFirstButtonReturn else { return nil }
        let trimmed = field.stringValue.trimmingCharacters(in: .whitespacesAndNewlines)
        return trimmed.isEmpty ? nil : trimmed
    }

    private func vaultRequest() -> VaultRequest {
        VaultRequest(path: vaultPath, password: currentPasswordForSession(), key_file: optionalKeyFilePath())
    }

    private func currentPasswordForSession() -> String {
        unlocked ? sessionPassword : masterPassword
    }

    private func optionalKeyFilePath() -> String? {
        let trimmed = keyFilePath.trimmingCharacters(in: .whitespacesAndNewlines)
        return trimmed.isEmpty ? nil : trimmed
    }

    private func normalizeGroupPath(_ path: String) -> String {
        path.replacingOccurrences(of: "^Root/?", with: "", options: .regularExpression)
            .trimmingCharacters(in: CharacterSet(charactersIn: "/"))
    }

    private func openFilePanel(title: String, allowedExtensions: [String]?) -> URL? {
        let panel = NSOpenPanel()
        panel.title = title
        panel.allowsMultipleSelection = false
        panel.canChooseDirectories = false
        panel.canChooseFiles = true
        if let allowedExtensions {
            panel.allowedContentTypes = allowedExtensions.map { UTType(filenameExtension: $0) ?? .data }
        }
        return panel.runModal() == .OK ? panel.url : nil
    }
}
