import { invoke } from "@tauri-apps/api/core";
import "./styles.css";

type BackendStatus = {
  app: string;
  version: string;
  service: string;
};

const app = document.querySelector<HTMLDivElement>("#app");

if (!app) {
  throw new Error("#app root element is missing");
}

app.innerHTML = `
  <section class="shell">
    <header>
      <p class="eyebrow">Anahtar GUI Alpha</p>
      <h1>Anahtar</h1>
      <p class="subtitle">macOS-first KeePass/KDBX password manager GUI.</p>
    </header>

    <section class="card" aria-live="polite">
      <h2>Backend status</h2>
      <p id="backend-status">Checking Rust backend…</p>
      <button id="refresh-status" type="button">Refresh backend status</button>
    </section>
  </section>
`;

async function refreshBackendStatus(): Promise<void> {
  const statusEl = document.querySelector<HTMLParagraphElement>("#backend-status");
  if (!statusEl) return;

  try {
    const status = await invoke<BackendStatus>("backend_status");
    statusEl.textContent = `${status.app} ${status.version} — ${status.service}`;
  } catch (error) {
    statusEl.textContent = `Backend unavailable: ${String(error)}`;
  }
}

document
  .querySelector<HTMLButtonElement>("#refresh-status")
  ?.addEventListener("click", () => {
    void refreshBackendStatus();
  });

void refreshBackendStatus();
