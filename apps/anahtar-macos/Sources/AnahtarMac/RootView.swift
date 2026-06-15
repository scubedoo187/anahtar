import SwiftUI

struct RootView: View {
    @EnvironmentObject private var model: AppModel

    var body: some View {
        VStack(spacing: 0) {
            topBar
            Divider()
            if model.unlocked {
                splitView
            } else {
                UnlockView()
            }
        }
        .overlay(alignment: .topTrailing) {
            if let toast = model.toastMessage {
                ToastView(message: toast)
                    .padding(.top, 46)
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

    private var topBar: some View {
        HStack(spacing: 12) {
            Text("Anahtar")
                .font(.headline)
            Spacer()
            if model.unlocked {
                Button("Audit") { model.runAudit() }
                Button("Refresh") { model.refresh() }
                Button("Lock") { model.lockVault() }
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
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
        List(selection: $model.selectedGroupSelection) {
            Text("All Entries (\(model.entries.count))")
                .tag(AppModel.allGroupsSelection)
            ForEach(model.groups.compactMap { groupView($0) }, id: \.path) { group in
                Text("\(group.name) (\(group.count))")
                    .padding(.leading, CGFloat(group.depth * 10))
                    .tag(group.path)
            }
        }
        .listStyle(.sidebar)
        .navigationTitle("Groups")
        .toolbar {
            Button("＋") { model.addGroupPrompt() }
            Button("✎") { model.renameSelectedGroupPrompt() }
                .disabled(model.selectedGroup == nil)
            Button("⌫") { model.deleteSelectedGroup() }
                .disabled(model.selectedGroup == nil)
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

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                TextField("Search entries", text: $model.searchQuery)
                    .textFieldStyle(.roundedBorder)
                    .onSubmit { model.search() }
                    .onChange(of: model.searchQuery) { _ in model.updateSearchResults() }
                Button("Search") { model.search() }
                Button("Reset") { model.resetList() }
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
                model.selectedEntryID = entry.id
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
                .background(model.selectedEntryID == entry.id ? Color.accentColor.opacity(0.18) : Color.clear)
            }
            .buttonStyle(.plain)
            Divider()
        }
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
                detailRow("ID", detail.id)
                detailRow("Group", detail.group_path)
                detailRow("Username", detail.username ?? "") {
                    inlineAction("⧉", "Copy username") { model.copyUsername() }
                        .disabled((detail.username ?? "").isEmpty)
                }
                detailRow("Password", model.detailRevealed ? (detail.password ?? "") : "<hidden>") {
                    inlineAction("⧉", "Copy password") { model.copyPassword() }
                    inlineAction(model.detailRevealed ? "🙈" : "👁", model.detailRevealed ? "Hide password" : "Reveal password") {
                        model.toggleRevealPassword()
                    }
                }
                detailRow("URL", detail.url ?? "") {
                    inlineAction("⧉", "Copy URL") { model.copyURL() }
                        .disabled((detail.url ?? "").isEmpty)
                }
                detailRow("TOTP", detail.has_totp ? "one-time code available" : "No TOTP code available") {
                    inlineAction("⧉", "Copy TOTP") { model.copyTotp() }
                        .disabled(!detail.has_totp)
                }
                detailRow("Notes", detail.notes ?? "")
                if !detail.custom_fields.isEmpty {
                    Divider()
                    Text("Custom fields")
                        .font(.headline)
                    ForEach(detail.custom_fields) { field in
                        detailRow(field.key, field.value)
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
