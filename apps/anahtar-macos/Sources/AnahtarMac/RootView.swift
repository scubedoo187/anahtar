import SwiftUI

struct RootView: View {
    @EnvironmentObject private var model: AppModel

    var body: some View {
        Group {
            if model.unlocked {
                splitView
            } else {
                UnlockView()
            }
        }
        .overlay(alignment: .topTrailing) {
            if let toast = model.toastMessage {
                ToastView(message: toast)
                    .padding(.top, 16)
                    .padding(.trailing, 16)
                    .transition(.move(edge: .top).combined(with: .opacity))
            }
        }
        .sheet(isPresented: $model.showAuditWindow) {
            AuditResultsWindow()
                .environmentObject(model)
        }
        .animation(.easeOut(duration: 0.18), value: model.toastMessage)
    }

    private var splitView: some View {
        NavigationSplitView {
            GroupListView()
                .navigationSplitViewColumnWidth(min: 190, ideal: 230, max: 320)
        } content: {
            EntryListView()
                .navigationSplitViewColumnWidth(min: 280, ideal: 340, max: 460)
        } detail: {
            EntryDetailView()
        }
    }

}



private struct KeyboardCaptureView: NSViewRepresentable {
    @Binding var active: Bool
    let handler: (NSEvent) -> Bool

    func makeNSView(context: Context) -> KeyView {
        let view = KeyView()
        view.handler = handler
        view.onResign = { active = false }
        return view
    }

    func updateNSView(_ nsView: KeyView, context: Context) {
        nsView.handler = handler
        nsView.onResign = { active = false }
        if active, nsView.window?.firstResponder !== nsView {
            DispatchQueue.main.async {
                nsView.window?.makeFirstResponder(nsView)
            }
        }
    }

    final class KeyView: NSView {
        var handler: ((NSEvent) -> Bool)?
        var onResign: (() -> Void)?

        override var acceptsFirstResponder: Bool { true }

        override func resignFirstResponder() -> Bool {
            onResign?()
            return super.resignFirstResponder()
        }

        override func keyDown(with event: NSEvent) {
            if handler?(event) == true {
                return
            }
            super.keyDown(with: event)
        }
    }
}

private enum KeyCode {
    static let `return`: UInt16 = 36
    static let keypadEnter: UInt16 = 76
    static let arrowUp: UInt16 = 126
    static let arrowDown: UInt16 = 125
}

struct ToastView: View {
    let message: String

    var body: some View {
        Text(message)
            .font(.callout)
            .lineLimit(3)
            .padding(.horizontal, 14)
            .padding(.vertical, 10)
            .frame(maxWidth: 420, alignment: .leading)
            .background(.regularMaterial)
            .overlay {
                RoundedRectangle(cornerRadius: 10)
                    .stroke(Color(nsColor: .separatorColor), lineWidth: 1)
            }
            .clipShape(RoundedRectangle(cornerRadius: 10))
            .shadow(radius: 8, y: 3)
    }
}

struct UnlockView: View {
    @EnvironmentObject private var model: AppModel
    @FocusState private var passwordFocused: Bool

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text("Anahtar")
                .font(.largeTitle)
                .fontWeight(.bold)
            Text("Choose a KDBX vault and enter its master password for this in-memory session.")
                .foregroundStyle(.secondary)

            LabeledContent("Vault") {
                HStack {
                    TextField("Vault path", text: $model.vaultPath)
                    Button("Choose…") { model.chooseVault() }
                }
            }
            LabeledContent("Key file") {
                HStack {
                    TextField("Optional key-file path", text: $model.keyFilePath)
                    Button("Choose…") { model.chooseKeyFile() }
                }
            }
            LabeledContent("Password") {
                SecureField("Master password", text: $model.masterPassword)
                    .focused($passwordFocused)
                    .onSubmit { model.unlockVault() }
            }
            HStack {
                Button("Unlock") { model.unlockVault() }
                    .keyboardShortcut(.return, modifiers: [])
                Button("Backend Status") { model.refreshBackendStatus() }
            }

            if !model.recentVaults.isEmpty {
                Divider()
                HStack {
                    Text("Recent vaults")
                        .font(.headline)
                    Spacer()
                    Button("Clear") { model.clearRecentVaults() }
                        .buttonStyle(.borderless)
                }
                VStack(spacing: 0) {
                    ForEach(model.recentVaults) { recent in
                        Button {
                            model.selectRecentVault(recent)
                        } label: {
                            VStack(alignment: .leading, spacing: 2) {
                                Text(recent.displayName)
                                    .fontWeight(.semibold)
                                Text(recent.path)
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                                    .lineLimit(1)
                            }
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .padding(.vertical, 6)
                            .contentShape(Rectangle())
                        }
                        .buttonStyle(.plain)
                        Divider()
                    }
                }
            }
        }
        .textFieldStyle(.roundedBorder)
        .padding(24)
        .frame(maxWidth: 720)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .onAppear { passwordFocused = true }
    }
}

struct GroupListView: View {
    @EnvironmentObject private var model: AppModel

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 8) {
                Text("Groups")
                    .font(.headline)
                Spacer()
                Button("＋") { model.addGroupPrompt() }
                    .help("Add group")
                Button("Rename") { model.renameSelectedGroupPrompt() }
                    .disabled(model.selectedGroup == nil)
                Button("Delete") { model.deleteSelectedGroup() }
                    .disabled(model.selectedGroup == nil)
            }
            .buttonStyle(.borderless)
            .padding(8)
            Divider()
            List(selection: $model.selectedGroupSelection) {
                Text("All Entries (\(model.entries.count))")
                    .tag(GroupSelection.allEntries)
                ForEach(model.groups.compactMap { groupView($0) }, id: \.path) { group in
                    Text("\(group.name) (\(group.count))")
                        .padding(.leading, CGFloat(group.depth * 10))
                        .tag(GroupSelection.group(group.path))
                }
            }
            .listStyle(.sidebar)
        }
    }

    private func groupView(_ group: GroupSummary) -> (path: String, name: String, depth: Int, count: Int)? {
        let path = normalizeGroupPath(group.path)
        guard !path.isEmpty else { return nil }
        let name = group.name.isEmpty ? path.split(separator: "/").last.map(String.init) ?? path : group.name
        let depth = max(path.split(separator: "/").count - 1, 0)
        let count = model.entries.filter { entry in
            let entryPath = normalizeGroupPath(entry.group_path)
            return entryPath == path || entryPath.hasPrefix("\(path)/")
        }.count
        return (path, name, depth, count)
    }

    private func normalizeGroupPath(_ path: String) -> String {
        path.replacingOccurrences(of: "^Root/?", with: "", options: .regularExpression)
            .trimmingCharacters(in: CharacterSet(charactersIn: "/"))
    }
}
struct EntryListView: View {
    @EnvironmentObject private var model: AppModel
    @FocusState private var searchFocused: Bool
    @State private var listFocused = false

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                ZStack(alignment: .trailing) {
                    TextField("Search entries", text: $model.searchQuery)
                        .textFieldStyle(.roundedBorder)
                        .focused($searchFocused)
                        .onSubmit { model.search() }
                        .onChange(of: model.searchQuery) { _ in model.updateSearchResults() }
                    if !model.searchQuery.isEmpty {
                        Button {
                            model.resetList()
                        } label: {
                            Image(systemName: "xmark.circle.fill")
                                .foregroundStyle(.secondary)
                        }
                        .buttonStyle(.plain)
                        .help("Clear search")
                        .padding(.trailing, 6)
                    }
                }
                Button("Search") { model.search() }
            }
            .padding(8)
            Divider()
            HStack(spacing: 8) {
                Button("＋") { model.prepareAddEntry() }
                Spacer()
            }
            .buttonStyle(.borderless)
            .padding(8)
            Divider()
            ScrollView {
                VStack(spacing: 0) {
                    ForEach(model.filteredEntries) { entry in
                        entryRow(entry)
                    }
                }
            }
            .focusable(true)
            .background(
                KeyboardCaptureView(active: $listFocused) { event in
                    switch event.keyCode {
                    case KeyCode.arrowUp:
                        model.selectAdjacentEntry(delta: -1)
                        return true
                    case KeyCode.arrowDown:
                        model.selectAdjacentEntry(delta: 1)
                        return true
                    case KeyCode.return, KeyCode.keypadEnter:
                        model.openSelectedEntryDetail()
                        return true
                    default:
                        return false
                    }
                }
                .frame(width: 0, height: 0)
            )
        }
        .onAppear { listFocused = true }
        .onChange(of: model.searchFocusRequest) { _ in
            listFocused = false
            searchFocused = true
        }
        .onChange(of: searchFocused) { focused in
            if focused {
                listFocused = false
            }
        }
        .sheet(isPresented: $model.showAddEntrySheet) {
            AddEntrySheet()
                .environmentObject(model)
        }
        .sheet(isPresented: $model.showEditEntrySheet) {
            EditEntrySheet()
                .environmentObject(model)
        }
    }

    private func entryRow(_ entry: EntrySummary) -> some View {
        VStack(spacing: 0) {
            Button {
                model.selectEntry(entry.id)
                listFocused = true
            } label: {
                VStack(alignment: .leading, spacing: 3) {
                    Text(entry.title ?? "<untitled>")
                        .fontWeight(model.selectedEntryID == entry.id ? .semibold : .regular)
                    Text("\(entry.group_path) · \(entry.username ?? "") · \(entry.url ?? "")")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }
                .padding(.horizontal, 10)
                .padding(.vertical, 8)
                .frame(maxWidth: .infinity, alignment: .leading)
                .contentShape(Rectangle())
                .background(entryBackground(entry))
            }
            .buttonStyle(.plain)
            Divider()
        }
    }

    private func entryBackground(_ entry: EntrySummary) -> Color {
        if model.focusedEntryID == entry.id {
            return Color.accentColor.opacity(0.22)
        }
        if model.selectedEntryID == entry.id {
            return Color.accentColor.opacity(0.12)
        }
        return Color.clear
    }
}

struct AddEntrySheet: View {
    @EnvironmentObject private var model: AppModel
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("New Entry")
                .font(.title2)
                .fontWeight(.semibold)
            TextField("Group path", text: $model.newEntryGroup)
            TextField("Title", text: $model.newEntryTitle)
            TextField("Username", text: $model.newEntryUsername)
            SecureField("Password", text: $model.newEntryPassword)
            TextField("URL", text: $model.newEntryURL)
            TextField("Notes", text: $model.newEntryNotes)
            HStack {
                Button("Save") { model.saveNewEntry() }
                    .keyboardShortcut(.return, modifiers: [])
                Button("Cancel") { dismiss() }
            }
        }
        .textFieldStyle(.roundedBorder)
        .padding(20)
        .frame(width: 460)
    }
}

struct EditEntrySheet: View {
    @EnvironmentObject private var model: AppModel
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Edit Entry")
                .font(.title2)
                .fontWeight(.semibold)
            Text("Leave password blank to keep the current password.")
                .foregroundStyle(.secondary)
            TextField("Group path", text: $model.editEntryGroup)
            TextField("Title", text: $model.editEntryTitle)
            TextField("Username", text: $model.editEntryUsername)
            SecureField("Password", text: $model.editEntryPassword)
            TextField("URL", text: $model.editEntryURL)
            TextField("Notes", text: $model.editEntryNotes)
            HStack {
                Button("Save") { model.saveEditedEntry() }
                    .keyboardShortcut(.return, modifiers: [])
                Button("Cancel") { dismiss() }
            }
        }
        .textFieldStyle(.roundedBorder)
        .padding(20)
        .frame(width: 460)
    }
}


struct AuditResultsWindow: View {
    @EnvironmentObject private var model: AppModel
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack {
                VStack(alignment: .leading, spacing: 2) {
                    Text("Audit Results")
                        .font(.title2)
                        .fontWeight(.semibold)
                    Text("Vault-level findings for this unlocked vault.")
                        .foregroundStyle(.secondary)
                }
                Spacer()
                Button("Close") { dismiss() }
                    .keyboardShortcut(.cancelAction)
            }
            .padding(16)
            Divider()

            if model.auditFindings.isEmpty {
                VStack(alignment: .leading, spacing: 8) {
                    Image(systemName: "checkmark.shield")
                        .font(.largeTitle)
                        .foregroundStyle(.secondary)
                    Text("No audit findings.")
                        .font(.headline)
                }
                .padding(20)
                Spacer()
            } else {
                ScrollView {
                    VStack(spacing: 0) {
                        ForEach(model.auditFindings) { finding in
                            VStack(alignment: .leading, spacing: 5) {
                                HStack {
                                    Text(finding.kind)
                                        .font(.caption)
                                        .fontWeight(.bold)
                                        .textCase(.uppercase)
                                        .foregroundStyle(.secondary)
                                    Spacer()
                                    Text(finding.group_path)
                                        .font(.caption)
                                        .foregroundStyle(.secondary)
                                }
                                Text(finding.title ?? finding.entry_id)
                                    .fontWeight(.semibold)
                                Text(finding.message)
                            }
                            .padding(.horizontal, 16)
                            .padding(.vertical, 10)
                            Divider()
                        }
                    }
                }
            }
        }
        .frame(minWidth: 620, minHeight: 420)
    }
}


struct DetailAttributeRow<Actions: View>: View {
    let label: String
    let value: String
    let copy: () -> Void
    let actions: Actions
    @State private var focused = false

    init(label: String, value: String, copy: @escaping () -> Void, @ViewBuilder actions: () -> Actions) {
        self.label = label
        self.value = value
        self.copy = copy
        self.actions = actions()
    }

    var body: some View {
        VStack(spacing: 0) {
            HStack(alignment: .firstTextBaseline, spacing: 10) {
                Text(label)
                    .fontWeight(.semibold)
                    .foregroundStyle(.secondary)
                    .frame(width: 90, alignment: .leading)
                Text(value)
                    .textSelection(.enabled)
                    .frame(maxWidth: .infinity, alignment: .leading)
                HStack(spacing: 4) {
                    actions
                }
            }
            .padding(.vertical, 5)
            .padding(.horizontal, 4)
            .background(focused ? Color.accentColor.opacity(0.14) : Color.clear)
            .contentShape(Rectangle())
            .focusable(true)
            .onTapGesture(count: 1) { focused = true }
            .onTapGesture(count: 2) { copy() }
            .background(
                KeyboardCaptureView(active: $focused) { event in
                    switch event.keyCode {
                    case KeyCode.return, KeyCode.keypadEnter:
                        copy()
                        return true
                    default:
                        return false
                    }
                }
                .frame(width: 0, height: 0)
            )
            Divider()
        }
    }
}

struct EntryDetailView: View {
    @EnvironmentObject private var model: AppModel

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            if let detail = model.selectedDetail {
                HStack {
                    Text(detail.title ?? "<untitled>")
                        .font(.title2)
                        .fontWeight(.semibold)
                    Spacer()
                    Button("✎") { model.prepareEditEntry() }
                        .buttonStyle(.borderless)
                        .help("Edit entry")
                    Button("⌫") { model.deleteSelectedEntry() }
                        .buttonStyle(.borderless)
                        .help("Delete entry")
                }
                copyableDetailRow("ID", detail.id) { copyToClipboard(detail.id, label: "ID") }
                copyableDetailRow("Group", detail.group_path) { copyToClipboard(detail.group_path, label: "group") }
                copyableDetailRow("Username", detail.username ?? "") {
                    model.copyUsername()
                } actions: {
                    inlineAction("⧉", "Copy username") { model.copyUsername() }
                        .disabled((detail.username ?? "").isEmpty)
                }
                copyableDetailRow("Password", model.detailRevealed ? (detail.password ?? "") : "<hidden>") {
                    model.copyPassword()
                } actions: {
                    inlineAction("⧉", "Copy password") { model.copyPassword() }
                    inlineAction(model.detailRevealed ? "🙈" : "👁", model.detailRevealed ? "Hide password" : "Reveal password") {
                        model.toggleRevealPassword()
                    }
                }
                copyableDetailRow("URL", detail.url ?? "") {
                    model.copyURL()
                } actions: {
                    inlineAction("⧉", "Copy URL") { model.copyURL() }
                        .disabled((detail.url ?? "").isEmpty)
                }
                copyableDetailRow("TOTP", detail.has_totp ? "one-time code available" : "No TOTP code available") {
                    model.copyTotp()
                } actions: {
                    inlineAction("⧉", "Copy TOTP") { model.copyTotp() }
                        .disabled(!detail.has_totp)
                }
                copyableDetailRow("Notes", detail.notes ?? "") { copyToClipboard(detail.notes ?? "", label: "notes") }
                if !detail.custom_fields.isEmpty {
                    Divider()
                    Text("Custom fields")
                        .font(.headline)
                    ForEach(detail.custom_fields) { field in
                        copyableDetailRow(field.key, field.value) { copyToClipboard(field.value, label: field.key) }
                    }
                }
            } else {
                VStack(alignment: .leading, spacing: 8) {
                    Image(systemName: "key")
                        .font(.largeTitle)
                        .foregroundStyle(.secondary)
                    Text("Select an entry")
                        .font(.title2)
                        .fontWeight(.semibold)
                    Text("Unlock a vault and select an entry to view safe details.")
                        .foregroundStyle(.secondary)
                }
            }
            Spacer()
        }
        .padding(16)
        .navigationTitle("Detail")
    }

    private func copyableDetailRow(_ label: String, _ value: String, copy: @escaping () -> Void) -> some View {
        copyableDetailRow(label, value, copy: copy) { EmptyView() }
    }

    private func copyableDetailRow<Actions: View>(
        _ label: String,
        _ value: String,
        copy: @escaping () -> Void,
        @ViewBuilder actions: () -> Actions
    ) -> some View {
        DetailAttributeRow(label: label, value: value, copy: copy, actions: actions)
    }

    private func copyToClipboard(_ value: String, label: String) {
        guard !value.isEmpty else { return }
        model.copyValue(value, label: label)
    }

    private func detailRow(_ label: String, _ value: String) -> some View {
        detailRow(label, value) { EmptyView() }
    }

    private func detailRow<Actions: View>(
        _ label: String,
        _ value: String,
        @ViewBuilder actions: () -> Actions
    ) -> some View {
        VStack(spacing: 0) {
            HStack(alignment: .firstTextBaseline, spacing: 10) {
                Text(label)
                    .fontWeight(.semibold)
                    .foregroundStyle(.secondary)
                    .frame(width: 90, alignment: .leading)
                Text(value)
                    .textSelection(.enabled)
                    .frame(maxWidth: .infinity, alignment: .leading)
                HStack(spacing: 4) {
                    actions()
                }
            }
            .padding(.vertical, 5)
            Divider()
        }
    }

    private func inlineAction(_ title: String, _ help: String, action: @escaping () -> Void) -> some View {
        Button(title, action: action)
            .buttonStyle(.borderless)
            .help(help)
    }
}
