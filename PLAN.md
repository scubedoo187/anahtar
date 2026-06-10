# Anahtar MVP development plan

## Context

Anahtar의 1차 목표는 개인적으로 사용하는 Strongbox/1Password류 유료 도구 의존도를 줄이기 위해, KeePass `.kdbx` 파일을 읽고 쓸 수 있는 개인용 password manager를 직접 개발하는 것이다.

현재 검증된 사실:

- 개인 vault 원본은 `KDBX 4.0`이다.
- `assets/private-vault.backup.kdbx`를 백업 복사본으로 보존했다.
- `keepass = 0.13.8` 기반 spike에서 `KDBX 4.1` 변환 파일을 생성했다.
- `assets/private-vault.kdbx41.test.kdbx`는 `KDBX 4.1`이며 Strongbox에서 정상적으로 열린다.
- `keepass` crate는 read path가 충분히 유망하고, write path는 `KDBX 4.1` 중심으로 설계해야 한다.

## Approach

Rust workspace로 시작한다.

```text
crates/
  anahtar-core/   # KDBX open/read/search/inspect/write-safe abstraction
  anahtar-cli/    # CLI command UX
```

첫 개발 단계는 **read-only CLI MVP**로 제한한다. 이후 검증된 core 위에 write-safe CLI와 GUI를 순차적으로 붙인다.

핵심 정책:

- 원본 vault in-place 수정 금지.
- 읽기는 KDBX 3.x/4.0/4.1을 지원한다.
- 쓰기는 일단 KDBX 4.1 `save-as`만 지원한다.
- KDBX 4.0 → 4.1 변환은 명시적 `upgrade` 명령에서만 수행한다.
- 마스터 패스워드는 CLI argument로 받지 않고 TTY prompt로만 입력한다.
- 비밀번호는 기본 출력하지 않고, 명시적 reveal/copy 옵션에서만 다룬다.

## Files to modify

초기 구현에서 생성/수정할 주요 파일:

- `Cargo.toml` — Rust workspace 정의
- `crates/anahtar-core/Cargo.toml`
- `crates/anahtar-core/src/lib.rs`
- `crates/anahtar-cli/Cargo.toml`
- `crates/anahtar-cli/src/main.rs`
- `crates/anahtar-cli/src/commands/*.rs` — inspect/list/search/show/upgrade 등
- `docs/` — 사용법, 보안 정책, 검증 절차 문서

기존 spike는 보존한다.

- `spikes/keepass-compat/` — 라이브러리 검증/회귀 참고용

## Reuse

이미 확인한 재사용 자산:

- `spikes/keepass-compat/src/main.rs`
  - fixture open/save/reopen 검증 로직
  - group/entry count traversal
  - KDBX4.1 save/reopen verification pattern
- `spikes/keepass-compat/src/bin/upgrade_asset.rs`
  - KDBX4.0 → KDBX4.1 save-as upgrade 흐름
  - temp file 저장 후 reopen 검증
  - 원본 미수정 정책
- `docs/research-kdbx.md`
  - 라이브러리 비교 및 rationale
- `docs/spike-keepass-compat-report.md`
  - `keepass` crate 제약/검증 결과
- Rust crate `keepass = 0.13.8`
  - `Database::open`
  - `Database::save` with `save_kdbx4`
  - `Database::get_xml` for diagnostics if needed
  - `GroupRef`, `EntryRef`, `iter_all_entries`
- Rust crate candidates for CLI:
  - `clap` for command parsing
  - `rpassword` for hidden master password prompt
  - `anyhow`/`thiserror` for error handling
  - later: clipboard crate such as `arboard` for copy support

## Planning records

The user's requested planning records are now split into separate documents:

1. **Entire roadmap record**: `docs/anahtar-roadmap.md`
   - Fixed high-level Phase 1 → Phase 5 roadmap.
   - Should remain stable unless a major discovery changes the strategy.
2. **Per-phase implementation plans**: `docs/phase-plans/phase-1-readonly-cli.md`
   - Active phase plan updated as the phase progresses.
   - Future phase plans should be created sequentially when that phase starts.
   - Each phase must be closed by validating its exit criteria before moving on.
3. **Final goal record**: `docs/anahtar-final-goals.md`
   - Product/technical/safety goals expected after completing the full roadmap.

## Record 2 — Immediate phase plan summary

The active implementation phase is Phase 1. The canonical detailed plan is `docs/phase-plans/phase-1-readonly-cli.md`. The summary below is included for review convenience.

### Phase 1 — Workspace and read-only CLI MVP

- [x] Create Rust workspace root `Cargo.toml`.
- [x] Create `crates/anahtar-core`.
- [x] Create `crates/anahtar-cli`.
- [x] Define core data structures for safe output:
  - [x] `VaultInfo`
  - [x] `EntrySummary`
  - [x] `EntryDetail`
  - [x] `KdbxVersion`
- [x] Implement `inspect`:
  - [x] read KDBX header without password
  - [x] print KDBX version and file metadata
- [x] Implement unlock/open path:
  - [x] prompt password with hidden input
  - [x] open DB via `keepass::Database::open`
  - [x] return safe error messages without secrets
- [x] Implement `list`:
  - [x] traverse groups/entries
  - [x] show group path, title, username, url, id
  - [x] never show password
- [x] Implement `search`:
  - [x] case-insensitive search over title/username/url/notes
  - [x] show safe summaries only
- [x] Implement `show`:
  - [x] show one entry by id or exact title
  - [x] hide password by default
  - [x] require `--reveal-password` for password display
- [x] Add `--json` output for inspect/list/search/show to support future GUI integration.

#### Phase 1 exit criteria

- [x] `anahtar inspect` works without a password and reports KDBX version.
- [x] `anahtar list/search/show` work after password prompt.
- [x] Passwords are never printed unless an explicit reveal flag is used.
- [x] Commands work against `assets/private-vault.kdbx41.test.kdbx` and synthetic fixtures.

### Phase 2 — Upgrade/save-as CLI

- [x] Implement `upgrade` command:
  - [x] input KDBX3/KDBX4.0/KDBX4.1
  - [x] output explicit path only
  - [x] never overwrite existing output unless `--force` is passed
  - [x] set output version to KDBX4.1
  - [x] save to temp file
  - [x] reopen temp file and compare group/entry counts
  - [x] atomic rename to final output
- [x] Add warning if input is not KDBX4.1.
- [x] Add `--dry-run` to report what would happen.
- [x] Add manual Strongbox verification instructions after upgrade.

#### Phase 2 exit criteria

- [x] `anahtar upgrade` creates a KDBX4.1 output file without modifying the input.
- [x] Output is reopened and count-checked automatically.
- [x] Generated output opens in Strongbox manually.
- [x] Existing output is not overwritten unless explicitly forced.
- [x] `input == output` is rejected before save/unlock-sensitive write logic.
- [x] Temp output collision and failure cleanup are handled.

### Phase 3 — Minimal write commands

- [ ] Implement `add` with save-as output first.
- [ ] Implement `edit` with save-as output first.
- [ ] Implement `delete` with save-as output first.
- [ ] Add automatic backup policy before any in-place operation.
- [ ] Keep in-place save disabled until save-as flow is trusted.

#### Phase 3 exit criteria

- [x] `add/edit/delete` operate on a save-as output path.
- [x] Modified output reopens in Anahtar and Strongbox.
- [x] In-place save remains disabled unless backup/atomic/reopen verification is implemented.
- [x] `add` creates KDBX4.1 output, persists after reopen, and opens in Strongbox.
- [x] `edit` creates KDBX4.1 output, persists after reopen, and opens in Strongbox.
- [x] `delete` creates KDBX4.1 output, removes the entry after reopen, and opens in Strongbox.
- [x] Existing output protection, `input == output` rejection, temp save, and reopen verification are implemented for write commands.

### Phase 4 — Daily-use CLI polish

- [ ] Implement password generator.
- [ ] Implement clipboard copy with timed clear.
- [ ] Implement config file for default vault path.
- [ ] Implement shell completion.
- [ ] Add TOTP display if fields are compatible.

#### Phase 4 exit criteria

- CLI is usable for common daily retrieval tasks.
- Clipboard copy clears after a timeout.
- Default vault config is supported.
- No secrets appear in logs or normal command output.

### Phase 5 — GUI alpha planning

- [ ] Choose Tauri frontend stack after CLI core stabilizes.
- [ ] Reuse `anahtar-core` APIs from GUI commands.
- [ ] Implement open/unlock/search/list/detail/copy first.
- [ ] Add edit/save-as only after CLI write path is stable.

#### Phase 5 exit criteria

- GUI can open/unlock/search/list/detail/copy using `anahtar-core`.
- GUI write actions are either disabled or use the same save-as safety model as CLI.
- The GUI can be packaged locally for macOS first, with Windows/Linux left as follow-up packaging work.

## Record 3 — Final goal reference

The final goal setting is recorded separately in `docs/anahtar-final-goals.md`.

This implementation plan should be evaluated against that final goal record, but it should not duplicate or rewrite the final goal text on every phase update.

## Verification

### Automated verification

- [ ] `cargo test --workspace`
- [ ] Run CLI against synthetic upstream fixtures.
- [ ] Run CLI against `assets/private-vault.kdbx41.test.kdbx` only, not the original local cloud-synced vault file.
- [ ] Verify `inspect` reports:
  - [ ] backup file: KDBX 4.0
  - [ ] test upgraded file: KDBX 4.1
- [ ] Verify `list/search/show` never prints passwords by default.
- [ ] Verify `upgrade` output reopens with the same password.
- [ ] Verify group/entry counts match after upgrade.

### Manual verification

- [ ] Open generated KDBX4.1 files in Strongbox.
- [ ] Confirm unlock works.
- [ ] Confirm important entries are visible.
- [ ] Confirm username/password/url/notes/custom fields look correct.
- [ ] Confirm TOTP entries if present.
- [ ] Optionally edit/save in Strongbox, then reopen with Anahtar CLI.

## Non-goals for the first implementation

- Browser extension
- Mobile/iOS app
- Sync engine
- Passkeys
- Shared vaults
- In-place modification of the active cloud-synced vault
- Custom KDBX crypto implementation
