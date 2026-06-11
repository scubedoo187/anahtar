# Phase 6 — GUI Alpha

Status: proposed, deferred until Phase 5 completion

## Goal

Build the first Anahtar desktop GUI on top of the Phase 5 CLI/productized core capabilities.

The GUI should not invent core password-manager behavior. It should reuse `anahtar-core` and the policies validated by the CLI.

## Preconditions

Phase 6 should start only after Phase 5 has settled:

- Credential material model.
- Stable selectors.
- Safe write model.
- Basic group/move support.
- Audit result types.
- Product documentation and threat model.

## Candidate stack

Tauri remains the likely first GUI stack because it can reuse Rust directly and package for desktop platforms.

Mac is the first packaging target. Windows/Linux are follow-up targets.

## GUI alpha scope

Initial GUI features:

- Open/configure vault.
- Unlock vault.
- Search entries.
- List entries.
- View safe details.
- Copy username/password/TOTP using the same clipboard policy.
- Add/edit/delete using the same safe write model as CLI.
- Group list and move if Phase 5 exposes stable APIs.
- Show audit findings if Phase 5 audit result types are available.

## GUI alpha non-goals

- Browser extension.
- Mobile app.
- Cloud sync engine.
- Shared vaults.
- Background unlock daemon.
- Biometric unlock.
- Passkeys.

## Exit criteria

- GUI can perform daily read/copy workflows using `anahtar-core`.
- GUI write actions use the same safety model as CLI.
- GUI can be packaged locally for macOS.
- GUI does not require duplicating KDBX logic outside `anahtar-core`.
