export const defaultVaultPath = "../../../test-vaults/generated/phase3-base.kdbx";

export function renderShell(app: HTMLDivElement): void {
  // Static shell only: do not interpolate vault, entry, or secret data into this template.
  app.innerHTML = `
    <section class="shell desktop-shell">
      <section id="auth-screen" class="auth-screen" aria-label="Unlock vault">
        <section class="auth-card">
          <h1>Unlock vault</h1>
          <p class="hint">Choose a KDBX vault and enter its master password for this in-memory session.</p>

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

          <div class="session-actions">
            <button id="unlock-vault" type="button">Unlock</button>
          </div>
        </section>
      </section>

      <section id="app-frame" class="app-frame" aria-label="Anahtar workspace" hidden>
        <header class="workspace-topbar">
          <div>
            <div id="session-status" class="status locked">Locked</div>
          </div>
          <div class="topbar-actions">
            <button id="nav-browse" type="button" data-view="browse" aria-selected="true">Browse</button>
            <button id="nav-audit" type="button" data-view="audit" aria-selected="false">Audit</button>
            <button id="nav-status" type="button" data-view="status" aria-selected="false">Status</button>
            <button id="lock-vault" type="button" disabled>Lock</button>
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
                  <button id="add-group" type="button" disabled>+</button>
                  <button id="rename-group" type="button" disabled>Rename</button>
                  <button id="delete-group" type="button" disabled>Delete</button>
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
                <button id="new-entry" type="button" disabled>+ New</button>
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
                  <button id="reload-detail" type="button" disabled>Reload</button>
                  <button id="edit-selected" type="button" disabled>Edit</button>
                  <button id="delete-entry" type="button" disabled>Delete</button>
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
                <button id="run-audit" type="button" disabled>Run audit</button>
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
                <button id="refresh-status" type="button">Refresh backend status</button>
              </div>
              <p id="backend-status">Checking Rust backend…</p>
              <button id="inspect-vault" type="button">Inspect vault</button>
              <div id="command-output" class="output compact" aria-live="polite">No command run yet.</div>
            </section>
          </section>
        </main>
      </section>
    </section>
  `;
}
