export const defaultVaultPath = "../../../test-vaults/generated/phase3-base.kdbx";

export function renderShell(app: HTMLDivElement): void {
  // Static shell only: do not interpolate vault, entry, or secret data into this template.
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
          <input id="vault-path" type="text" spellcheck="false" />
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
}
