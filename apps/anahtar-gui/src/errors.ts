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
  return normalizeError(error).message;
}

export function formatError(error: unknown): string {
  const normalized = normalizeError(error);
  return `${normalized.kind}: ${normalized.message}`;
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
