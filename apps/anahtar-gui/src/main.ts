import {
  addEntry,
  auditVault,
  backendStatus,
  deleteEntry,
  editEntry,
  inspectVault,
  listGroups,
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
  writeReportEl,
} from "./dom";
import { errorMessage, formatError } from "./errors";
import {
  renderAudit,
  renderEmptyDetail,
  filterEntriesForSelectedGroup,
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

bindNavigation();
bindButton("#refresh-status", refreshBackendStatus);
bindButton("#inspect-vault", runInspect);
bindButton("#unlock-vault", runUnlock);
bindButton("#lock-vault", lockVault);
bindButton("#search-entries", runSearch);
bindButton("#reset-list", resetList);
bindButton("#reload-detail", reloadSafeDetail);
bindButton("#run-audit", runAudit);
bindForm("#add-entry-form", runAddEntry);
bindForm("#edit-entry-form", runEditEntry);
bindButton("#delete-entry", runDeleteEntry);

function bindNavigation(): void {
  for (const view of ["browse", "audit", "write", "status"] as ActiveView[]) {
    bindButton(`#nav-${view}`, async () => {
      setActiveView(view);
    });
  }
}

function setActiveView(view: ActiveView): void {
  state.activeView = view;
  renderAppChrome();
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
  await renderCommand(async () => {
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
  writeReportEl().textContent = "No write action run yet.";
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
    const request: AddEntryRequest = {
      group_path: inputValue("#add-group"),
      title: inputValue("#add-title"),
      username: inputValue("#add-username"),
      password: inputValue("#add-password"),
      url: inputValue("#add-url"),
      notes: inputValue("#add-notes"),
      backup_dir: inputValue("#backup-dir"),
    };
    if (!request.group_path.trim() || !request.title.trim()) {
      throw new Error("group path and title are required");
    }
    const report = await addEntry(session, request);
    clearWritePasswordInputs();
    await refreshEntriesAfterWrite(report.changed_entry_id);
    return renderWriteReport(report);
  });
}

async function runEditEntry(): Promise<void> {
  await renderWriteAction(async () => {
    const session = requireSession(state);
    if (!state.selectedEntryId) {
      throw new Error("select an entry first");
    }
    const request: EditEntryRequest = {
      title: inputValue("#edit-title"),
      username: inputValue("#edit-username"),
      password: inputValue("#edit-password"),
      url: inputValue("#edit-url"),
      notes: inputValue("#edit-notes"),
      backup_dir: inputValue("#backup-dir"),
    };
    if (!hasEditChange(request)) {
      throw new Error("provide at least one edit field");
    }
    const report = await editEntry(session, state.selectedEntryId, request);
    clearEditInputs();
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
    const report = await deleteEntry(session, state.selectedEntryId, inputValue("#backup-dir"));
    await refreshEntriesAfterWrite(null);
    return renderWriteReport(report);
  });
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
  const output = writeReportEl();
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

function hasEditChange(request: EditEntryRequest): boolean {
  return [request.title, request.username, request.password, request.url, request.notes].some(
    (value) => (value ?? "").trim().length > 0,
  );
}

function clearEditInputs(): void {
  for (const selector of ["#edit-title", "#edit-username", "#edit-password", "#edit-url", "#edit-notes"]) {
    setInputValue(selector, "");
  }
}

function clearWritePasswordInputs(): void {
  for (const selector of ["#add-password", "#edit-password"]) {
    setInputValue(selector, "");
  }
}

function clearPasswordInput(): void {
  setInputValue("#master-password", "");
}

function vaultPath(): string {
  return inputValue("#vault-path");
}

window.addEventListener("unhandledrejection", (event: PromiseRejectionEvent) => {
  setClipboardStatus(errorMessage(event.reason), "locked");
});
