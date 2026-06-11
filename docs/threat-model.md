# Anahtar threat model

Anahtar is a local KeePass/KDBX password-manager CLI. This document describes what it is intended to protect and the risks users still need to manage.

## In scope

Anahtar aims to protect secrets stored in KDBX vaults by:

- using the existing `keepass` crate instead of custom cryptography,
- prompting for the master password through the terminal instead of CLI args,
- supporting optional key-file material,
- avoiding password output by default,
- using timed clipboard clearing for copy workflows,
- writing KDBX output through backup/temp/reopen verification flows.

## Out of scope

Anahtar does not protect against:

- malware or keyloggers on the local machine,
- a compromised terminal emulator or shell,
- a compromised OS clipboard service,
- malicious shell history/scrollback capture,
- an attacker who can read the unlocked vault contents from process memory,
- cloud sync conflicts created outside Anahtar,
- loss of both vault and backups.

## Master password handling

The master password is prompted through TTY using hidden input. It should not be passed as a command-line argument. Anahtar may keep the password in process memory while the command is running.

## Key-file handling

Key-file paths may be stored in Anahtar config as canonical absolute paths. The path itself is treated as local configuration, not as secret material. The key-file contents are sensitive and must be protected and backed up separately.

## Clipboard risks

Copy commands place secrets in the OS clipboard. Anahtar waits for the configured timeout and clears the clipboard only if it still contains the exact value Anahtar copied. Other applications may still read clipboard contents before the clear happens.

Headless or SSH-only environments may not have a usable clipboard.

## Terminal output and scrollback risks

`show --reveal-password` and JSON detail output that includes revealed password data can persist in terminal scrollback, shell logs, screen recordings, or command capture tools. Prefer copy commands for secrets.

## Safe write model

In-place writes create a timestamped backup before replacing the target vault. The write path uses a temporary output, reopens it for verification, replaces the target, then reopens the final target. Backups are retained after success.

If a preflight failure occurs before replacement, the original vault should remain unchanged. If a failure happens during OS-level file replacement, recovery may require using the backup.

## Backups

By default, backups are stored under `anahtar-backups/` next to the vault. A config override may point backups elsewhere. Users should ensure backups are included in their normal backup strategy and not accidentally exposed.

## Public repo hygiene

Do not commit real `.kdbx`, `.kdb`, `.key`, or `.keyx` files. Do not commit private vault paths, screenshots, copied secrets, or generated output containing real credentials.
