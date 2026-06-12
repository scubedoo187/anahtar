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
  loadGuiConfig,
  moveEntry,
  rememberVault,
  clearRecentVaults,
  renameGroup,
  searchEntries,
  showEntry,
  totpCode,
  unlockVault,
  versionLabel,
  type AddEntryRequest,
  type EditEntryRequest,
  type GuiConfig,
  type RecentVault,
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
let entryDialogMode: "add" | "edit" | null = null;
let groupDialogMode: "add" | "rename" | null = null;
let pendingConfirm: ((confirmed: boolean) => void) | null = null;

renderShell(app);
setInputValue("#vault-path", defaultVaultPath);
void initializeGuiConfig();
void refreshBackendStatus();
renderAppChrome();
focusMasterPassword();

bindNavigation();
bindButton("#refresh-status", refreshBackendStatus);
bindButton("#inspect-vault", runInspect);
bindButton("#browse-vault", chooseVaultFile);
bindButton("#browse-key-file", chooseKeyFile);
bindButton("#clear-recent-vaults", clearRecentVaultList);
bindForm("#unlock-form", runUnlock);
bindButton("#lock-vault", lockVault);
bindButton("#search-entries", runSearch);
bindButton("#reset-list", resetList);
bindButton("#reload-detail", reloadSafeDetail);
bindButton("#new-entry", openAddEntryDialog);
bindButton("#edit-selected", openEditEntryDialog);
bindButton("#entry-dialog-close", closeEntryDialog);
bindButton("#entry-dialog-cancel", closeEntryDialog);
bindForm("#entry-dialog-form", submitEntryDialog);
bindButton("#confirm-dialog-confirm", async () => resolveConfirm(true));
bindButton("#confirm-dialog-cancel", async () => resolveConfirm(false));
bindButton("#add-group", openAddGroupDialog);
bindButton("#rename-group", openRenameGroupDialog);
bindButton("#group-dialog-close", closeGroupDialog);
bindButton("#group-dialog-cancel", closeGroupDialog);
bindForm("#group-dialog-form", submitGroupDialog);
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

async function initializeGuiConfig(): Promise<void> {
  try {
    const config = await loadGuiConfig();
    applyGuiConfig(config);
    authOutputEl().textContent = config.last_vault_path
      ? "Recent vault loaded. Enter the master password to unlock."
      : "Ready to unlock.";
  } catch (error) {
    authOutputEl().textContent = formatError(error);
  } finally {
    focusMasterPassword();
  }
}

function applyGuiConfig(config: GuiConfig): void {
  if (config.last_vault_path) {
    setInputValue("#vault-path", config.last_vault_path);
    const recent = config.recent_vaults.find((vault) => vault.path === config.last_vault_path);
    setInputValue("#key-file", recent?.key_file ?? "");
  }
  renderRecentVaults(config.recent_vaults);
}

function renderRecentVaults(recentVaults: RecentVault[]): void {
  const list = document.querySelector<HTMLDivElement>("#recent-vaults");
  if (!list) return;
  list.textContent = "";
  if (recentVaults.length === 0) {
    list.textContent = "No recent vaults.";
    return;
  }

  for (const vault of recentVaults) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "recent-vault-row";
    button.title = vault.path;
    button.addEventListener("click", () => {
      setInputValue("#vault-path", vault.path);
      setInputValue("#key-file", vault.key_file ?? "");
      authOutputEl().textContent = "Recent vault selected. Enter the master password to unlock.";
      focusMasterPassword();
    });

    const name = document.createElement("strong");
    name.textContent = basename(vault.path);
    const path = document.createElement("span");
    path.textContent = vault.path;
    button.append(name, path);
    list.append(button);
  }
}

async function clearRecentVaultList(): Promise<void> {
  try {
    const config = await clearRecentVaults();
    applyGuiConfig(config);
    setInputValue("#vault-path", defaultVaultPath);
    setInputValue("#key-file", "");
    authOutputEl().textContent = "Recent vaults cleared.";
    focusMasterPassword();
  } catch (error) {
    authOutputEl().textContent = formatError(error);
  }
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
    authOutputEl().textContent = formatError(error);
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
    authOutputEl().textContent = formatError(error);
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
    statusEl.textContent = `Backend unavailable. ${formatError(error)}`;
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
    const config = await rememberVault(request.path, request.keyFile);
    renderRecentVaults(config.recent_vaults);
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

function confirmAction(title: string, message: string): Promise<boolean> {
  const dialog = document.querySelector<HTMLElement>("#confirm-dialog");
  const titleEl = document.querySelector<HTMLHeadingElement>("#confirm-dialog-title");
  const messageEl = document.querySelector<HTMLParagraphElement>("#confirm-dialog-message");
  if (!dialog || !titleEl || !messageEl) {
    return Promise.resolve(false);
  }
  titleEl.textContent = title;
  messageEl.textContent = message;
  dialog.hidden = false;
  document.querySelector<HTMLButtonElement>("#confirm-dialog-cancel")?.focus();
  return new Promise((resolve) => {
    pendingConfirm = resolve;
  });
}

function resolveConfirm(confirmed: boolean): void {
  const dialog = document.querySelector<HTMLElement>("#confirm-dialog");
  if (dialog) dialog.hidden = true;
  pendingConfirm?.(confirmed);
  pendingConfirm = null;
}

function showGroupDialog(): void {
  const dialog = document.querySelector<HTMLElement>("#group-dialog");
  if (dialog) dialog.hidden = false;
  document.querySelector<HTMLInputElement>("#group-dialog-value")?.focus();
}

function setGroupDialogText(title: string, hint: string, label: string): void {
  const titleEl = document.querySelector<HTMLHeadingElement>("#group-dialog-title");
  const hintEl = document.querySelector<HTMLParagraphElement>("#group-dialog-hint");
  const labelEl = document.querySelector<HTMLSpanElement>("#group-dialog-label");
  if (titleEl) titleEl.textContent = title;
  if (hintEl) hintEl.textContent = hint;
  if (labelEl) labelEl.textContent = label;
}

function groupDialogOutputEl(): HTMLDivElement {
  const output = document.querySelector<HTMLDivElement>("#group-dialog-output");
  if (!output) throw new Error("group dialog output element missing");
  return output;
}

function showEntryDialog(): void {
  const dialog = document.querySelector<HTMLElement>("#entry-dialog");
  if (dialog) dialog.hidden = false;
  document.querySelector<HTMLInputElement>("#dialog-title")?.focus();
}

function setDialogTitle(title: string): void {
  const titleEl = document.querySelector<HTMLHeadingElement>("#entry-dialog-title");
  if (titleEl) titleEl.textContent = title;
}

function entryDialogOutputEl(): HTMLDivElement {
  const output = document.querySelector<HTMLDivElement>("#entry-dialog-output");
  if (!output) throw new Error("entry dialog output element missing");
  return output;
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
    detailEl.textContent = formatError(error);
  }
}

async function openAddGroupDialog(): Promise<void> {
  requireSession(state);
  groupDialogMode = "add";
  setGroupDialogText("New group", "Enter the full path for the new group.", "Group path");
  setInputValue("#group-dialog-value", state.selectedGroupPath ? `${state.selectedGroupPath}/` : "");
  groupDialogOutputEl().textContent = "Ready.";
  showGroupDialog();
}

async function openRenameGroupDialog(): Promise<void> {
  requireSession(state);
  if (!state.selectedGroupPath) throw new Error("select a group first");
  groupDialogMode = "rename";
  setGroupDialogText("Rename group", `Renaming ${state.selectedGroupPath}.`, "New name");
  setInputValue("#group-dialog-value", state.selectedGroupPath.split("/").pop() ?? state.selectedGroupPath);
  groupDialogOutputEl().textContent = "Ready.";
  showGroupDialog();
}

async function closeGroupDialog(): Promise<void> {
  groupDialogMode = null;
  const dialog = document.querySelector<HTMLElement>("#group-dialog");
  if (dialog) dialog.hidden = true;
}

async function submitGroupDialog(): Promise<void> {
  const output = groupDialogOutputEl();
  output.textContent = "Saving…";
  try {
    output.textContent = await saveGroupDialog();
  } catch (error) {
    output.textContent = formatError(error);
  }
}

async function saveGroupDialog(): Promise<string> {
  const session = requireSession(state);
  const value = inputValue("#group-dialog-value").trim();

  if (groupDialogMode === "add") {
    if (!value) throw new Error("group path is required");
    const report = await addGroup(session, value);
    await closeGroupDialog();
    await refreshEntriesAfterWrite(null);
    state.selectedGroupPath = value;
    renderGroupTree(state, state.groups, selectGroup);
    renderEntryList(state, filteredEntries(), selectEntry);
    return renderWriteReport(report);
  }

  if (groupDialogMode === "rename") {
    if (!state.selectedGroupPath) throw new Error("select a group first");
    if (!value) throw new Error("new group name is required");
    const oldPath = state.selectedGroupPath;
    const report = await renameGroup(session, oldPath, value);
    await closeGroupDialog();
    await refreshEntriesAfterWrite(null);
    const parent = oldPath.split("/").slice(0, -1).join("/");
    state.selectedGroupPath = parent ? `${parent}/${value}` : value;
    renderGroupTree(state, state.groups, selectGroup);
    renderEntryList(state, filteredEntries(), selectEntry);
    return renderWriteReport(report);
  }

  throw new Error("group dialog is not open");
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
    if (!(await confirmAction("Delete group", `Delete empty group "${groupPath}"? This cannot be undone.`))) {
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

async function openAddEntryDialog(): Promise<void> {
  requireSession(state);
  entryDialogMode = "add";
  setDialogTitle("New entry");
  setInputValue("#dialog-group", state.selectedGroupPath ?? "General/Web");
  setInputValue("#dialog-title", "");
  setInputValue("#dialog-username", "");
  setInputValue("#dialog-password", "");
  setInputValue("#dialog-url", "");
  setInputValue("#dialog-notes", "");
  entryDialogOutputEl().textContent = "Ready.";
  showEntryDialog();
}

async function openEditEntryDialog(): Promise<void> {
  requireSession(state);
  const detail = requireSelectedDetail(state);
  entryDialogMode = "edit";
  setDialogTitle("Edit entry");
  setInputValue("#dialog-group", detail.group_path);
  setInputValue("#dialog-title", detail.title ?? "");
  setInputValue("#dialog-username", detail.username ?? "");
  setInputValue("#dialog-password", "");
  setInputValue("#dialog-url", detail.url ?? "");
  setInputValue("#dialog-notes", detail.notes ?? "");
  entryDialogOutputEl().textContent = "Leave password blank to keep the current password.";
  showEntryDialog();
}

async function closeEntryDialog(): Promise<void> {
  entryDialogMode = null;
  const dialog = document.querySelector<HTMLElement>("#entry-dialog");
  if (dialog) dialog.hidden = true;
}

async function submitEntryDialog(): Promise<void> {
  const output = entryDialogOutputEl();
  output.textContent = "Saving…";
  try {
    output.textContent = await saveEntryDialog();
  } catch (error) {
    output.textContent = formatError(error);
  }
}

async function saveEntryDialog(): Promise<string> {
  const session = requireSession(state);
  if (entryDialogMode === "add") {
    const request: AddEntryRequest = {
      group_path: inputValue("#dialog-group"),
      title: inputValue("#dialog-title"),
      username: inputValue("#dialog-username"),
      password: inputValue("#dialog-password"),
      url: inputValue("#dialog-url"),
      notes: inputValue("#dialog-notes"),
      backup_dir: null,
    };
    if (!request.group_path.trim() || !request.title.trim()) {
      throw new Error("group path and title are required");
    }
    const report = await addEntry(session, request);
    await closeEntryDialog();
    await refreshEntriesAfterWrite(report.changed_entry_id);
    return renderWriteReport(report);
  }

  if (entryDialogMode === "edit") {
    const detail = requireSelectedDetail(state);
    if (!state.selectedEntryId) {
      throw new Error("select an entry first");
    }
    const groupPath = inputValue("#dialog-group").trim();
    const request: EditEntryRequest = {
      title: inputValue("#dialog-title"),
      username: inputValue("#dialog-username"),
      password: inputValue("#dialog-password"),
      url: inputValue("#dialog-url"),
      notes: inputValue("#dialog-notes"),
      backup_dir: null,
    };
    let report = await editEntry(session, state.selectedEntryId, request);
    if (groupPath && groupPath !== detail.group_path) {
      report = await moveEntry(session, state.selectedEntryId, groupPath);
    }
    await closeEntryDialog();
    await refreshEntriesAfterWrite(report.changed_entry_id ?? state.selectedEntryId);
    return renderWriteReport(report);
  }

  throw new Error("entry dialog is not open");
}

async function runDeleteEntry(): Promise<void> {
  await renderWriteAction(async () => {
    const session = requireSession(state);
    if (!state.selectedEntryId) {
      throw new Error("select an entry first");
    }
    const detailTitle = state.selectedDetail?.title ?? state.selectedEntryId;
    if (!(await confirmAction("Delete entry", `Delete "${detailTitle}"? A backup will be created before the vault is replaced.`))) {
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
    output.textContent = formatError(error);
  }
}

async function renderCommand(action: () => Promise<string>): Promise<void> {
  const output = outputEl();
  output.textContent = "Running…";
  try {
    output.textContent = await action();
  } catch (error) {
    output.textContent = formatError(error);
  }
}

async function renderWriteAction(action: () => Promise<string>): Promise<void> {
  const output = outputEl();
  output.textContent = "Running write action…";
  try {
    output.textContent = await action();
  } catch (error) {
    output.textContent = formatError(error);
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

function basename(path: string): string {
  return path.split(/[\\/]/).filter(Boolean).pop() ?? path;
}

window.addEventListener("unhandledrejection", (event: PromiseRejectionEvent) => {
  setClipboardStatus(errorMessage(event.reason), "locked");
});
