import { clipboardStatusEl } from "./dom";
import { errorMessage } from "./errors";

type ClipboardState = "locked" | "neutral" | "unlocked";

let clipboardClearTimer: number | null = null;

export async function copyWithOwnedClear(value: string, label: string): Promise<void> {
  if (!value) {
    setClipboardStatus(`${label} is empty or unavailable.`, "locked");
    return;
  }
  if (!navigator.clipboard) {
    setClipboardStatus("Clipboard API is unavailable in this environment.", "locked");
    return;
  }

  await navigator.clipboard.writeText(value);
  setClipboardStatus(`Copied ${label}. Clipboard will clear in 30 seconds if unchanged.`, "unlocked");
  clearClipboardTimer();
  clipboardClearTimer = window.setTimeout(() => {
    void clearClipboardIfOwned(value);
  }, 30_000);
}

export async function clearClipboardIfOwned(value: string): Promise<void> {
  try {
    if ((await navigator.clipboard.readText()) === value) {
      await navigator.clipboard.writeText("");
      setClipboardStatus("Clipboard cleared.", "neutral");
    } else {
      setClipboardStatus("Clipboard changed externally; Anahtar left it untouched.", "neutral");
    }
  } catch (error) {
    setClipboardStatus(`Clipboard clear skipped: ${errorMessage(error)}`, "locked");
  } finally {
    clipboardClearTimer = null;
  }
}

export function clearClipboardTimer(): void {
  if (clipboardClearTimer !== null) {
    window.clearTimeout(clipboardClearTimer);
    clipboardClearTimer = null;
  }
}

export function setClipboardStatus(message: string, state: ClipboardState): void {
  const status = clipboardStatusEl();
  status.className = `status ${state}`;
  status.textContent = message;
}
