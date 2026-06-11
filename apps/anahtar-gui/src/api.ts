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

export function versionLabel(version: KdbxVersion): string {
  if ("kdbx" in version) {
    return `KDBX ${version.kdbx.major}.${version.kdbx.minor}`;
  }
  return `KDB ${version.kdb1.major}.${version.kdb1.minor}`;
}

function emptyToNull(value?: string | null): string | null {
  const trimmed = value?.trim() ?? "";
  return trimmed.length > 0 ? trimmed : null;
}
