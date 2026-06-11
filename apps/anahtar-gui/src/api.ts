import { invoke } from "@tauri-apps/api/core";

export type BackendStatus = {
  app: string;
  version: string;
  service: string;
};

export type KdbxVersion =
  | { kdb1: { major: number; minor: number } }
  | { kdbx: { major: number; minor: number } };

export type VaultInfo = {
  path: string;
  file_size_bytes: number;
  version: KdbxVersion;
};

export type EntrySummary = {
  id: string;
  group_path: string;
  title?: string | null;
  username?: string | null;
  url?: string | null;
};

export type CustomField = {
  key: string;
  value: string;
  protected: boolean;
};

export type EntryDetail = EntrySummary & {
  notes?: string | null;
  password?: string | null;
  custom_fields: CustomField[];
};

export type TotpCode = {
  code: string;
  valid_for_seconds: number;
  period_seconds: number;
};

export type WriteOperation =
  | "upgrade"
  | "add"
  | "edit"
  | "delete"
  | "group_add"
  | "group_rename"
  | "group_delete"
  | "move_entry";

export type WriteReport = {
  operation: WriteOperation;
  input_path: string;
  output_path: string;
  input_version: KdbxVersion;
  output_version: KdbxVersion;
  input_group_count: number;
  input_entry_count: number;
  output_group_count: number;
  output_entry_count: number;
  changed_entry_id?: string | null;
  backup_path?: string | null;
  final_target_path?: string | null;
};

export type AddEntryRequest = {
  group_path: string;
  title: string;
  username?: string | null;
  password?: string | null;
  url?: string | null;
  notes?: string | null;
  backup_dir?: string | null;
};

export type EditEntryRequest = {
  title?: string | null;
  username?: string | null;
  password?: string | null;
  url?: string | null;
  notes?: string | null;
  backup_dir?: string | null;
};

export type SelectorKind = "id" | "title" | "url" | "username" | "auto";

export type VaultRequest = {
  path: string;
  password: string;
  keyFile?: string | null;
};

export function backendStatus(): Promise<BackendStatus> {
  return invoke<BackendStatus>("backend_status");
}

export function inspectVault(path: string): Promise<VaultInfo> {
  return invoke<VaultInfo>("inspect_vault", { path });
}

export function unlockVault(request: VaultRequest): Promise<EntrySummary[]> {
  return invoke<EntrySummary[]>("unlock_vault", {
    path: request.path,
    password: request.password,
    keyFile: emptyToNull(request.keyFile),
  });
}

export function searchEntries(
  request: VaultRequest,
  query: string,
): Promise<EntrySummary[]> {
  return invoke<EntrySummary[]>("search_entries", {
    path: request.path,
    password: request.password,
    keyFile: emptyToNull(request.keyFile),
    query,
  });
}

export function showEntry(
  request: VaultRequest,
  selectorKind: SelectorKind,
  selectorValue: string,
  revealPassword: boolean,
): Promise<EntryDetail> {
  return invoke<EntryDetail>("show_entry", {
    path: request.path,
    password: request.password,
    keyFile: emptyToNull(request.keyFile),
    selectorKind,
    selectorValue,
    revealPassword,
  });
}

export function totpCode(
  request: VaultRequest,
  selectorKind: SelectorKind,
  selectorValue: string,
): Promise<TotpCode> {
  return invoke<TotpCode>("totp_code", {
    path: request.path,
    password: request.password,
    keyFile: emptyToNull(request.keyFile),
    selectorKind,
    selectorValue,
  });
}

export function addEntry(
  request: VaultRequest,
  entry: AddEntryRequest,
): Promise<WriteReport> {
  return invoke<WriteReport>("add_entry", {
    path: request.path,
    password: request.password,
    keyFile: emptyToNull(request.keyFile),
    request: normalizeAddRequest(entry),
  });
}

export function editEntry(
  request: VaultRequest,
  entryId: string,
  entry: EditEntryRequest,
): Promise<WriteReport> {
  return invoke<WriteReport>("edit_entry", {
    path: request.path,
    password: request.password,
    keyFile: emptyToNull(request.keyFile),
    entryId,
    request: normalizeEditRequest(entry),
  });
}

export function deleteEntry(
  request: VaultRequest,
  entryId: string,
  backupDir?: string | null,
): Promise<WriteReport> {
  return invoke<WriteReport>("delete_entry", {
    path: request.path,
    password: request.password,
    keyFile: emptyToNull(request.keyFile),
    entryId,
    backupDir: emptyToNull(backupDir),
  });
}

export function versionLabel(version: KdbxVersion): string {
  if ("kdbx" in version) {
    return `KDBX ${version.kdbx.major}.${version.kdbx.minor}`;
  }
  return `KDB ${version.kdb1.major}.${version.kdb1.minor}`;
}

function normalizeAddRequest(entry: AddEntryRequest): AddEntryRequest {
  return {
    group_path: entry.group_path,
    title: entry.title,
    username: emptyToNull(entry.username),
    password: emptyToNull(entry.password),
    url: emptyToNull(entry.url),
    notes: emptyToNull(entry.notes),
    backup_dir: emptyToNull(entry.backup_dir),
  };
}

function normalizeEditRequest(entry: EditEntryRequest): EditEntryRequest {
  return {
    title: emptyToNull(entry.title),
    username: emptyToNull(entry.username),
    password: emptyToNull(entry.password),
    url: emptyToNull(entry.url),
    notes: emptyToNull(entry.notes),
    backup_dir: emptyToNull(entry.backup_dir),
  };
}

function emptyToNull(value?: string | null): string | null {
  const trimmed = value?.trim() ?? "";
  return trimmed.length > 0 ? trimmed : null;
}
