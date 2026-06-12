export const defaultVaultPath = "../../../test-vaults/generated/phase3-base.kdbx";

export function renderShell(app: HTMLDivElement): void {
  // Static shell only: do not interpolate vault, entry, or secret data into this template.
  app.innerHTML = `
    <section class="shell desktop-shell">
      <section id="auth-screen" class="auth-screen" aria-label="Unlock vault">
        <form id="unlock-form" class="auth-card">
          <h1>Unlock vault</h1>
          <p class="hint">Choose a KDBX vault and enter its master password for this in-memory session.</p>

          <label>
            Vault path
            <span class="path-picker-row">
              <input id="vault-path" type="text" spellcheck="false" />
              <button id="browse-vault" class="icon-button" type="button" title="Choose vault file" aria-label="Choose vault file">…</button>
            </span>
          </label>

          <label>
            Key-file path <span class="muted">optional</span>
            <span class="path-picker-row">
              <input id="key-file" type="text" spellcheck="false" placeholder="/path/to/key-file.keyx" />
              <button id="browse-key-file" class="icon-button" type="button" title="Choose key file" aria-label="Choose key file">…</button>
            </span>
          </label>

          <label>
            Master password
            <input id="master-password" type="password" autocomplete="off" />
          </label>

          <div class="session-actions">
            <button id="unlock-vault" type="submit">Unlock</button>
          </div>

          <div id="auth-output" class="output compact" aria-live="polite">Ready to unlock.</div>
        </form>
      </section>

      <section id="app-frame" class="app-frame" aria-label="Anahtar workspace" hidden>
        <header class="workspace-topbar">
          <div>
            <div id="session-status" class="status locked">Locked</div>
          </div>
          <div class="topbar-actions">
            <button id="nav-browse" class="icon-button" type="button" data-view="browse" aria-selected="true" title="Browse" aria-label="Browse">⌂</button>
            <button id="nav-audit" class="icon-button" type="button" data-view="audit" aria-selected="false" title="Audit" aria-label="Audit">✓</button>
            <button id="nav-status" class="icon-button" type="button" data-view="status" aria-selected="false" title="Status" aria-label="Status">ⓘ</button>
            <button id="lock-vault" class="icon-button" type="button" disabled title="Lock" aria-label="Lock">⏻</button>
          </div>
        </header>

        <main class="workspace" aria-live="polite">
          <section id="view-panel-browse" class="view-panel browse-layout" data-view-panel="browse">
            <section class="pane groups-pane">
              <div class="pane-header split-header">
                <div>
                  <h2>Groups</h2>
                  <p class="hint">Select a group to filter entries.</p>
                </div>
                <div class="mini-actions">
                  <button id="add-group" class="icon-button" type="button" disabled title="Add group" aria-label="Add group">＋</button>
                  <button id="rename-group" class="icon-button" type="button" disabled title="Rename group" aria-label="Rename group">✎</button>
                  <button id="delete-group" class="icon-button danger-icon" type="button" disabled title="Delete group" aria-label="Delete group">⌫</button>
                </div>
              </div>
              <div id="group-list" class="group-tree" aria-live="polite">Unlock first to inspect groups.</div>
            </section>

            <section class="pane list-pane">
              <div class="pane-header split-header">
                <div>
                  <h2>Entries</h2>
                  <p class="hint">Search and select an entry. Detail lookup uses the selected UUID.</p>
                </div>
                <button id="new-entry" class="icon-button" type="button" disabled title="New entry" aria-label="New entry">＋</button>
              </div>
              <label>
                Search query
                <input id="search-query" type="search" value="Github" disabled />
              </label>
              <div class="button-row compact-actions">
                <button id="search-entries" type="button" disabled>Search</button>
                <button id="reset-list" type="button" disabled>Show unlocked list</button>
              </div>
              <div id="entry-list" class="entry-list" aria-live="polite">No vault unlocked.</div>
            </section>

            <section class="pane detail-pane">
              <div class="pane-header split-header">
                <div>
                  <h2>Entry detail</h2>
                  <p class="hint">Passwords and protected fields are hidden by default.</p>
                </div>
                <div class="mini-actions">
                  <button id="reload-detail" class="icon-button" type="button" disabled title="Reload detail" aria-label="Reload detail">↻</button>
                  <button id="edit-selected" class="icon-button" type="button" disabled title="Edit entry" aria-label="Edit entry">✎</button>
                  <button id="delete-entry" class="icon-button danger-icon" type="button" disabled title="Delete entry" aria-label="Delete entry">⌫</button>
                </div>
              </div>
              <div id="clipboard-status" class="status neutral">Clipboard idle.</div>
              <div id="entry-detail" class="detail-output" aria-live="polite">Select an entry to view details.</div>
            </section>
          </section>

          <section id="view-panel-audit" class="view-panel tool-layout" data-view-panel="audit" hidden>
            <section class="pane">
              <div class="pane-header split-header">
                <div>
                  <h2>Audit</h2>
                  <p class="hint">Audit findings are designed to be actionable without printing secret values.</p>
                </div>
                <button id="run-audit" class="icon-button" type="button" disabled title="Run audit" aria-label="Run audit">▶</button>
              </div>
              <div id="audit-findings" class="detail-output" aria-live="polite">Unlock first to run audit.</div>
            </section>
          </section>

          <section id="view-panel-status" class="view-panel status-layout" data-view-panel="status" hidden>
            <section class="pane command-surface">
              <div class="pane-header split-header">
                <div>
                  <h2>Backend status</h2>
                  <p class="hint">Inspect the Rust backend and current vault file.</p>
                </div>
                <button id="refresh-status" class="icon-button" type="button" title="Refresh backend status" aria-label="Refresh backend status">↻</button>
              </div>
              <p id="backend-status">Checking Rust backend…</p>
              <button id="inspect-vault" class="icon-button" type="button" title="Inspect vault" aria-label="Inspect vault">ⓘ</button>
              <div id="command-output" class="output compact" aria-live="polite">No command run yet.</div>
            </section>
          </section>
        </main>
      </section>
    </section>
  `;
}
