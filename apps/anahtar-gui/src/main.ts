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
  type AuditReport,
  type EditEntryRequest,
  type EntryDetail,
  type EntrySummary,
  type GroupSummary,
  type VaultRequest,
  type WriteReport,
} from "./api";
import "./styles.css";

const app = document.querySelector<HTMLDivElement>("#app");

if (!app) {
  throw new Error("#app root element is missing");
}

const defaultVaultPath = "../../test-vaults/generated/phase3-base.kdbx";
let activeSession: VaultRequest | null = null;
let activeEntries: EntrySummary[] = [];
let selectedEntryId: string | null = null;
let selectedDetail: EntryDetail | null = null;
let detailRevealed = false;
let clipboardClearTimer: number | null = null;

app.innerHTML = `
  <section class="shell">
    <header>
      <p class="eyebrow">Anahtar GUI Alpha</p>
      <h1>Anahtar</h1>
      <p class="subtitle">macOS-first KeePass/KDBX password manager GUI.</p>
    </header>

    <section class="card" aria-live="polite">
      <h2>Backend status</h2>
      <p id="backend-status">Checking Rust backend…</p>
      <button id="refresh-status" type="button">Refresh backend status</button>
    </section>

    <section class="card command-surface">
      <h2>Unlock vault</h2>
      <p class="hint">Password is kept in memory only for this alpha session. It is cleared when you lock or close the app.</p>

      <label>
        Vault path
        <input id="vault-path" type="text" spellcheck="false" value="${defaultVaultPath}" />
      </label>

      <label>
        Key-file path <span class="muted">optional</span>
        <input id="key-file" type="text" spellcheck="false" placeholder="/path/to/key-file.keyx" />
      </label>

      <label>
        Master password
        <input id="master-password" type="password" autocomplete="off" />
      </label>

      <div class="button-row">
        <button id="inspect-vault" type="button">Inspect</button>
        <button id="unlock-vault" type="button">Unlock/List</button>
        <button id="lock-vault" type="button" disabled>Lock</button>
      </div>

      <div id="session-status" class="status locked">Locked</div>
      <div id="command-output" class="output compact" aria-live="polite">No command run yet.</div>
    </section>

    <section class="workspace-grid">
      <section class="card entry-browser">
        <h2>Entries</h2>
        <p class="hint">Unlock first. Selecting an entry uses its UUID for detail lookup.</p>
        <label>
          Search query
          <input id="search-query" type="search" value="Github" disabled />
        </label>
        <div class="button-row">
          <button id="search-entries" type="button" disabled>Search</button>
          <button id="reset-list" type="button" disabled>Show unlocked list</button>
        </div>
        <div id="entry-list" class="entry-list" aria-live="polite">No vault unlocked.</div>
      </section>

      <section class="card detail-panel">
        <h2>Entry detail</h2>
        <p class="hint">Passwords and protected fields are hidden by default.</p>
        <div class="button-row">
          <button id="reload-detail" type="button" disabled>Reload safe detail</button>
          <button id="reveal-detail" type="button" disabled>Reveal sensitive fields</button>
        </div>
        <div class="button-row">
          <button id="copy-username" type="button" disabled>Copy username</button>
          <button id="copy-password" type="button" disabled>Copy password</button>
          <button id="copy-url" type="button" disabled>Copy URL</button>
          <button id="copy-totp" type="button" disabled>Copy TOTP</button>
        </div>
        <div id="clipboard-status" class="status neutral">Clipboard idle.</div>
        <div id="entry-detail" class="detail-output" aria-live="polite">Select an entry to view details.</div>
      </section>
    </section>

    <section class="workspace-grid">
      <section class="card">
        <h2>Groups</h2>
        <p class="hint">Group list comes from anahtar-app and contains no secret values.</p>
        <button id="load-groups" type="button" disabled>Load groups</button>
        <div id="group-list" class="entry-list compact-list" aria-live="polite">Unlock first to inspect groups.</div>
      </section>

      <section class="card">
        <h2>Audit</h2>
        <p class="hint">Audit findings are designed to be actionable without printing secret values.</p>
        <button id="run-audit" type="button" disabled>Run audit</button>
        <div id="audit-findings" class="detail-output" aria-live="polite">Unlock first to run audit.</div>
      </section>
    </section>

    <section class="card command-surface">
      <h2>Write actions</h2>
      <p class="hint">Alpha write actions update the unlocked vault in place through anahtar-app and display the backup path. Test on generated-vault copies only.</p>

      <label>
        Backup directory <span class="muted">optional</span>
        <input id="backup-dir" type="text" spellcheck="false" placeholder="defaults to sibling anahtar-backups/" disabled />
      </label>

      <div class="write-grid">
        <form id="add-entry-form" class="write-form">
          <h3>Add entry</h3>
          <label>Group path <input id="add-group" type="text" value="General/Web" disabled /></label>
          <label>Title <input id="add-title" type="text" value="GUI Added Entry" disabled /></label>
          <label>Username <input id="add-username" type="text" value="gui-user" disabled /></label>
          <label>Password <input id="add-password" type="password" autocomplete="new-password" disabled /></label>
          <label>URL <input id="add-url" type="url" value="https://gui.example.com" disabled /></label>
          <label>Notes <input id="add-notes" type="text" value="Added from Anahtar GUI alpha" disabled /></label>
          <button id="add-entry" type="submit" disabled>Add entry</button>
        </form>

        <form id="edit-entry-form" class="write-form">
          <h3>Edit selected entry</h3>
          <label>Title <input id="edit-title" type="text" placeholder="leave blank to keep" disabled /></label>
          <label>Username <input id="edit-username" type="text" placeholder="leave blank to keep" disabled /></label>
          <label>Password <input id="edit-password" type="password" autocomplete="new-password" placeholder="leave blank to keep" disabled /></label>
          <label>URL <input id="edit-url" type="url" placeholder="leave blank to keep" disabled /></label>
          <label>Notes <input id="edit-notes" type="text" placeholder="leave blank to keep" disabled /></label>
          <button id="edit-entry" type="submit" disabled>Edit selected</button>
        </form>
      </div>

      <div class="danger-zone">
        <h3>Delete selected entry</h3>
        <p class="hint">Requires confirmation. A backup is created before the vault is replaced.</p>
        <button id="delete-entry" type="button" disabled>Delete selected entry</button>
      </div>

      <div id="write-report" class="output compact" aria-live="polite">No write action run yet.</div>
    </section>
  </section>
`;

void refreshBackendStatus();
renderSessionState();

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
    statusEl.textContent = `Backend unavailable: ${errorMessage(error)}`;
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
    activeSession = request;
    activeEntries = entries;
    selectedEntryId = null;
    selectedDetail = null;
    detailRevealed = false;
    clearPasswordInput();
    renderSessionState();
    renderEntryList(entries);
    renderEmptyDetail("Select an entry to view details.");
    return `Unlocked ${entries.length} entries. Select an entry from the list to view safe details.`;
  });
}

async function refreshEntriesAfterWrite(changedEntryId?: string | null): Promise<void> {
  const session = requireSession();
  activeEntries = await unlockVault(session);
  selectedEntryId = changedEntryId ?? null;
  selectedDetail = null;
  detailRevealed = false;
  renderEntryList(activeEntries);
  if (selectedEntryId) {
    await loadSelectedDetail(false);
  } else {
    renderEmptyDetail("Select an entry to view details.");
  }
  renderSessionState();
}

function lockVault(): Promise<void> {
  activeSession = null;
  activeEntries = [];
  selectedEntryId = null;
  selectedDetail = null;
  detailRevealed = false;
  clearClipboardTimer();
  clearPasswordInput();
  renderSessionState();
  renderEntryList([]);
  renderEmptyDetail("Select an entry to view details.");
  renderGroups([]);
  renderAudit(null);
  clipboardStatus("Clipboard idle.", "neutral");
  writeReportEl().textContent = "No write action run yet.";
  outputEl().textContent = "Locked. In-memory session cleared.";
  return Promise.resolve();
}

async function runSearch(): Promise<void> {
  await renderCommand(async () => {
    const session = requireSession();
    const entries = await searchEntries(session, inputValue("#search-query"));
    activeEntries = entries;
    selectedEntryId = null;
    selectedDetail = null;
    detailRevealed = false;
    renderEntryList(entries);
    renderEmptyDetail("Select a search result to view details.");
    renderSessionState();
    return `Search returned ${entries.length} entries.`;
  });
}

function resetList(): Promise<void> {
  selectedEntryId = null;
  selectedDetail = null;
  detailRevealed = false;
  renderEntryList(activeEntries);
  renderEmptyDetail("Select an entry to view details.");
  outputEl().textContent = `Showing ${activeEntries.length} entries from current in-memory list.`;
  renderSessionState();
  return Promise.resolve();
}

async function selectEntry(entryId: string): Promise<void> {
  selectedEntryId = entryId;
  detailRevealed = false;
  renderEntryList(activeEntries);
  await loadSelectedDetail(false);
}

async function reloadSafeDetail(): Promise<void> {
  await loadSelectedDetail(false);
}

async function revealSelectedDetail(): Promise<void> {
  await loadSelectedDetail(true);
}

async function copySelectedUsername(): Promise<void> {
  const detail = requireSelectedDetail();
  await copyWithOwnedClear(detail.username ?? "", "username");
}

async function copySelectedUrl(): Promise<void> {
  const detail = requireSelectedDetail();
  await copyWithOwnedClear(detail.url ?? "", "URL");
}

async function copySelectedPassword(): Promise<void> {
  const session = requireSession();
  if (!selectedEntryId) {
    throw new Error("select an entry first");
  }
  const detail = await showEntry(session, "id", selectedEntryId, true);
  await copyWithOwnedClear(detail.password ?? "", "password");
}

async function copySelectedTotp(): Promise<void> {
  const session = requireSession();
  if (!selectedEntryId) {
    throw new Error("select an entry first");
  }
  const code = await totpCode(session, "id", selectedEntryId);
  await copyWithOwnedClear(code.code, `TOTP code valid for ${code.valid_for_seconds}s`);
}

async function loadSelectedDetail(revealPassword: boolean): Promise<void> {
  const session = requireSession();
  if (!selectedEntryId) {
    throw new Error("select an entry first");
  }

  const detailEl = detailOutputEl();
  detailEl.textContent = "Loading detail…";
  try {
    const detail = await showEntry(session, "id", selectedEntryId, revealPassword);
    selectedDetail = detail;
    detailRevealed = revealPassword;
    renderEntryDetail(detail, revealPassword);
    renderSessionState();
  } catch (error) {
    detailEl.textContent = `Error: ${errorMessage(error)}`;
  }
}

async function runLoadGroups(): Promise<void> {
  await renderCommand(async () => {
    const groups = await listGroups(requireSession());
    renderGroups(groups);
    return `Loaded ${groups.length} groups.`;
  });
}

async function runAudit(): Promise<void> {
  await renderCommand(async () => {
    const report = await auditVault(requireSession());
    renderAudit(report);
    return `Audit found ${report.findings.length} findings.`;
  });
}

async function runAddEntry(): Promise<void> {
  await renderWriteAction(async () => {
    const session = requireSession();
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
    const session = requireSession();
    if (!selectedEntryId) {
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
    const report = await editEntry(session, selectedEntryId, request);
    clearEditInputs();
    await refreshEntriesAfterWrite(report.changed_entry_id ?? selectedEntryId);
    return renderWriteReport(report);
  });
}

async function runDeleteEntry(): Promise<void> {
  await renderWriteAction(async () => {
    const session = requireSession();
    if (!selectedEntryId) {
      throw new Error("select an entry first");
    }
    const detailTitle = selectedDetail?.title ?? selectedEntryId;
    const confirmed = window.confirm(
      `Delete "${detailTitle}"? A backup will be created before the vault is replaced.`,
    );
    if (!confirmed) {
      return "Delete cancelled.";
    }
    const report = await deleteEntry(session, selectedEntryId, inputValue("#backup-dir"));
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
    output.textContent = `Error: ${errorMessage(error)}`;
  }
}

async function renderWriteAction(action: () => Promise<string>): Promise<void> {
  const output = writeReportEl();
  output.textContent = "Running write action…";
  try {
    output.textContent = await action();
  } catch (error) {
    output.textContent = `Error: ${errorMessage(error)}`;
  }
}

function renderWriteReport(report: WriteReport): string {
  return [
    `Operation: ${report.operation}`,
    `Input: ${report.input_path}`,
    `Output: ${report.output_path}`,
    `Entries: ${report.input_entry_count} -> ${report.output_entry_count}`,
    `Groups: ${report.input_group_count} -> ${report.output_group_count}`,
    `Changed ID: ${report.changed_entry_id ?? ""}`,
    `Backup: ${report.backup_path ?? ""}`,
    `Final target: ${report.final_target_path ?? ""}`,
  ].join("\n");
}

function renderGroups(groups: GroupSummary[]): void {
  const list = groupListEl();
  list.textContent = "";

  if (!activeSession) {
    list.textContent = "Unlock first to inspect groups.";
    return;
  }

  if (groups.length === 0) {
    list.textContent = "No groups loaded.";
    return;
  }

  for (const group of groups) {
    const item = document.createElement("div");
    item.className = "group-row";
    const title = document.createElement("strong");
    title.textContent = group.path;
    const meta = document.createElement("span");
    meta.textContent = `${group.entry_count} entries · ${group.child_group_count} child groups`;
    item.append(title, meta);
    list.append(item);
  }
}

function renderAudit(report: AuditReport | null): void {
  const list = auditFindingsEl();
  list.textContent = "";

  if (!activeSession) {
    list.textContent = "Unlock first to run audit.";
    return;
  }

  if (!report) {
    list.textContent = "No audit run yet.";
    return;
  }

  if (report.findings.length === 0) {
    list.textContent = "No audit findings.";
    return;
  }

  for (const finding of report.findings) {
    const item = document.createElement("div");
    item.className = "audit-row";
    item.append(
      detailLine("Kind", finding.kind),
      detailLine("Entry", finding.title ?? finding.entry_id),
      detailLine("Group", finding.group_path),
      detailLine("Message", finding.message),
    );
    list.append(item);
  }
}

function renderEntryList(entries: EntrySummary[]): void {
  const list = entryListEl();
  list.textContent = "";

  if (!activeSession) {
    list.textContent = "No vault unlocked.";
    return;
  }

  if (entries.length === 0) {
    list.textContent = "No entries to show.";
    return;
  }

  for (const entry of entries) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = entry.id === selectedEntryId ? "entry-row selected" : "entry-row";
    button.addEventListener("click", () => {
      void selectEntry(entry.id);
    });

    const title = document.createElement("strong");
    title.textContent = entry.title ?? "<untitled>";

    const meta = document.createElement("span");
    meta.textContent = `${entry.group_path} · ${entry.username ?? ""} · ${entry.url ?? ""}`;

    const id = document.createElement("code");
    id.textContent = entry.id;

    button.append(title, meta, id);
    list.append(button);
  }
}

async function copyWithOwnedClear(value: string, label: string): Promise<void> {
  if (!value) {
    clipboardStatus(`${label} is empty or unavailable.`, "locked");
    return;
  }
  if (!navigator.clipboard) {
    clipboardStatus("Clipboard API is unavailable in this environment.", "locked");
    return;
  }

  await navigator.clipboard.writeText(value);
  clipboardStatus(`Copied ${label}. Clipboard will clear in 30 seconds if unchanged.`, "unlocked");
  clearClipboardTimer();
  clipboardClearTimer = window.setTimeout(() => {
    void clearClipboardIfOwned(value);
  }, 30_000);
}

async function clearClipboardIfOwned(value: string): Promise<void> {
  try {
    if ((await navigator.clipboard.readText()) === value) {
      await navigator.clipboard.writeText("");
      clipboardStatus("Clipboard cleared.", "neutral");
    } else {
      clipboardStatus("Clipboard changed externally; Anahtar left it untouched.", "neutral");
    }
  } catch (error) {
    clipboardStatus(`Clipboard clear skipped: ${errorMessage(error)}`, "locked");
  } finally {
    clipboardClearTimer = null;
  }
}

function clearClipboardTimer(): void {
  if (clipboardClearTimer !== null) {
    window.clearTimeout(clipboardClearTimer);
    clipboardClearTimer = null;
  }
}

function renderEntryDetail(detail: EntryDetail, revealPassword: boolean): void {
  const detailEl = detailOutputEl();
  detailEl.textContent = "";

  detailEl.append(
    detailLine("ID", detail.id),
    detailLine("Group", detail.group_path),
    detailLine("Title", detail.title ?? ""),
    detailLine("Username", detail.username ?? ""),
    detailLine("URL", detail.url ?? ""),
    detailLine("Notes", detail.notes ?? ""),
    detailLine("Password", revealPassword ? (detail.password ?? "") : "<hidden>"),
    detailLine("Sensitive fields", revealPassword ? "revealed by explicit action" : "hidden"),
  );

  const customTitle = document.createElement("h3");
  customTitle.textContent = "Custom fields";
  detailEl.append(customTitle);

  if (detail.custom_fields.length === 0) {
    const empty = document.createElement("p");
    empty.className = "muted";
    empty.textContent = "No custom fields.";
    detailEl.append(empty);
    return;
  }

  for (const field of detail.custom_fields) {
    detailEl.append(detailLine(field.key, field.value));
  }
}

function detailLine(label: string, value: string): HTMLDivElement {
  const row = document.createElement("div");
  row.className = "detail-line";

  const labelEl = document.createElement("span");
  labelEl.className = "detail-label";
  labelEl.textContent = label;

  const valueEl = document.createElement("span");
  valueEl.className = "detail-value";
  valueEl.textContent = value;

  row.append(labelEl, valueEl);
  return row;
}

function renderEmptyDetail(message: string): void {
  const detailEl = detailOutputEl();
  detailEl.textContent = message;
}

function formVaultRequest(): VaultRequest {
  return {
    path: vaultPath(),
    password: inputValue("#master-password"),
    keyFile: inputValue("#key-file"),
  };
}

function requireSession(): VaultRequest {
  if (!activeSession) {
    throw new Error("unlock the vault first");
  }
  return activeSession;
}

function renderSessionState(): void {
  const unlocked = activeSession !== null;
  setDisabled("#lock-vault", !unlocked);
  setDisabled("#vault-path", unlocked);
  setDisabled("#key-file", unlocked);
  setDisabled("#search-query", !unlocked);
  setDisabled("#search-entries", !unlocked);
  setDisabled("#reset-list", !unlocked);
  setDisabled("#reload-detail", !unlocked || !selectedEntryId);
  setDisabled("#reveal-detail", !unlocked || !selectedEntryId || detailRevealed);
  setDisabled("#copy-username", !unlocked || !selectedDetail?.username);
  setDisabled("#copy-password", !unlocked || !selectedEntryId);
  setDisabled("#copy-url", !unlocked || !selectedDetail?.url);
  setDisabled("#copy-totp", !unlocked || !selectedEntryId);
  setDisabled("#load-groups", !unlocked);
  setDisabled("#run-audit", !unlocked);
  setDisabled("#backup-dir", !unlocked);
  setDisabled("#add-group", !unlocked);
  setDisabled("#add-title", !unlocked);
  setDisabled("#add-username", !unlocked);
  setDisabled("#add-password", !unlocked);
  setDisabled("#add-url", !unlocked);
  setDisabled("#add-notes", !unlocked);
  setDisabled("#add-entry", !unlocked);
  setDisabled("#edit-title", !unlocked || !selectedEntryId);
  setDisabled("#edit-username", !unlocked || !selectedEntryId);
  setDisabled("#edit-password", !unlocked || !selectedEntryId);
  setDisabled("#edit-url", !unlocked || !selectedEntryId);
  setDisabled("#edit-notes", !unlocked || !selectedEntryId);
  setDisabled("#edit-entry", !unlocked || !selectedEntryId);
  setDisabled("#delete-entry", !unlocked || !selectedEntryId);

  const status = document.querySelector<HTMLDivElement>("#session-status");
  if (!status) return;
  if (unlocked) {
    status.className = "status unlocked";
    status.textContent = `Unlocked: ${activeEntries.length} entries loaded. Password is held in memory only.`;
  } else {
    status.className = "status locked";
    status.textContent = "Locked";
  }
}

function requireSelectedDetail(): EntryDetail {
  if (!selectedDetail) {
    throw new Error("select an entry first");
  }
  return selectedDetail;
}

function clipboardStatus(message: string, state: "locked" | "neutral" | "unlocked"): void {
  const status = document.querySelector<HTMLDivElement>("#clipboard-status");
  if (!status) return;
  status.className = `status ${state}`;
  status.textContent = message;
}

function hasEditChange(request: EditEntryRequest): boolean {
  return [request.title, request.username, request.password, request.url, request.notes].some(
    (value) => (value ?? "").trim().length > 0,
  );
}

function clearEditInputs(): void {
  for (const selector of ["#edit-title", "#edit-username", "#edit-password", "#edit-url", "#edit-notes"]) {
    const input = document.querySelector<HTMLInputElement>(selector);
    if (input) input.value = "";
  }
}

function clearWritePasswordInputs(): void {
  for (const selector of ["#add-password", "#edit-password"]) {
    const input = document.querySelector<HTMLInputElement>(selector);
    if (input) input.value = "";
  }
}

function clearPasswordInput(): void {
  const input = document.querySelector<HTMLInputElement>("#master-password");
  if (input) {
    input.value = "";
  }
}

function vaultPath(): string {
  return inputValue("#vault-path");
}

function inputValue(selector: string): string {
  const input = document.querySelector<HTMLInputElement | HTMLSelectElement>(selector);
  return input?.value ?? "";
}

function writeReportEl(): HTMLDivElement {
  const output = document.querySelector<HTMLDivElement>("#write-report");
  if (!output) {
    throw new Error("write report element missing");
  }
  return output;
}

function outputEl(): HTMLDivElement {
  const output = document.querySelector<HTMLDivElement>("#command-output");
  if (!output) {
    throw new Error("command output element missing");
  }
  return output;
}

function entryListEl(): HTMLDivElement {
  const list = document.querySelector<HTMLDivElement>("#entry-list");
  if (!list) {
    throw new Error("entry list element missing");
  }
  return list;
}

function groupListEl(): HTMLDivElement {
  const list = document.querySelector<HTMLDivElement>("#group-list");
  if (!list) {
    throw new Error("group list element missing");
  }
  return list;
}

function auditFindingsEl(): HTMLDivElement {
  const list = document.querySelector<HTMLDivElement>("#audit-findings");
  if (!list) {
    throw new Error("audit findings element missing");
  }
  return list;
}

function detailOutputEl(): HTMLDivElement {
  const detail = document.querySelector<HTMLDivElement>("#entry-detail");
  if (!detail) {
    throw new Error("entry detail element missing");
  }
  return detail;
}

function setDisabled(selector: string, disabled: boolean): void {
  const element = document.querySelector<HTMLInputElement | HTMLButtonElement | HTMLSelectElement>(
    selector,
  );
  if (element) {
    element.disabled = disabled;
  }
}

function bindButton(selector: string, handler: () => Promise<void>): void {
  document.querySelector<HTMLButtonElement>(selector)?.addEventListener("click", () => {
    void handler().catch((error: unknown) => {
      clipboardStatus(errorMessage(error), "locked");
    });
  });
}

function bindForm(selector: string, handler: () => Promise<void>): void {
  document.querySelector<HTMLFormElement>(selector)?.addEventListener("submit", (event) => {
    event.preventDefault();
    void handler().catch((error: unknown) => {
      writeReportEl().textContent = `Error: ${errorMessage(error)}`;
    });
  });
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
