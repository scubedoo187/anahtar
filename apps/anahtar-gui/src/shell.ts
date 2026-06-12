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
          <button id="lock-vault" type="button" disabled>Lock</button>
        </header>

        <aside class="sidebar" aria-label="Navigation">
          <div class="sidebar-group">
            <p class="sidebar-heading">Vault</p>
            <button id="nav-browse" class="nav-button" type="button" data-view="browse" aria-selected="true">Browse</button>
            <button id="nav-groups" class="nav-button" type="button" data-view="groups" aria-selected="false">Groups</button>
          </div>

          <div class="sidebar-group">
            <p class="sidebar-heading">Tools</p>
            <button id="nav-audit" class="nav-button" type="button" data-view="audit" aria-selected="false">Audit</button>
            <button id="nav-write" class="nav-button" type="button" data-view="write" aria-selected="false">Write</button>
            <button id="nav-status" class="nav-button" type="button" data-view="status" aria-selected="false">Status</button>
          </div>
        </aside>

        <main class="workspace" aria-live="polite">
          <section id="view-panel-browse" class="view-panel browse-layout" data-view-panel="browse">
            <section class="pane list-pane">
              <div class="pane-header">
                <div>
                  <h2>Entries</h2>
                  <p class="hint">Search and select an entry. Detail lookup uses the selected UUID.</p>
                </div>
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
              <div class="pane-header">
                <div>
                  <h2>Entry detail</h2>
                  <p class="hint">Passwords and protected fields are hidden by default.</p>
                </div>
              </div>
              <div class="button-row compact-actions">
                <button id="reload-detail" type="button" disabled>Reload safe detail</button>
                <button id="reveal-detail" type="button" disabled>Reveal sensitive fields</button>
              </div>
              <div class="button-row compact-actions">
                <button id="copy-username" type="button" disabled>Copy username</button>
                <button id="copy-password" type="button" disabled>Copy password</button>
                <button id="copy-url" type="button" disabled>Copy URL</button>
                <button id="copy-totp" type="button" disabled>Copy TOTP</button>
              </div>
              <div id="clipboard-status" class="status neutral">Clipboard idle.</div>
              <div id="entry-detail" class="detail-output" aria-live="polite">Select an entry to view details.</div>
            </section>
          </section>

          <section id="view-panel-groups" class="view-panel tool-layout" data-view-panel="groups" hidden>
            <section class="pane">
              <div class="pane-header split-header">
                <div>
                  <h2>Groups</h2>
                  <p class="hint">Group list comes from anahtar-app and contains no secret values.</p>
                </div>
                <button id="load-groups" type="button" disabled>Load groups</button>
              </div>
              <div id="group-list" class="entry-list compact-list" aria-live="polite">Unlock first to inspect groups.</div>
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

          <section id="view-panel-write" class="view-panel tool-layout" data-view-panel="write" hidden>
            <section class="pane command-surface">
              <div class="pane-header">
                <div>
                  <h2>Write actions</h2>
                  <p class="hint">Write actions update the unlocked vault in place and display the backup path. Test on generated-vault copies first.</p>
                </div>
              </div>

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
                  <label>Notes <input id="add-notes" type="text" value="Added from Anahtar GUI" disabled /></label>
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
