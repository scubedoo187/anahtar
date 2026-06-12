import { open } from "@tauri-apps/plugin-dialog";
import {
  addEntry,
  addGroup,
  auditVault,
  backendStatus,
  deleteEntry,
  deleteGroup,
  editEntry,
  inspectVault,
  listGroups,
  moveEntry,
  renameGroup,
  searchEntries,
  showEntry,
  totpCode,
  unlockVault,
  versionLabel,
  type AddEntryRequest,
  type EditEntryRequest,
  type VaultRequest,
} from "./api";
import { clearClipboardTimer, copyWithOwnedClear, setClipboardStatus } from "./clipboard";
import {
  bindButton,
  bindForm,
  detailOutputEl,
  inputValue,
  outputEl,
  setInputValue,
} from "./dom";
import { errorMessage, formatError } from "./errors";
import {
  renderAudit,
  renderEmptyDetail,
  filterEntriesForSelectedGroup,
  normalizeGroupPath,
  renderEntryDetail,
  renderEntryList,
  renderGroupTree,
  renderNavigationState,
  renderSessionState,
  renderWriteReport,
} from "./render";
import { createInitialState, clearSelection, requireSelectedDetail, requireSession, type ActiveView } from "./state";
import { defaultVaultPath, renderShell } from "./shell";
import "./styles.css";

const app = document.querySelector<HTMLDivElement>("#app");

if (!app) {
  throw new Error("#app root element is missing");
}

const state = createInitialState();

renderShell(app);
setInputValue("#vault-path", defaultVaultPath);
void refreshBackendStatus();
renderAppChrome();
focusMasterPassword();

bindNavigation();
bindButton("#refresh-status", refreshBackendStatus);
bindButton("#inspect-vault", runInspect);
bindButton("#browse-vault", chooseVaultFile);
bindButton("#browse-key-file", chooseKeyFile);
bindForm("#unlock-form", runUnlock);
bindButton("#lock-vault", lockVault);
bindButton("#search-entries", runSearch);
bindButton("#reset-list", resetList);
bindButton("#reload-detail", reloadSafeDetail);
bindButton("#new-entry", runAddEntry);
bindButton("#edit-selected", runEditEntry);
bindButton("#add-group", runAddGroup);
bindButton("#rename-group", runRenameGroup);
bindButton("#delete-group", runDeleteGroup);
bindButton("#run-audit", runAudit);
bindButton("#delete-entry", runDeleteEntry);

function bindNavigation(): void {
  for (const view of ["browse", "audit", "status"] as ActiveView[]) {
    bindButton(`#nav-${view}`, async () => {
      setActiveView(view);
    });
  }
}

function setActiveView(view: ActiveView): void {
  state.activeView = view;
  renderAppChrome();
}

async function chooseVaultFile(): Promise<void> {
  try {
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "KeePass vault", extensions: ["kdbx", "kdb"] }],
    });
    if (typeof selected === "string") {
      setInputValue("#vault-path", selected);
      authOutputEl().textContent = "Vault selected. Enter the master password to unlock.";
      focusMasterPassword();
    }
  } catch (error) {
    authOutputEl().textContent = `File picker error: ${formatError(error)}`;
  }
}

async function chooseKeyFile(): Promise<void> {
  try {
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "Key file", extensions: ["key", "keyx", "keyfile", "txt"] }],
    });
    if (typeof selected === "string") {
      setInputValue("#key-file", selected);
      authOutputEl().textContent = "Key file selected. Enter the master password to unlock.";
      focusMasterPassword();
    }
  } catch (error) {
    authOutputEl().textContent = `File picker error: ${formatError(error)}`;
  }
}

function renderAppChrome(): void {
  renderSessionState(state);
  renderNavigationState(state);
}

async function refreshBackendStatus(): Promise<void> {
  const statusEl = document.querySelector<HTMLParagraphElement>("#backend-status");
  if (!statusEl) return;

  try {
    const status = await backendStatus();
    statusEl.textContent = `${status.app} ${status.version} — ${status.service}`;
  } catch (error) {
    statusEl.textContent = `Backend unavailable: ${formatError(error)}`;
  }
}

async function runInspect(): Promise<void> {
  await renderCommand(async () => {
    const info = await inspectVault(vaultPath());
    return `Vault: ${info.path}\nSize: ${info.file_size_bytes} bytes\nVersion: ${versionLabel(info.version)}`;
  });
}

async function runUnlock(): Promise<void> {
  await renderAuthAction(async () => {
    const request = formVaultRequest();
    if (!request.path.trim()) {
      throw new Error("vault path is required");
    }
    if (!request.password) {
      throw new Error("master password is required");
    }

    const [entries, groups] = await Promise.all([unlockVault(request), listGroups(request)]);
    state.activeSession = request;
    state.activeEntries = entries;
    state.visibleEntries = entries;
    state.groups = groups;
    state.selectedGroupPath = null;
    state.activeView = "browse";
    clearSelection(state);
    clearPasswordInput();
    renderAppChrome();
    renderGroupTree(state, state.groups, selectGroup);
    renderEntryList(state, filteredEntries(), selectEntry);
    renderEmptyDetail("Select an entry to view details.");
    return `Unlocked ${entries.length} entries. Select an entry from the list to view safe details.`;
  });
}

async function refreshEntriesAfterWrite(changedEntryId?: string | null): Promise<void> {
  const session = requireSession(state);
  const [entries, groups] = await Promise.all([unlockVault(session), listGroups(session)]);
  state.activeEntries = entries;
  state.visibleEntries = entries;
  state.groups = groups;
  state.selectedEntryId = changedEntryId ?? null;
  state.selectedDetail = null;
  state.detailRevealed = false;
  renderGroupTree(state, state.groups, selectGroup);
  renderEntryList(state, filteredEntries(), selectEntry);
  if (state.selectedEntryId) {
    await loadSelectedDetail(false);
  } else {
    renderEmptyDetail("Select an entry to view details.");
  }
  renderAppChrome();
}

async function lockVault(): Promise<void> {
  state.activeSession = null;
  state.activeEntries = [];
  state.visibleEntries = [];
  state.groups = [];
  state.selectedGroupPath = null;
  state.activeView = "browse";
  clearSelection(state);
  clearClipboardTimer();
  clearPasswordInput();
  renderAppChrome();
  renderGroupTree(state, [], selectGroup);
  renderEntryList(state, [], selectEntry);
  renderEmptyDetail("Select an entry to view details.");
  renderAudit(state, null);
  setClipboardStatus("Clipboard idle.", "neutral");
  outputEl().textContent = "Locked. In-memory session cleared.";
}

async function runSearch(): Promise<void> {
  await renderCommand(async () => {
    const session = requireSession(state);
    const entries = await searchEntries(session, inputValue("#search-query"));
    state.visibleEntries = entries;
    clearSelection(state);
    renderEntryList(state, filteredEntries(), selectEntry);
    renderEmptyDetail("Select a search result to view details.");
    renderAppChrome();
    return `Search returned ${entries.length} entries.`;
  });
}

async function resetList(): Promise<void> {
  clearSelection(state);
  state.visibleEntries = state.activeEntries;
  renderEntryList(state, filteredEntries(), selectEntry);
  renderEmptyDetail("Select an entry to view details.");
  outputEl().textContent = `Showing ${state.activeEntries.length} entries from current in-memory list.`;
  renderAppChrome();
}

function selectGroup(groupPath: string | null): void {
  state.selectedGroupPath = groupPath;
  clearSelection(state);
  renderGroupTree(state, state.groups, selectGroup);
  renderEntryList(state, filteredEntries(), selectEntry);
  renderEmptyDetail("Select an entry to view details.");
  renderAppChrome();
}

function selectEntry(entryId: string): void {
  state.selectedEntryId = entryId;
  state.detailRevealed = false;
  renderEntryList(state, filteredEntries(), selectEntry);
  void loadSelectedDetail(false);
}

async function reloadSafeDetail(): Promise<void> {
  await loadSelectedDetail(false);
}

async function revealSelectedDetail(): Promise<void> {
  await loadSelectedDetail(true);
}

async function copySelectedUsername(): Promise<void> {
  const detail = requireSelectedDetail(state);
  await copyWithOwnedClear(detail.username ?? "", "username");
}

async function copySelectedUrl(): Promise<void> {
  const detail = requireSelectedDetail(state);
  await copyWithOwnedClear(detail.url ?? "", "URL");
}

async function copySelectedPassword(): Promise<void> {
  const session = requireSession(state);
  if (!state.selectedEntryId) {
    throw new Error("select an entry first");
  }
  const detail = await showEntry(session, "id", state.selectedEntryId, true);
  await copyWithOwnedClear(detail.password ?? "", "password");
}

async function copySelectedTotp(): Promise<void> {
  const session = requireSession(state);
  if (!state.selectedEntryId) {
    throw new Error("select an entry first");
  }
  const code = await totpCode(session, "id", state.selectedEntryId);
  await copyWithOwnedClear(code.code, `TOTP code valid for ${code.valid_for_seconds}s`);
}

function detailActions() {
  return {
    copyUsername: copySelectedUsername,
    copyPassword: copySelectedPassword,
    copyUrl: copySelectedUrl,
    copyTotp: copySelectedTotp,
    revealPassword: revealSelectedDetail,
  };
}

function filteredEntries() {
  return filterEntriesForSelectedGroup(state, state.visibleEntries);
}

function entryInGroup(entryGroupPath: string, groupPath: string): boolean {
  const entryPath = normalizeGroupPath(entryGroupPath);
  const selected = normalizeGroupPath(groupPath);
  return entryPath === selected || entryPath.startsWith(`${selected}/`);
}

function promptValue(label: string, defaultValue: string): string | null {
  return window.prompt(label, defaultValue);
}

async function loadSelectedDetail(revealPassword: boolean): Promise<void> {
  const session = requireSession(state);
  if (!state.selectedEntryId) {
    throw new Error("select an entry first");
  }

  const detailEl = detailOutputEl();
  detailEl.textContent = "Loading detail…";
  try {
    const detail = await showEntry(session, "id", state.selectedEntryId, revealPassword);
    state.selectedDetail = detail;
    state.detailRevealed = revealPassword;
    renderEntryDetail(detail, revealPassword, detailActions());
    renderAppChrome();
  } catch (error) {
    detailEl.textContent = `Error: ${formatError(error)}`;
  }
}

async function runAddGroup(): Promise<void> {
  await renderWriteAction(async () => {
    const session = requireSession(state);
    const base = state.selectedGroupPath ? `${state.selectedGroupPath}/` : "";
    const groupPath = promptValue("New group path", base);
    if (groupPath === null) return "Add group cancelled.";
    if (!groupPath.trim()) throw new Error("group path is required");
    const report = await addGroup(session, groupPath.trim());
    await refreshEntriesAfterWrite(null);
    state.selectedGroupPath = groupPath.trim();
    renderGroupTree(state, state.groups, selectGroup);
    renderEntryList(state, filteredEntries(), selectEntry);
    return renderWriteReport(report);
  });
}

async function runRenameGroup(): Promise<void> {
  await renderWriteAction(async () => {
    const session = requireSession(state);
    if (!state.selectedGroupPath) throw new Error("select a group first");
    const currentName = state.selectedGroupPath.split("/").pop() ?? state.selectedGroupPath;
    const newName = promptValue("New group name", currentName);
    if (newName === null) return "Rename group cancelled.";
    if (!newName.trim()) throw new Error("new group name is required");
    const oldPath = state.selectedGroupPath;
    const report = await renameGroup(session, oldPath, newName.trim());
    await refreshEntriesAfterWrite(null);
    const parent = oldPath.split("/").slice(0, -1).join("/");
    state.selectedGroupPath = parent ? `${parent}/${newName.trim()}` : newName.trim();
    renderGroupTree(state, state.groups, selectGroup);
    renderEntryList(state, filteredEntries(), selectEntry);
    return renderWriteReport(report);
  });
}

async function runDeleteGroup(): Promise<void> {
  await renderWriteAction(async () => {
    const session = requireSession(state);
    const groupPath = state.selectedGroupPath;
    if (!groupPath) throw new Error("select a group first");
    const count = state.activeEntries.filter((entry) => entryInGroup(entry.group_path, groupPath)).length;
    if (count > 0) {
      throw new Error(`cannot delete group with ${count} entries in it or its child groups`);
    }
    if (!window.confirm(`Delete empty group "${groupPath}"?`)) {
      return "Delete group cancelled.";
    }
    const report = await deleteGroup(session, groupPath);
    state.selectedGroupPath = null;
    await refreshEntriesAfterWrite(null);
    return renderWriteReport(report);
  });
}

async function runAudit(): Promise<void> {
  await renderCommand(async () => {
    const report = await auditVault(requireSession(state));
    renderAudit(state, report);
    return `Audit found ${report.findings.length} findings.`;
  });
}

async function runAddEntry(): Promise<void> {
  await renderWriteAction(async () => {
    const session = requireSession(state);
    const groupPath = promptValue("Group path", state.selectedGroupPath ?? "General/Web");
    if (groupPath === null) return "Add entry cancelled.";
    const title = promptValue("Title", "");
    if (title === null) return "Add entry cancelled.";
    const username = promptValue("Username", "");
    if (username === null) return "Add entry cancelled.";
    const password = promptValue("Password", "");
    if (password === null) return "Add entry cancelled.";
    const url = promptValue("URL", "");
    if (url === null) return "Add entry cancelled.";
    const notes = promptValue("Notes", "");
    if (notes === null) return "Add entry cancelled.";

    const request: AddEntryRequest = {
      group_path: groupPath,
      title,
      username,
      password,
      url,
      notes,
      backup_dir: null,
    };
    if (!request.group_path.trim() || !request.title.trim()) {
      throw new Error("group path and title are required");
    }
    const report = await addEntry(session, request);
    await refreshEntriesAfterWrite(report.changed_entry_id);
    return renderWriteReport(report);
  });
}

async function runEditEntry(): Promise<void> {
  await renderWriteAction(async () => {
    const session = requireSession(state);
    const detail = requireSelectedDetail(state);
    if (!state.selectedEntryId) {
      throw new Error("select an entry first");
    }

    const groupPath = promptValue("Group path", detail.group_path);
    if (groupPath === null) return "Edit entry cancelled.";
    const title = promptValue("Title", detail.title ?? "");
    if (title === null) return "Edit entry cancelled.";
    const username = promptValue("Username", detail.username ?? "");
    if (username === null) return "Edit entry cancelled.";
    const password = promptValue("Password (leave blank to keep current password)", "");
    if (password === null) return "Edit entry cancelled.";
    const url = promptValue("URL", detail.url ?? "");
    if (url === null) return "Edit entry cancelled.";
    const notes = promptValue("Notes", detail.notes ?? "");
    if (notes === null) return "Edit entry cancelled.";

    const request: EditEntryRequest = { title, username, password, url, notes, backup_dir: null };
    let report = await editEntry(session, state.selectedEntryId, request);
    if (groupPath.trim() && groupPath.trim() !== detail.group_path) {
      report = await moveEntry(session, state.selectedEntryId, groupPath.trim());
    }
    await refreshEntriesAfterWrite(report.changed_entry_id ?? state.selectedEntryId);
    return renderWriteReport(report);
  });
}

async function runDeleteEntry(): Promise<void> {
  await renderWriteAction(async () => {
    const session = requireSession(state);
    if (!state.selectedEntryId) {
      throw new Error("select an entry first");
    }
    const detailTitle = state.selectedDetail?.title ?? state.selectedEntryId;
    const confirmed = window.confirm(
      `Delete "${detailTitle}"? A backup will be created before the vault is replaced.`,
    );
    if (!confirmed) {
      return "Delete cancelled.";
    }
    const report = await deleteEntry(session, state.selectedEntryId, null);
    await refreshEntriesAfterWrite(null);
    return renderWriteReport(report);
  });
}

async function renderAuthAction(action: () => Promise<string>): Promise<void> {
  const output = authOutputEl();
  output.textContent = "Unlocking…";
  try {
    output.textContent = await action();
  } catch (error) {
    output.textContent = `Error: ${formatError(error)}`;
  }
}

async function renderCommand(action: () => Promise<string>): Promise<void> {
  const output = outputEl();
  output.textContent = "Running…";
  try {
    output.textContent = await action();
  } catch (error) {
    output.textContent = `Error: ${formatError(error)}`;
  }
}

async function renderWriteAction(action: () => Promise<string>): Promise<void> {
  const output = outputEl();
  output.textContent = "Running write action…";
  try {
    output.textContent = await action();
  } catch (error) {
    output.textContent = `Error: ${formatError(error)}`;
  }
}

function formVaultRequest(): VaultRequest {
  return {
    path: vaultPath(),
    password: inputValue("#master-password"),
    keyFile: inputValue("#key-file"),
  };
}

function clearPasswordInput(): void {
  setInputValue("#master-password", "");
}

function authOutputEl(): HTMLDivElement {
  const output = document.querySelector<HTMLDivElement>("#auth-output");
  if (!output) {
    throw new Error("auth output element missing");
  }
  return output;
}

function focusMasterPassword(): void {
  document.querySelector<HTMLInputElement>("#master-password")?.focus();
}

function vaultPath(): string {
  return inputValue("#vault-path");
}

window.addEventListener("unhandledrejection", (event: PromiseRejectionEvent) => {
  setClipboardStatus(errorMessage(event.reason), "locked");
});
