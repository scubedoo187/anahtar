export const defaultVaultPath = "";

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
              <input id="key-file" type="text" spellcheck="false" />
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

          <section class="recent-vaults-section" aria-label="Recent vaults">
            <div class="pane-header split-header">
              <h2>Recent vaults</h2>
              <button id="clear-recent-vaults" class="icon-button" type="button" title="Clear recent vaults" aria-label="Clear recent vaults">⌫</button>
            </div>
            <div id="recent-vaults" class="recent-vaults" aria-live="polite">No recent vaults.</div>
          </section>

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
                <input id="search-query" type="search" disabled />
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

        <div id="confirm-dialog" class="modal-backdrop" hidden>
          <section class="modal-card confirm-card" role="dialog" aria-modal="true" aria-labelledby="confirm-dialog-title">
            <div>
              <h2 id="confirm-dialog-title">Confirm action</h2>
              <p id="confirm-dialog-message" class="hint">Are you sure?</p>
            </div>
            <div class="session-actions">
              <button id="confirm-dialog-confirm" class="danger-icon" type="button">Confirm</button>
              <button id="confirm-dialog-cancel" type="button">Cancel</button>
            </div>
          </section>
        </div>

        <div id="group-dialog" class="modal-backdrop" hidden>
          <form id="group-dialog-form" class="modal-card">
            <div class="pane-header split-header">
              <div>
                <h2 id="group-dialog-title">Group</h2>
                <p id="group-dialog-hint" class="hint">Enter a group path.</p>
              </div>
              <button id="group-dialog-close" class="icon-button" type="button" title="Close" aria-label="Close">×</button>
            </div>
            <label><span id="group-dialog-label">Group path</span><input id="group-dialog-value" type="text" required /></label>
            <div id="group-dialog-output" class="output compact" aria-live="polite">Ready.</div>
            <div class="session-actions">
              <button id="group-dialog-submit" type="submit">Save</button>
              <button id="group-dialog-cancel" type="button">Cancel</button>
            </div>
          </form>
        </div>

        <div id="entry-dialog" class="modal-backdrop" hidden>
          <form id="entry-dialog-form" class="modal-card">
            <div class="pane-header split-header">
              <div>
                <h2 id="entry-dialog-title">Entry</h2>
                <p class="hint">Password is only changed when the password field is non-empty.</p>
              </div>
              <button id="entry-dialog-close" class="icon-button" type="button" title="Close" aria-label="Close">×</button>
            </div>
            <label>Group path <input id="dialog-group" type="text" required /></label>
            <label>Title <input id="dialog-title" type="text" required /></label>
            <label>Username <input id="dialog-username" type="text" /></label>
            <label>Password <input id="dialog-password" type="password" autocomplete="new-password" /></label>
            <label>URL <input id="dialog-url" type="url" /></label>
            <label>Notes <input id="dialog-notes" type="text" /></label>
            <div id="entry-dialog-output" class="output compact" aria-live="polite">Ready.</div>
            <div class="session-actions">
              <button id="entry-dialog-submit" type="submit">Save</button>
              <button id="entry-dialog-cancel" type="button">Cancel</button>
            </div>
          </form>
        </div>
      </section>
    </section>
  `;
}
