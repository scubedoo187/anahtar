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
  renderEntryDetail,
  renderEntryList,
  renderGroups,
  renderSessionState,
  renderWriteReport,
} from "./render";
import { createInitialState, clearSelection, requireSelectedDetail, requireSession } from "./state";
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
renderSessionState(state);

bindButton("#refresh-status", refreshBackendStatus);
bindButton("#inspect-vault", runInspect);
bindButton("#unlock-vault", runUnlock);
bindButton("#lock-vault", lockVault);
bindButton("#search-entries", runSearch);
bindButton("#reset-list", resetList);
bindButton("#reload-detail", reloadSafeDetail);
bindButton("#reveal-detail", revealSelectedDetail);
bindButton("#copy-username", copySelectedUsername);
bindButton("#copy-password", copySelectedPassword);
bindButton("#copy-url", copySelectedUrl);
bindButton("#copy-totp", copySelectedTotp);
bindButton("#load-groups", runLoadGroups);
bindButton("#run-audit", runAudit);
bindForm("#add-entry-form", runAddEntry);
bindForm("#edit-entry-form", runEditEntry);
bindButton("#delete-entry", runDeleteEntry);

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

    const entries = await unlockVault(request);
    state.activeSession = request;
    state.activeEntries = entries;
    clearSelection(state);
    clearPasswordInput();
    renderSessionState(state);
    renderEntryList(state, entries, selectEntry);
    renderEmptyDetail("Select an entry to view details.");
    return `Unlocked ${entries.length} entries. Select an entry from the list to view safe details.`;
  });
}

async function refreshEntriesAfterWrite(changedEntryId?: string | null): Promise<void> {
  const session = requireSession(state);
  state.activeEntries = await unlockVault(session);
  state.selectedEntryId = changedEntryId ?? null;
  state.selectedDetail = null;
  state.detailRevealed = false;
  renderEntryList(state, state.activeEntries, selectEntry);
  if (state.selectedEntryId) {
    await loadSelectedDetail(false);
  } else {
    renderEmptyDetail("Select an entry to view details.");
  }
  renderSessionState(state);
}

async function lockVault(): Promise<void> {
  state.activeSession = null;
  state.activeEntries = [];
  clearSelection(state);
  clearClipboardTimer();
  clearPasswordInput();
  renderSessionState(state);
  renderEntryList(state, [], selectEntry);
  renderEmptyDetail("Select an entry to view details.");
  renderGroups(state, []);
  renderAudit(state, null);
  setClipboardStatus("Clipboard idle.", "neutral");
  writeReportEl().textContent = "No write action run yet.";
  outputEl().textContent = "Locked. In-memory session cleared.";
}

async function runSearch(): Promise<void> {
  await renderCommand(async () => {
    const session = requireSession(state);
    const entries = await searchEntries(session, inputValue("#search-query"));
    state.activeEntries = entries;
    clearSelection(state);
    renderEntryList(state, entries, selectEntry);
    renderEmptyDetail("Select a search result to view details.");
    renderSessionState(state);
    return `Search returned ${entries.length} entries.`;
  });
}

async function resetList(): Promise<void> {
  clearSelection(state);
  renderEntryList(state, state.activeEntries, selectEntry);
  renderEmptyDetail("Select an entry to view details.");
  outputEl().textContent = `Showing ${state.activeEntries.length} entries from current in-memory list.`;
  renderSessionState(state);
}

function selectEntry(entryId: string): void {
  state.selectedEntryId = entryId;
  state.detailRevealed = false;
  renderEntryList(state, state.activeEntries, selectEntry);
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
    renderEntryDetail(detail, revealPassword);
    renderSessionState(state);
  } catch (error) {
    detailEl.textContent = `Error: ${formatError(error)}`;
  }
}

async function runLoadGroups(): Promise<void> {
  await renderCommand(async () => {
    const groups = await listGroups(requireSession(state));
    renderGroups(state, groups);
    return `Loaded ${groups.length} groups.`;
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
