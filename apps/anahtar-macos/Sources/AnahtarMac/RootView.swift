import SwiftUI

struct RootView: View {
    @EnvironmentObject private var model: AppModel

    var body: some View {
        VStack(spacing: 0) {
            topBar
            Divider()
            NavigationSplitView {
                GroupListView()
                    .navigationSplitViewColumnWidth(min: 180, ideal: 220)
            } content: {
                EntryListView()
                    .navigationSplitViewColumnWidth(min: 240, ideal: 300)
            } detail: {
                EntryDetailView()
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
            Button("Open Vault…") { model.openVault() }
            Button("Lock") { model.lockVault() }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
    }
}

struct GroupListView: View {
    @EnvironmentObject private var model: AppModel

    var body: some View {
        List(selection: $model.selectedGroup) {
            ForEach(model.placeholderGroups, id: \.self) { group in
                Text(group)
                    .tag(group == "All Entries" ? nil : Optional(group))
            }
        }
        .navigationTitle("Groups")
    }
}

struct EntryListView: View {
    @EnvironmentObject private var model: AppModel

    var body: some View {
        List(selection: $model.selectedEntryID) {
            ForEach(model.placeholderEntries, id: \.self) { entry in
                VStack(alignment: .leading) {
                    Text(entry)
                    Text(model.selectedGroup ?? "All Entries")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                .tag(Optional(entry))
            }
        }
        .navigationTitle("Entries")
        .searchable(text: .constant(""), prompt: "Search entries")
    }
}

struct EntryDetailView: View {
    @EnvironmentObject private var model: AppModel

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            if let selectedEntryID = model.selectedEntryID {
                Text(selectedEntryID)
                    .font(.title2)
                    .fontWeight(.semibold)
                LabeledContent("Group", value: model.selectedGroup ?? "General/Web")
                LabeledContent("Username", value: "Hidden until Rust bridge is connected")
                LabeledContent("Password", value: "<hidden>")
                HStack {
                    Button("Copy Username") {}
                    Button("Reveal Password") {}
                    Button("Copy TOTP") {}
                        .disabled(true)
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
