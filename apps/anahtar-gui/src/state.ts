import type { EntryDetail, EntrySummary, GroupSummary, VaultRequest } from "./api";

export type ActiveView = "browse" | "audit" | "write" | "status";

export type AppState = {
  activeView: ActiveView;
  activeSession: VaultRequest | null;
  activeEntries: EntrySummary[];
  visibleEntries: EntrySummary[];
  groups: GroupSummary[];
  selectedGroupPath: string | null;
  selectedEntryId: string | null;
  selectedDetail: EntryDetail | null;
  detailRevealed: boolean;
};

export function createInitialState(): AppState {
  return {
    activeView: "browse",
    activeSession: null,
    activeEntries: [],
    visibleEntries: [],
    groups: [],
    selectedGroupPath: null,
    selectedEntryId: null,
    selectedDetail: null,
    detailRevealed: false,
  };
}

export function clearSelection(state: AppState): void {
  state.selectedEntryId = null;
  state.selectedDetail = null;
  state.detailRevealed = false;
}

export function requireSession(state: AppState): VaultRequest {
  if (!state.activeSession) {
    throw new Error("unlock the vault first");
  }
  return state.activeSession;
}

export function requireSelectedDetail(state: AppState): EntryDetail {
  if (!state.selectedDetail) {
    throw new Error("select an entry first");
  }
  return state.selectedDetail;
}
