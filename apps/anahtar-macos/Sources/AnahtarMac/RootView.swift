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
    }

    private var topBar: some View {
        HStack(spacing: 12) {
            Text("Anahtar")
                .font(.headline)
            Text(model.statusMessage)
                .foregroundStyle(.secondary)
                .lineLimit(1)
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
        List(selection: $model.selectedGroup) {
            Text("All Entries (\(model.entries.count))")
                .tag(String?.none)
            ForEach(model.groups.compactMap { groupView($0) }, id: \.path) { group in
                Text("\(group.name) (\(group.count))")
                    .padding(.leading, CGFloat(group.depth * 10))
                    .tag(Optional(group.path))
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
                Button("✎") { model.prepareEditEntry() }
                    .disabled(model.selectedEntryID == nil)
                Button("⌫") { model.deleteSelectedEntry() }
                    .disabled(model.selectedEntryID == nil)
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
        .navigationTitle("Entries")
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

struct EntryDetailView: View {
    @EnvironmentObject private var model: AppModel

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            if let detail = model.selectedDetail {
                Text(detail.title ?? "<untitled>")
                    .font(.title2)
                    .fontWeight(.semibold)
                LabeledContent("ID", value: detail.id)
                LabeledContent("Group", value: detail.group_path)
                LabeledContent("Username", value: detail.username ?? "")
                LabeledContent("Password", value: model.detailRevealed ? (detail.password ?? "") : "<hidden>")
                LabeledContent("URL", value: detail.url ?? "")
                LabeledContent("TOTP", value: detail.has_totp ? "one-time code available" : "No TOTP code available")
                LabeledContent("Notes", value: detail.notes ?? "")
                HStack {
                    Button("Copy Username") { model.copyUsername() }
                        .disabled((detail.username ?? "").isEmpty)
                    Button("Copy Password") { model.copyPassword() }
                    Button(model.detailRevealed ? "Hide Password" : "Reveal Password") {
                        model.toggleRevealPassword()
                    }
                    Button("Copy URL") { model.copyURL() }
                        .disabled((detail.url ?? "").isEmpty)
                    Button("Copy TOTP") { model.copyTotp() }
                        .disabled(!detail.has_totp)
                }
                if !model.auditFindings.isEmpty {
                    Divider()
                    Text("Audit findings")
                        .font(.headline)
                    ForEach(model.auditFindings.prefix(8)) { finding in
                        Text("\(finding.kind): \(finding.title ?? finding.entry_id) — \(finding.message)")
                            .font(.caption)
                    }
                }
                if !detail.custom_fields.isEmpty {
                    Divider()
                    Text("Custom fields")
                        .font(.headline)
                    ForEach(detail.custom_fields) { field in
                        LabeledContent(field.key, value: field.value)
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
}
