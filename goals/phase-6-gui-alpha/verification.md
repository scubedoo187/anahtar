# Verification: Phase 6 GUI Alpha

## Automated Commands

Run these from the repository root unless noted otherwise.

| Command | Purpose | Expected pass condition | Evidence location |
| --- | --- | --- | --- |
| `cargo fmt --all -- --check` | Rust formatting for workspace crates | exits 0 | append command/result to `progress.jsonl` |
| `cargo test --workspace` | Core/app/CLI regression tests | all tests pass | append command/result to `progress.jsonl` |
| `cargo clippy --workspace --all-targets -- -D warnings` | Rust lint gate | exits 0 with no warnings | append command/result to `progress.jsonl` |
| `cargo check --workspace --examples` | Workspace examples compile | exits 0 | append command/result to `progress.jsonl` |
| `cargo run -q -p anahtar-core --example generate_phase3_vault` | Regenerate synthetic KDBX test vault | creates/refreshes `test-vaults/generated/phase3-base.kdbx` | append command/result to `progress.jsonl` |
| `tmp="$(mktemp -d)"; cargo install --path crates/anahtar-cli --root "$tmp" --quiet; "$tmp/bin/anahtar" --version; "$tmp/bin/anahtar" completions bash >/tmp/anahtar-completions.bash; rm -rf "$tmp"` | CLI install/version/completion smoke | exits 0 and prints `anahtar 0.1.0` or current version | append command/result to `progress.jsonl` |
| GUI dev command, documented after scaffold | Tauri local launch | GUI window opens on macOS | append command/result and screenshot path if possible |
| GUI package command, documented after scaffold | macOS local package | package artifact is produced | append command/result and artifact path |

## Manual Checks

Record each manual check in `progress.jsonl` with timestamp, slice, result, and artifact path when available.

- GUI launches locally and shows Anahtar alpha shell.
- Frontend can call a trivial Rust backend command and render the returned status.
- Generated test vault unlocks with password `testpass`.
- Wrong password fails with a safe user-readable error.
- Master password is not stored in config files or logs.
- Entry list/search/detail workflows work against the generated test vault.
- Password and protected fields are hidden by default.
- Explicit reveal shows the password only after user action.
- Copy username/password/URL/TOTP actions do not print secrets.
- Clipboard clear only clears if clipboard still contains Anahtar's copied value.
- Add/edit/delete are tested only on a generated-vault copy, not a real personal vault.
- In-place write creates a backup and displays backup path in the GUI.
- Delete requires explicit confirmation.
- Group list and audit findings display without secret leakage.
- macOS local package excludes local/private vaults and ignored assets.

## Evidence Rules

- Append proof to `goals/phase-6-gui-alpha/progress.jsonl`; do not rewrite it.
- Include command, status, timestamp, and artifact path when available.
- For screenshots, store them under an ignored local artifact path or another non-secret location and record the path.
- Do not claim a slice complete unless its acceptance criteria have observable evidence.
- If a check cannot be completed, append the reason and list the remaining blocker in `blockers.md` or final user report.
