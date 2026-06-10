# Keepass crate compatibility spike report

Date: 2026-06-10  
Spike path: `spikes/keepass-compat`

## Goal

Validate whether Rust `keepass = 0.13.8` can serve as Anahtar's first KDBX core for:

1. reading existing KDBX files,
2. safely round-tripping writable files,
3. mutating entries and saving,
4. informing the CLI/GUI architecture decision.

## Setup

Created a small Rust binary:

- `spikes/keepass-compat/Cargo.toml`
- `spikes/keepass-compat/src/main.rs`
- `spikes/keepass-compat/fetch-fixtures.sh`

Dependency:

```toml
keepass = { version = "0.13.8", features = ["save_kdbx4", "totp"] }
```

Fixtures were downloaded from upstream `sseemayer/keepass-rs` test resources. They contain synthetic data only.

## Tested fixtures

| Fixture | Password | Open | Native save | Upgrade-to-KDBX4.1 mutated save |
|---|---|---:|---:|---:|
| `test_db_with_password.kdbx` | `demopass` | OK, KDBX3.1 | Unsupported | OK |
| `test_db_kdbx4_with_password_aes.kdbx` | `demopass` | OK, KDBX4.1 | OK | n/a |
| `test_db_kdbx4_with_password_argon2.kdbx` | `demopass` | OK, KDBX4.0 | Unsupported | OK |
| `test_db_kdbx4_with_password_argon2_chacha20.kdbx` | `demopass` | OK, KDBX4.0 | Unsupported | OK |
| `test_db_kdbx4_with_password_argon2id.kdbx` | `demopass` | OK, KDBX4.0 | Unsupported | OK |
| `test_db_kdbx4_with_totp_entry.kdbx` | `test` | OK, KDBX4.0 | Unsupported | OK |
| `test_db_kdbx41_features.kdbx` | `demopass` | OK, KDBX4.1 | OK | n/a |

## Run result

Command:

```bash
cd spikes/keepass-compat
cargo run -- fixtures
```

Result:

```text
SUMMARY: 7 fixtures passed basic open/save/reopen checks
```

## Important findings

### 1. Read support looks good for first CLI MVP

The crate successfully opened tested KDBX3.1, KDBX4.0, and KDBX4.1 fixtures.

This is enough confidence to proceed with a read-only CLI MVP using this library.

### 2. Native save is KDBX4.1-only

Although `Database::save` is exposed for KDBX4, the writer currently rejects anything except:

```rust
DatabaseVersion::KDB4(1)
```

So direct save of KDBX3.1 and KDBX4.0 fixtures returns `UnsupportedVersion`.

Implication:

- Anahtar must not promise transparent in-place save for every opened database initially.
- The first write MVP should either:
  - require/produce KDBX4.1, or
  - perform an explicit conversion step with user confirmation.

### 3. Explicit KDBX4.1 upgrade worked in self-tests

For KDBX3.1 and KDBX4.0 fixtures, the spike set:

```rust
db.config.version = DatabaseVersion::KDB4(1);
```

Then added a synthetic marker entry, saved, and reopened with `keepass`. This worked for every tested non-KDBX4.1 fixture.

However, this is only a **library self-compatibility** result. It still needs manual validation with KeePassXC/Strongbox before trusting it for real vaults.

### 4. KDBX4.1 writer includes known KeePassXC-compatible XML markers

For `test_db_kdbx41_features.kdbx`, the spike decrypted the saved XML and checked upstream KeePassXC compatibility markers:

- `<EnableSearching>null</EnableSearching>`
- `<EnableAutoType>null</EnableAutoType>`
- `<DataTransferObfuscation>0</DataTransferObfuscation>`
- no bool-string `DataTransferObfuscation` values

This passed.

This is encouraging because these markers correspond to real KeePassXC rejection bugs documented in upstream tests.

## Limitations

This spike does **not** yet prove:

- Strongbox can open the generated files.
- KeePassXC installed locally can open the generated files.
- A real personal vault with attachments/custom icons/history/Strongbox-specific metadata roundtrips without loss.
- Keyfile-based vaults work in the Anahtar UX.
- In-place save is safe.

No local `keepassxc-cli` was found on this machine. Strongbox.app is installed, but CLI automation was not available in this spike.

Generated files that can be manually opened in Strongbox:

- `spikes/keepass-compat/out/test_db_kdbx4_with_password_aes.mutated.kdbx` / password `demopass`
- `spikes/keepass-compat/out/test_db_kdbx41_features.mutated.kdbx` / password `demopass`
- `spikes/keepass-compat/out/test_db_kdbx4_with_password_argon2.upgraded-mutated.kdbx` / password `demopass`
- `spikes/keepass-compat/out/test_db_kdbx4_with_totp_entry.upgraded-mutated.kdbx` / password `test`

## Recommendation

Proceed with `keepass` as the first Anahtar core candidate.

Recommended implementation posture:

1. Build read-only CLI first.
2. For write MVP, only support safe `save-as` initially.
3. If input DB is not KDBX4.1, require explicit `--upgrade-to-kdbx41` or similar.
4. Never perform implicit in-place format upgrade.
5. Add a manual compatibility checklist for Strongbox/KeePassXC before using on real vaults.

## Next validation step

Manual validation in Strongbox:

1. Open generated `.kdbx` files under `spikes/keepass-compat/out/`.
2. Verify password unlock works.
3. Verify original sample entries are visible.
4. Verify the `Anahtar Spike Marker ...` entry is visible.
5. Optionally edit/save in Strongbox, then reopen through the spike or later CLI.

If these pass, the next coding step should be the real workspace skeleton:

```text
crates/anahtar-core
crates/anahtar-cli
```

with read-only commands first.
