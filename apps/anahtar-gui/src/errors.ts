import type { AnahtarGuiError } from "./api";

export function normalizeError(error: unknown): AnahtarGuiError {
  if (isGuiError(error)) {
    return error;
  }

  if (error instanceof Error) {
    return { kind: "frontend_error", message: error.message };
  }

  if (typeof error === "string") {
    return { kind: "unknown", message: error };
  }

  return { kind: "unknown", message: String(error) };
}

export function errorMessage(error: unknown): string {
  return friendlyErrorMessage(normalizeError(error));
}

export function formatError(error: unknown): string {
  return friendlyErrorMessage(normalizeError(error));
}

function friendlyErrorMessage(error: AnahtarGuiError): string {
  const message = error.message.trim();
  const lower = message.toLowerCase();

  if (lower.includes("vault path is required")) {
    return "Choose a vault file first.";
  }
  if (lower.includes("master password is required")) {
    return "Enter the master password to unlock this vault.";
  }
  if (error.kind === "unlock_failed") {
    if (lower.includes("no such file or directory") || lower.includes("os error 2")) {
      return "We couldn't find that vault file. Choose a different file and try again.";
    }
    if (lower.includes("failed to open database") || lower.includes("invalid credentials")) {
      return "We couldn't unlock this vault. Check the password, key file, and selected vault.";
    }
    return "We couldn't unlock this vault. Check the password, key file, and selected vault.";
  }
  if (error.kind === "config_failed") {
    return "We couldn't read or update the recent vault list.";
  }
  if (error.kind === "inspect_failed") {
    return "We couldn't inspect that vault file.";
  }
  if (error.kind === "read_failed") {
    return "We couldn't load that item. Try selecting it again.";
  }
  if (error.kind === "totp_failed") {
    return "No TOTP code is available for this item.";
  }
  if (error.kind === "audit_failed") {
    return "We couldn't run the audit for this vault.";
  }
  if (error.kind === "group_failed") {
    return "We couldn't load the groups for this vault.";
  }
  if (error.kind === "write_failed") {
    return writeFriendlyMessage(lower, message);
  }
  if (lower.includes("select an entry first")) {
    return "Select an entry first.";
  }
  if (lower.includes("select a group first")) {
    return "Select a group first.";
  }
  if (lower.includes("group path is required")) {
    return "Enter a group path.";
  }
  if (lower.includes("group path and title are required")) {
    return "Enter both a group path and a title.";
  }
  if (lower.includes("new group name is required")) {
    return "Enter a new group name.";
  }
  if (lower.includes("cannot delete group with")) {
    return message.replace("cannot", "Cannot");
  }
  if (lower.includes("dialog.open not allowed")) {
    return "The file picker is not available yet. Restart the app and try again.";
  }

  return sentenceCase(message || "Something went wrong. Please try again.");
}

function writeFriendlyMessage(lower: string, original: string): string {
  if (lower.includes("already exists") || lower.includes("duplicate")) {
    return "An item or group with that name already exists.";
  }
  if (lower.includes("not found") || lower.includes("missing")) {
    return "We couldn't find the selected item or group. Refresh and try again.";
  }
  if (lower.includes("no such file or directory") || lower.includes("os error 2")) {
    return "We couldn't find the vault file. Choose the vault again and retry.";
  }
  if (lower.includes("permission denied")) {
    return "Anahtar doesn't have permission to write to that location.";
  }
  return sentenceCase(original || "We couldn't save the change. Please try again.");
}

function sentenceCase(value: string): string {
  if (!value) {
    return "Something went wrong. Please try again.";
  }
  return value.charAt(0).toUpperCase() + value.slice(1);
}

function isGuiError(error: unknown): error is AnahtarGuiError {
  return (
    typeof error === "object" &&
    error !== null &&
    "kind" in error &&
    "message" in error &&
    typeof (error as AnahtarGuiError).kind === "string" &&
    typeof (error as AnahtarGuiError).message === "string"
  );
}
