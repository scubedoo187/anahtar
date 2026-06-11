import {
  backendStatus,
  inspectVault,
  searchEntries,
  showEntry,
  unlockVault,
  versionLabel,
  type EntryDetail,
  type EntrySummary,
  type SelectorKind,
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
    </section>

    <section class="card command-surface">
      <h2>Read commands</h2>
      <p class="hint">Unlock first. Search and show reuse the in-memory alpha session.</p>

      <label>
        Search query
        <input id="search-query" type="text" value="Github" disabled />
      </label>
      <button id="search-entries" type="button" disabled>Search</button>

      <div class="selector-row">
        <label>
          Selector kind
          <select id="selector-kind" disabled>
            <option value="title">title</option>
            <option value="id">id</option>
            <option value="url">url</option>
            <option value="username">username</option>
            <option value="auto">auto</option>
          </select>
        </label>
        <label>
          Selector value
          <input id="selector-value" type="text" value="Github Test" disabled />
        </label>
      </div>
      <label class="checkbox-label">
        <input id="reveal-password" type="checkbox" disabled />
        Reveal password for detail command
      </label>
      <button id="show-entry" type="button" disabled>Show detail</button>

      <div id="command-output" class="output" aria-live="polite">No command run yet.</div>
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
bindButton("#show-entry", runShow);

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
    clearPasswordInput();
    renderSessionState();
    return renderEntries("Unlocked entries", entries);
  });
}

function lockVault(): Promise<void> {
  activeSession = null;
  activeEntries = [];
  clearPasswordInput();
  renderSessionState();
  outputEl().textContent = "Locked. In-memory session cleared.";
  return Promise.resolve();
}

async function runSearch(): Promise<void> {
  await renderCommand(async () => {
    const session = requireSession();
    const entries = await searchEntries(session, inputValue("#search-query"));
    activeEntries = entries;
    return renderEntries("Search results", entries);
  });
}

async function runShow(): Promise<void> {
  await renderCommand(async () => {
    const session = requireSession();
    const detail = await showEntry(
      session,
      selectorKind(),
      inputValue("#selector-value"),
      checkboxValue("#reveal-password"),
    );
    return renderDetail(detail);
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

function renderEntries(title: string, entries: EntrySummary[]): string {
  const lines = entries.slice(0, 20).map((entry) => {
    const title = entry.title ?? "<untitled>";
    const username = entry.username ?? "";
    const url = entry.url ?? "";
    return `- ${title} | ${username} | ${url} | ${entry.id}`;
  });
  return `${title}: ${entries.length}\n${lines.join("\n")}`;
}

function renderDetail(detail: EntryDetail): string {
  const password = detail.password ? "<revealed>" : "<hidden>";
  const customFields = detail.custom_fields
    .map((field) => `  - ${field.key}: ${field.protected ? "<protected>" : field.value}`)
    .join("\n");
  return [
    `ID: ${detail.id}`,
    `Group: ${detail.group_path}`,
    `Title: ${detail.title ?? ""}`,
    `Username: ${detail.username ?? ""}`,
    `URL: ${detail.url ?? ""}`,
    `Notes: ${detail.notes ?? ""}`,
    `Password: ${password}`,
    `Custom fields:`,
    customFields || "  <none>",
  ].join("\n");
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
  setDisabled("#selector-kind", !unlocked);
  setDisabled("#selector-value", !unlocked);
  setDisabled("#reveal-password", !unlocked);
  setDisabled("#show-entry", !unlocked);

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

function selectorKind(): SelectorKind {
  return inputValue("#selector-kind") as SelectorKind;
}

function inputValue(selector: string): string {
  const input = document.querySelector<HTMLInputElement | HTMLSelectElement>(selector);
  return input?.value ?? "";
}

function checkboxValue(selector: string): boolean {
  return document.querySelector<HTMLInputElement>(selector)?.checked ?? false;
}

function outputEl(): HTMLDivElement {
  const output = document.querySelector<HTMLDivElement>("#command-output");
  if (!output) {
    throw new Error("command output element missing");
  }
  return output;
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
