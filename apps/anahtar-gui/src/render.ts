import type { AuditReport, EntryDetail, EntrySummary, GroupSummary, WriteReport } from "./api";
import { versionLabel } from "./api";
import {
  auditFindingsEl,
  detailOutputEl,
  entryListEl,
  groupListEl,
  setDisabled,
} from "./dom";
import type { ActiveView, AppState } from "./state";

export type EntrySelectHandler = (entryId: string) => void;
export type GroupSelectHandler = (groupPath: string | null) => void;
export type DetailActionHandlers = {
  copyUsername: () => Promise<void>;
  copyPassword: () => Promise<void>;
  copyUrl: () => Promise<void>;
  copyTotp: () => Promise<void>;
  revealPassword: () => Promise<void>;
};

const activeViews: ActiveView[] = ["browse", "audit", "write", "status"];

export function renderNavigationState(state: AppState): void {
  for (const view of activeViews) {
    const active = state.activeView === view;
    const button = document.querySelector<HTMLButtonElement>(`[data-view="${view}"]`);
    if (button) {
      button.classList.toggle("active", active);
      button.setAttribute("aria-selected", String(active));
    }

    const panel = document.querySelector<HTMLElement>(`[data-view-panel="${view}"]`);
    if (panel) {
      panel.hidden = !active;
    }
  }
}

export function renderSessionState(state: AppState): void {
  const unlocked = state.activeSession !== null;
  const authScreen = document.querySelector<HTMLElement>("#auth-screen");
  const appFrame = document.querySelector<HTMLElement>("#app-frame");
  if (authScreen) {
    authScreen.hidden = unlocked;
  }
  if (appFrame) {
    appFrame.hidden = !unlocked;
  }

  setDisabled("#lock-vault", !unlocked);
  setDisabled("#vault-path", unlocked);
  setDisabled("#key-file", unlocked);
  setDisabled("#search-query", !unlocked);
  setDisabled("#search-entries", !unlocked);
  setDisabled("#reset-list", !unlocked);
  setDisabled("#reload-detail", !unlocked || !state.selectedEntryId);
  setDisabled("#backup-dir", !unlocked);
  setDisabled("#add-group", !unlocked);
  setDisabled("#add-title", !unlocked);
  setDisabled("#add-username", !unlocked);
  setDisabled("#add-password", !unlocked);
  setDisabled("#add-url", !unlocked);
  setDisabled("#add-notes", !unlocked);
  setDisabled("#add-entry", !unlocked);
  setDisabled("#edit-title", !unlocked || !state.selectedEntryId);
  setDisabled("#edit-username", !unlocked || !state.selectedEntryId);
  setDisabled("#edit-password", !unlocked || !state.selectedEntryId);
  setDisabled("#edit-url", !unlocked || !state.selectedEntryId);
  setDisabled("#edit-notes", !unlocked || !state.selectedEntryId);
  setDisabled("#edit-entry", !unlocked || !state.selectedEntryId);
  setDisabled("#delete-entry", !unlocked || !state.selectedEntryId);
  setDisabled("#run-audit", !unlocked);

  const status = document.querySelector<HTMLDivElement>("#session-status");
  if (!status) return;
  if (unlocked) {
    status.className = "status unlocked";
    status.textContent = `Unlocked · ${state.activeEntries.length} entries loaded · ${vaultName(state.activeSession?.path)}`;
  } else {
    status.className = "status locked";
    status.textContent = "Locked";
  }
}

export function renderGroupTree(
  state: AppState,
  groups: GroupSummary[],
  onSelect: GroupSelectHandler,
): void {
  const list = groupListEl();
  list.textContent = "";

  if (!state.activeSession) {
    list.textContent = "Unlock first to inspect groups.";
    return;
  }

  list.append(groupTreeButton("All Entries", null, state.selectedGroupPath === null, 0, onSelect));

  if (groups.length === 0) {
    const empty = document.createElement("p");
    empty.className = "muted";
    empty.textContent = "No groups loaded.";
    list.append(empty);
    return;
  }

  for (const group of groups) {
    const normalizedPath = normalizeGroupPath(group.path);
    if (!normalizedPath) continue;
    const depth = normalizedPath.split("/").length - 1;
    const label = group.name || normalizedPath.split("/").pop() || normalizedPath;
    const button = groupTreeButton(
      label,
      normalizedPath,
      state.selectedGroupPath === normalizedPath,
      depth,
      onSelect,
    );
    const meta = document.createElement("span");
    meta.textContent = `${group.entry_count} entries`;
    button.append(meta);
    list.append(button);
  }
}

export function renderEntryList(
  state: AppState,
  entries: EntrySummary[],
  onSelect: EntrySelectHandler,
): void {
  const list = entryListEl();
  list.textContent = "";

  if (!state.activeSession) {
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
    button.className = entry.id === state.selectedEntryId ? "entry-row selected" : "entry-row";
    button.addEventListener("click", () => onSelect(entry.id));

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

export function renderEntryDetail(
  detail: EntryDetail,
  revealPassword: boolean,
  actions: DetailActionHandlers,
): void {
  const detailEl = detailOutputEl();
  detailEl.textContent = "";

  detailEl.append(
    detailLine("ID", detail.id),
    detailLine("Group", detail.group_path),
    detailLine("Title", detail.title ?? ""),
    detailLine("Username", detail.username ?? "", [fieldButton("Copy", actions.copyUsername, !detail.username)]),
    detailLine("Password", revealPassword ? (detail.password ?? "") : "<hidden>", [
      fieldButton("Copy", actions.copyPassword, false),
      fieldButton("Reveal", actions.revealPassword, revealPassword),
    ]),
    detailLine("URL", detail.url ?? "", [fieldButton("Copy", actions.copyUrl, !detail.url)]),
    detailLine("TOTP", "one-time code", [fieldButton("Copy code", actions.copyTotp, false)]),
    detailLine("Notes", detail.notes ?? ""),
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

export function renderEmptyDetail(message: string): void {
  detailOutputEl().textContent = message;
}

export function renderAudit(state: AppState, report: AuditReport | null): void {
  const list = auditFindingsEl();
  list.textContent = "";

  if (!state.activeSession) {
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

export function renderWriteReport(report: WriteReport): string {
  return [
    `Operation: ${report.operation}`,
    `Input: ${report.input_path}`,
    `Output: ${report.output_path}`,
    `Input version: ${versionLabel(report.input_version)}`,
    `Output version: ${versionLabel(report.output_version)}`,
    `Entries: ${report.input_entry_count} -> ${report.output_entry_count}`,
    `Groups: ${report.input_group_count} -> ${report.output_group_count}`,
    `Changed ID: ${report.changed_entry_id ?? ""}`,
    `Backup: ${report.backup_path ?? ""}`,
    `Final target: ${report.final_target_path ?? ""}`,
  ].join("\n");
}

export function normalizeGroupPath(path: string): string {
  return path.replace(/^Root\/?/, "").replace(/^\/+|\/+$/g, "");
}

function entryMatchesGroup(entry: EntrySummary, selectedGroupPath: string | null): boolean {
  if (!selectedGroupPath) {
    return true;
  }
  const entryPath = normalizeGroupPath(entry.group_path);
  return entryPath === selectedGroupPath || entryPath.startsWith(`${selectedGroupPath}/`);
}

export function filterEntriesForSelectedGroup(state: AppState, entries: EntrySummary[]): EntrySummary[] {
  return entries.filter((entry) => entryMatchesGroup(entry, state.selectedGroupPath));
}

function groupTreeButton(
  label: string,
  groupPath: string | null,
  selected: boolean,
  depth: number,
  onSelect: GroupSelectHandler,
): HTMLButtonElement {
  const button = document.createElement("button");
  button.type = "button";
  button.className = selected ? "group-tree-row selected" : "group-tree-row";
  button.style.setProperty("--depth", String(depth));
  button.addEventListener("click", () => onSelect(groupPath));

  const title = document.createElement("strong");
  title.textContent = label;
  button.append(title);
  return button;
}

function vaultName(path?: string): string {
  if (!path) {
    return "vault";
  }
  return path.split(/[\\/]/).filter(Boolean).pop() ?? path;
}

function fieldButton(label: string, action: () => Promise<void>, disabled: boolean): HTMLButtonElement {
  const button = document.createElement("button");
  button.type = "button";
  button.className = "field-action";
  button.textContent = label;
  button.disabled = disabled;
  button.addEventListener("click", () => {
    void action();
  });
  return button;
}

export function detailLine(label: string, value: string, actions: HTMLElement[] = []): HTMLDivElement {
  const row = document.createElement("div");
  row.className = actions.length > 0 ? "detail-line detail-line-with-actions" : "detail-line";

  const labelEl = document.createElement("span");
  labelEl.className = "detail-label";
  labelEl.textContent = label;

  const valueEl = document.createElement("span");
  valueEl.className = "detail-value";
  valueEl.textContent = value;

  row.append(labelEl, valueEl);
  if (actions.length > 0) {
    const actionsEl = document.createElement("span");
    actionsEl.className = "detail-actions";
    actionsEl.append(...actions);
    row.append(actionsEl);
  }
  return row;
}
