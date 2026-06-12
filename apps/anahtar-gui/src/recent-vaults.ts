import type { RecentVault } from "./api";

export type RecentVaultSelectHandler = (vault: RecentVault) => void;

export function renderRecentVaults(
  recentVaults: RecentVault[],
  onSelect: RecentVaultSelectHandler,
): void {
  const list = document.querySelector<HTMLDivElement>("#recent-vaults");
  if (!list) return;
  list.textContent = "";
  if (recentVaults.length === 0) {
    list.textContent = "No recent vaults.";
    return;
  }

  for (const vault of recentVaults) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "recent-vault-row";
    button.title = vault.path;
    button.addEventListener("click", () => onSelect(vault));

    const name = document.createElement("strong");
    name.textContent = basename(vault.path);
    const path = document.createElement("span");
    path.textContent = vault.path;
    button.append(name, path);
    list.append(button);
  }
}

function basename(path: string): string {
  return path.split(/[\\/]/).filter(Boolean).pop() ?? path;
}
