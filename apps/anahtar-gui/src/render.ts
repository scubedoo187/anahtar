import type { AuditReport, EntryDetail, EntrySummary, GroupSummary, WriteReport } from "./api";
import { versionLabel } from "./api";
import {
  auditFindingsEl,
  detailOutputEl,
  entryListEl,
  groupListEl,
  setDisabled,
} from "./dom";
import type { AppState } from "./state";

export type EntrySelectHandler = (entryId: string) => void;

export function renderSessionState(state: AppState): void {
  const unlocked = state.activeSession !== null;
  setDisabled("#lock-vault", !unlocked);
  setDisabled("#vault-path", unlocked);
  setDisabled("#key-file", unlocked);
  setDisabled("#search-query", !unlocked);
  setDisabled("#search-entries", !unlocked);
  setDisabled("#reset-list", !unlocked);
  setDisabled("#reload-detail", !unlocked || !state.selectedEntryId);
  setDisabled("#reveal-detail", !unlocked || !state.selectedEntryId || state.detailRevealed);
  setDisabled("#copy-username", !unlocked || !state.selectedDetail?.username);
  setDisabled("#copy-password", !unlocked || !state.selectedEntryId);
  setDisabled("#copy-url", !unlocked || !state.selectedDetail?.url);
  setDisabled("#copy-totp", !unlocked || !state.selectedEntryId);
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
  setDisabled("#edit-title", !unlocked || !state.selectedEntryId);
  setDisabled("#edit-username", !unlocked || !state.selectedEntryId);
  setDisabled("#edit-password", !unlocked || !state.selectedEntryId);
  setDisabled("#edit-url", !unlocked || !state.selectedEntryId);
  setDisabled("#edit-notes", !unlocked || !state.selectedEntryId);
  setDisabled("#edit-entry", !unlocked || !state.selectedEntryId);
  setDisabled("#delete-entry", !unlocked || !state.selectedEntryId);

  const status = document.querySelector<HTMLDivElement>("#session-status");
  if (!status) return;
  if (unlocked) {
    status.className = "status unlocked";
    status.textContent = `Unlocked: ${state.activeEntries.length} entries loaded. Password is held in memory only.`;
  } else {
    status.className = "status locked";
    status.textContent = "Locked";
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

export function renderEntryDetail(detail: EntryDetail, revealPassword: boolean): void {
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

export function renderEmptyDetail(message: string): void {
  detailOutputEl().textContent = message;
}

export function renderGroups(state: AppState, groups: GroupSummary[]): void {
  const list = groupListEl();
  list.textContent = "";

  if (!state.activeSession) {
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

export function detailLine(label: string, value: string): HTMLDivElement {
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
