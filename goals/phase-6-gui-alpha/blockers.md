# Blockers: Phase 6 GUI Alpha

## Open Questions

No blocker is currently known for starting Slice 1.

Questions that may need user steering during execution:

- Whether the first GUI frontend should remain vanilla TypeScript or add a small framework if Tauri scaffold defaults make that easier.
- Whether save-as write UI belongs in alpha or should remain CLI-only until after in-place write UX is proven.
- Whether group move should be included in alpha if it complicates the first usable GUI.
- What visual style/icon should be used beyond a placeholder.

## Stop And Ask

Pause and ask the user before:

- Adding a large frontend framework, design system, or state-management dependency.
- Adding telemetry, analytics, crash reporting, update infrastructure, or networked services.
- Persisting any secret material or changing credential storage policy.
- Running write tests against a real personal vault instead of generated/test copies.
- Changing `anahtar-app` or `anahtar-core` APIs in a way that breaks CLI behavior.
- Expanding scope to Windows/Linux packaging, biometric unlock, browser extension, mobile, or sync.
- Performing history rewrite, force push, or destructive repository operations.

## Dangerous Or High-Risk Actions

Require explicit approval:

- Any operation touching real `.kdbx`, `.kdb`, `.key`, or `.keyx` files.
- Any command that deletes, overwrites, migrates, or mutates non-generated vault files.
- Any dependency/tooling addition that requires broad install steps or changes CI platform assumptions.
- Any security-sensitive change that logs, stores, serializes, or caches master passwords or protected fields.

## Known Blockers

None at setup time.

If Tauri tooling is unavailable locally or requires installation, pause and ask before installing dependencies. If GUI packaging requires platform-specific signing/notarization decisions, defer that to a later goal unless the user explicitly expands scope.
