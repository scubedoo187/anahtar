import {
  backendStatus,
  inspectVault,
  searchEntries,
  showEntry,
  unlockVault,
  versionLabel,
  type EntryDetail,
  type EntrySummary,
  type VaultRequest,
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
        <div id="entry-detail" class="detail-output" aria-live="polite">Select an entry to view details.</div>
      </section>
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

function lockVault(): Promise<void> {
  activeSession = null;
  activeEntries = [];
  selectedEntryId = null;
  selectedDetail = null;
  detailRevealed = false;
  clearPasswordInput();
  renderSessionState();
  renderEntryList([]);
  renderEmptyDetail("Select an entry to view details.");
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

async function renderCommand(action: () => Promise<string>): Promise<void> {
  const output = outputEl();
  output.textContent = "Running…";
  try {
    output.textContent = await action();
  } catch (error) {
    output.textContent = `Error: ${errorMessage(error)}`;
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
    void handler();
  });
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
