import Foundation

@_silgen_name("anahtar_backend_status_json")
private func anahtar_backend_status_json() -> UnsafeMutablePointer<CChar>?

@_silgen_name("anahtar_unlock_vault_json")
private func anahtar_unlock_vault_json(_ request: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?

@_silgen_name("anahtar_search_entries_json")
private func anahtar_search_entries_json(_ request: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?

@_silgen_name("anahtar_show_entry_json")
private func anahtar_show_entry_json(_ request: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?

@_silgen_name("anahtar_totp_code_json")
private func anahtar_totp_code_json(_ request: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?

@_silgen_name("anahtar_list_groups_json")
private func anahtar_list_groups_json(_ request: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?


@_silgen_name("anahtar_audit_vault_json")
private func anahtar_audit_vault_json(_ request: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?

@_silgen_name("anahtar_add_entry_json")
private func anahtar_add_entry_json(_ request: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?

@_silgen_name("anahtar_edit_entry_json")
private func anahtar_edit_entry_json(_ request: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?

@_silgen_name("anahtar_delete_entry_json")
private func anahtar_delete_entry_json(_ request: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?

@_silgen_name("anahtar_add_group_json")
private func anahtar_add_group_json(_ request: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?

@_silgen_name("anahtar_rename_group_json")
private func anahtar_rename_group_json(_ request: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?

@_silgen_name("anahtar_delete_group_json")
private func anahtar_delete_group_json(_ request: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?

@_silgen_name("anahtar_move_entry_json")
private func anahtar_move_entry_json(_ request: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?

@_silgen_name("anahtar_string_free")
private func anahtar_string_free(_ ptr: UnsafeMutablePointer<CChar>?)

struct BackendStatus: Decodable {
    let status: String
    let version: String
    let service: String
}

struct VaultRequest: Codable {
    let path: String
    let password: String
    let key_file: String?
}

struct SearchRequest: Codable {
    let path: String
    let password: String
    let key_file: String?
    let query: String
}

struct ShowEntryRequest: Codable {
    let path: String
    let password: String
    let key_file: String?
    let selector_kind: String
    let selector_value: String
    let reveal_password: Bool
}

struct TotpRequest: Codable {
    let path: String
    let password: String
    let key_file: String?
    let selector_kind: String
    let selector_value: String
}


struct AddEntryInput: Codable {
    let group_path: String
    let title: String
    let username: String?
    let password: String?
    let url: String?
    let notes: String?
}

struct EditEntryInput: Codable {
    let title: String?
    let username: String?
    let password: String?
    let url: String?
    let notes: String?
}

struct AddEntryFfiRequest: Codable {
    let path: String
    let password: String
    let key_file: String?
    let entry: AddEntryInput
    let backup_dir: String?
}

struct EditEntryFfiRequest: Codable {
    let path: String
    let password: String
    let key_file: String?
    let entry_id: String
    let entry: EditEntryInput
    let backup_dir: String?
}

struct EntryIdFfiRequest: Codable {
    let path: String
    let password: String
    let key_file: String?
    let entry_id: String
    let backup_dir: String?
}

struct GroupFfiRequest: Codable {
    let path: String
    let password: String
    let key_file: String?
    let group_path: String
    let backup_dir: String?
}

struct RenameGroupFfiRequest: Codable {
    let path: String
    let password: String
    let key_file: String?
    let group_path: String
    let new_name: String
    let backup_dir: String?
}

struct MoveEntryFfiRequest: Codable {
    let path: String
    let password: String
    let key_file: String?
    let entry_id: String
    let group_path: String
    let backup_dir: String?
}

struct EntrySummary: Decodable, Identifiable, Hashable {
    let id: String
    let group_path: String
    let title: String?
    let username: String?
    let url: String?
}

struct EntryDetail: Decodable, Identifiable {
    let id: String
    let group_path: String
    let title: String?
    let username: String?
    let url: String?
    let notes: String?
    let has_totp: Bool
    let password: String?
    let custom_fields: [CustomField]
}

struct CustomField: Decodable, Identifiable {
    var id: String { key }
    let key: String
    let value: String
    let protected: Bool
}

struct GroupSummary: Decodable, Identifiable, Hashable {
    let id: String
    let path: String
    let name: String
    let entry_count: Int
    let child_group_count: Int
}

struct TotpCode: Decodable {
    let code: String
    let valid_for_seconds: UInt64
    let period_seconds: UInt64
}


struct WriteReport: Decodable {
    let operation: String
    let backup_path: String?
    let final_target_path: String?
    let changed_entry_id: String?
}

struct AuditFinding: Decodable, Identifiable {
    var id: String { "\(kind)-\(entry_id)-\(message)" }
    let kind: String
    let entry_id: String
    let title: String?
    let group_path: String
    let message: String
}

struct AuditReport: Decodable {
    let findings: [AuditFinding]
}

struct BackendError: Decodable, Error {
    let kind: String
    let message: String
}

private struct FfiResponse<T: Decodable>: Decodable {
    let ok: Bool
    let data: T?
    let error: BackendError?
}

enum BackendBridgeError: LocalizedError {
    case nullResponse
    case backend(BackendError)
    case missingData

    var errorDescription: String? {
        switch self {
        case .nullResponse:
            return "The Rust backend returned no response."
        case .backend(let error):
            if error.kind == "unlock_failed" {
                return "We couldn't unlock this vault. Check the password, key file, and selected vault."
            }
            return error.message
        case .missingData:
            return "The Rust backend response was incomplete."
        }
    }
}

final class BackendBridge {
    private let encoder = JSONEncoder()
    private let decoder = JSONDecoder()

    func backendStatus() throws -> BackendStatus {
        try decodeResponse(anahtar_backend_status_json())
    }

    func unlockVault(_ request: VaultRequest) throws -> [EntrySummary] {
        try call(request, anahtar_unlock_vault_json)
    }

    func searchEntries(_ request: SearchRequest) throws -> [EntrySummary] {
        try call(request, anahtar_search_entries_json)
    }

    func showEntry(_ request: ShowEntryRequest) throws -> EntryDetail {
        try call(request, anahtar_show_entry_json)
    }

    func totpCode(_ request: TotpRequest) throws -> TotpCode {
        try call(request, anahtar_totp_code_json)
    }


    func auditVault(_ request: VaultRequest) throws -> AuditReport {
        try call(request, anahtar_audit_vault_json)
    }

    func addEntry(_ request: AddEntryFfiRequest) throws -> WriteReport {
        try call(request, anahtar_add_entry_json)
    }

    func editEntry(_ request: EditEntryFfiRequest) throws -> WriteReport {
        try call(request, anahtar_edit_entry_json)
    }

    func deleteEntry(_ request: EntryIdFfiRequest) throws -> WriteReport {
        try call(request, anahtar_delete_entry_json)
    }

    func addGroup(_ request: GroupFfiRequest) throws -> WriteReport {
        try call(request, anahtar_add_group_json)
    }

    func renameGroup(_ request: RenameGroupFfiRequest) throws -> WriteReport {
        try call(request, anahtar_rename_group_json)
    }

    func deleteGroup(_ request: GroupFfiRequest) throws -> WriteReport {
        try call(request, anahtar_delete_group_json)
    }

    func moveEntry(_ request: MoveEntryFfiRequest) throws -> WriteReport {
        try call(request, anahtar_move_entry_json)
    }

    func listGroups(_ request: VaultRequest) throws -> [GroupSummary] {
        try call(request, anahtar_list_groups_json)
    }

    private func call<Request: Encodable, Response: Decodable>(
        _ request: Request,
        _ function: (UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?
    ) throws -> Response {
        let data = try encoder.encode(request)
        let json = String(decoding: data, as: UTF8.self)
        return try json.withCString { cString in
            try decodeResponse(function(cString))
        }
    }

    private func decodeResponse<T: Decodable>(_ ptr: UnsafeMutablePointer<CChar>?) throws -> T {
        guard let ptr else {
            throw BackendBridgeError.nullResponse
        }
        defer { anahtar_string_free(ptr) }

        let json = String(cString: ptr)
        let data = Data(json.utf8)
        let response = try decoder.decode(FfiResponse<T>.self, from: data)
        if response.ok, let data = response.data {
            return data
        }
        if let error = response.error {
            throw BackendBridgeError.backend(error)
        }
        throw BackendBridgeError.missingData
    }
}
