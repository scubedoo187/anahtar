# Anahtar KDBX research report

Date: 2026-06-10  
Goal: 개인 사용용 KeePass `.kdbx` CLI를 먼저 만들고, 검증된 core를 재사용해 Mac/Windows/Linux GUI까지 확장한다. 최종 목표는 paid password-manager 등 유료 password manager 비용을 0으로 낮추는 것이다.

## 1. Executive summary

### 결론

직접 개발은 가능하다. 다만 직접 `.kdbx` 포맷/암호화를 구현하는 것은 피해야 한다. 첫 MVP는 **검증된 KDBX 라이브러리 위에 read-only CLI → write-capable CLI → GUI** 순서로 가는 것이 현실적이다.

현재 후보 중 가장 실용적인 출발점은 다음 순서다.

1. **Rust `keepass` crate (`sseemayer/keepass-rs`)**
   - MIT license.
   - KDB, KDBX3, KDBX4 read 지원.
   - `save_kdbx4` feature로 KDBX4.1 writing을 실험적으로 지원.
   - 2026-06-07 기준 최신 업데이트, 다운로드/사용량이 Rust 후보 중 가장 높음.
   - CLI와 GUI core를 Rust로 공유하기에 좋음.
2. **TypeScript `kdbxweb`**
   - MIT license.
   - KeeWeb 생태계에서 오래 사용된 KDBX implementation.
   - GUI/prototype 속도는 좋지만, core를 JS에 두는 보안/패키징/메모리 관리상 찝찝함이 있음.
3. **Python `pykeepass`**
   - write 가능한 mature library.
   - 단, GPLv3이며 Python runtime 기반이라 크로스플랫폼 GUI 앱의 장기 core로는 덜 적합.
4. **Rust `kdbx-rs`**
   - read/write가 명확하나 GPL-3.0+.
   - 개인용은 괜찮지만, 추후 배포/라이선스 제약이 커질 수 있음.

### 추천 스택

**Rust core + Rust CLI + Tauri GUI**를 1차 방향으로 추천한다.

```text
anahtar/
  crates/
    anahtar-core/   # KDBX open/read/write/search/update abstraction
    anahtar-cli/    # CLI UX
    anahtar-gui/    # later: Tauri app using same core
```

핵심 rationale:

- CLI와 GUI가 같은 core를 공유할 수 있다.
- Mac/Windows/Linux 배포 경로가 자연스럽다.
- Tauri는 Electron보다 가볍고 Rust core와 잘 맞는다.
- 비밀번호 관리 도메인에서는 JS/Python보다 Rust binary가 장기적으로 운영하기 편하다.
- 단, `.kdbx` write compatibility를 prototype 단계에서 반드시 검증해야 한다.

## 2. KDBX 포맷 이해: 왜 library-first가 필요한가

KeePass `.kdbx`는 단순 암호화 JSON/SQLite가 아니다. 일반적인 KeePass 2 database는 다음을 포함한다.

- 파일 포맷 버전: KDBX 3.x, KDBX 4.x, KDBX 4.1 등
- 마스터 키 구성: password, key file, challenge-response 일부 구현
- KDF: AES-KDF, Argon2d, Argon2id
- Outer cipher: AES, ChaCha20, Twofish 일부 구현
- Inner protected stream cipher: Salsa20, ChaCha20 등
- 압축: gzip
- XML payload: group, entry, custom fields, history, icon, timestamps, auto-type 등
- KDBX4 integrity: header hash, HMAC block stream
- attachment/binary pool
- protected values: password 등 특정 XML 값의 inner stream 보호
- 저장 안정성: atomic write, backup, original preservation, conflict handling

따라서 MVP에서도 아래 원칙이 중요하다.

- 암호화 primitive 및 KDBX serializer를 직접 구현하지 않는다.
- 라이브러리의 read/write roundtrip compatibility를 테스트한다.
- 원본 vault에 직접 저장하기 전 backup + temp file + atomic rename 전략을 쓴다.
- 기존 KeePassXC/Strongbox/Keepassium에서 만든 DB를 fixture로 검증한다.

## 3. Rust library 후보

조사 소스:

- `cargo search keepass kdbx`
- crates.io API
- 각 crate README/Cargo.toml/local registry source
- GitHub API 일부

### 3.1 `keepass` crate / `sseemayer/keepass-rs`

- Crate: `keepass = "0.13.8"`
- Repository: <https://github.com/sseemayer/keepass-rs>
- License: MIT
- Updated: 2026-06-07
- Downloads: 174,620 total / 23,446 recent
- GitHub: 163 stars, 58 forks, archived=false
- Description: KeePass `.kdbx` database file parser

README states:

> Rust KeePass database file parser for KDB, KDBX3 and KDBX4, with experimental support for KDBX4.1 writing.

Cargo features observed:

- default: empty
- `save_kdbx4`
- `utilities`
- `serialization`
- `totp`
- `challenge_response`

Observed save support in source:

```rust
impl Database {
    pub fn save(&self, destination: impl Write, key: DatabaseKey) -> Result<(), DatabaseSaveError> {
        match self.config.version {
            DatabaseVersion::KDB(_) => Err(DatabaseSaveError::UnsupportedVersion),
            DatabaseVersion::KDB2(_) => Err(DatabaseSaveError::UnsupportedVersion),
            DatabaseVersion::KDB3(_) => Err(DatabaseSaveError::UnsupportedVersion),
            DatabaseVersion::KDB4(_) => dump_kdbx4(self, &key, destination),
        }
    }
}
```

Default config appears to target KDBX4 with ChaCha20 inner cipher and Argon2 KDF.

Strengths:

- Best Rust candidate by maturity/activity/usage.
- MIT license keeps future distribution flexible.
- KDBX3/KDBX4 read support is valuable for existing vaults.
- TOTP support exists as feature.
- Utility binaries exist; useful reference for CLI design.
- `save_kdbx4` is exactly what the MVP write path needs.

Weaknesses / risks:

- Writing is described as **experimental**.
- Save only supports KDBX4; KDBX3 files cannot be saved as-is through this API.
- Need to test whether opening a KDBX3 vault and saving as KDBX4 is supported/safe enough.
- Need to inspect API ergonomics for entry/group mutation.
- Need compatibility testing with KeePassXC, Strongbox, Keepassium.

Assessment:

- **Recommended primary candidate for Rust MVP.**
- Start read-only with this library.
- Enable `save_kdbx4` only after roundtrip tests pass.

### 3.2 `kdbx-rs`

- Crate: `kdbx-rs = "0.5.2"`
- Repository: <https://gitlab.com/tonyfinn/kdbx-rs>
- License: GPL-3.0+
- Updated: 2024-10-08
- Downloads: 28,607 total / 434 recent
- Description: Keepass 2 KDBX password database parsing and creation

README explicitly shows:

- Opening a KDBX file
- Unlocking with `CompositeKey`
- Creating a new database
- Saving with `kdbx.write(&mut file)`

README comparison table claims, as of May 2020:

- KDBX4: yes
- KDBX3: read-only
- AES/Argon2/ChaCha20/TwoFish support
- keyfile auth yes
- custom fields yes
- entry history yes
- memory protection no

Strengths:

- Write support is explicit and central to its API.
- Supports KDBX4 creation/writing.
- API appears conceptually clean: locked/unlocked KDBX states, `database_mut`, `write`.

Weaknesses / risks:

- GPL-3.0+ license can constrain future distribution or mixed-license usage.
- Less active and lower usage than `keepass`.
- README comparison is old; still need live compatibility tests.
- Memory protection listed as no.

Assessment:

- Good fallback/prototype reference if `keepass` write support is insufficient.
- License makes it less attractive as the main long-term core unless the whole project accepts GPL.

### 3.3 `keepass-rs` crate / `meission/keepass-rs`

- Crate: `keepass-rs = "0.1.0"`
- Repository in Cargo.toml: <https://github.com/meission/keepass-rs>
- License: MIT OR MulanPSL-2.0
- Updated: 2026-06-03
- Downloads: 12 recent/total at time of research
- Description: platform-independent KeePass library supporting KDB, KDBX 3.1, KDBX 4.0

README claims a very broad feature set:

- KDB v1 read/write
- KDBX 3.1 read/write
- KDBX 4.0 read/write
- AES/ChaCha20/Twofish/Salsa20
- AES-KDF/Argon2d/Argon2id
- Regex search
- Three-way merge
- Integrity repair
- OTP
- Digital signatures
- Change tracking
- Fuzz testing
- Android/HarmonyOS/Flutter bridges

Strengths:

- Feature claims are impressive.
- Explicit read/write support across KDB/KDBX3/KDBX4.
- Modern update date.

Weaknesses / risks:

- Very new crate with only 12 downloads observed.
- GitHub repository returned 404 via GitHub API at research time, even though crate metadata lists it.
- Feature breadth seems too large for a brand-new crate; requires careful verification.
- License includes MulanPSL option, which may require review if redistributed.

Assessment:

- Not recommended as primary until repository availability and tests are verified.
- Could be revisited if it becomes established.

### 3.4 `rust-kpdb`

- Crate: `rust-kpdb = "0.6.0"`, library name `kpdb`
- Repository: <https://github.com/sru-systems/rust-kpdb>
- License: MIT/Apache-2.0
- Updated: 2026-03-31
- Downloads: 12,042 total / 138 recent
- GitHub: 14 stars, 5 forks
- Description: Library for reading/writing KeePass 2 and KeePassX databases

README shows:

- Create DB
- Add groups/entries
- Open existing `.kdbx`
- Save to new `.kdbx`
- Password + keyfile support

Strengths:

- Permissive license.
- Read/write API exists.
- Updated recently.

Weaknesses / risks:

- Older feature support appears limited.
- Historical comparison says no KDBX4 and no Argon2.
- Modern KeePassXC/Strongbox databases are likely KDBX4/Argon2, so this may not handle real-world vaults.

Assessment:

- Not ideal for this project unless tests show modern KDBX4/Argon2 support has been added.
- Could be useful for legacy KDBX3 only.

### 3.5 `kdbx4`

- Crate: `kdbx4 = "0.5.1"`
- Repository: <https://github.com/makovich/kdbx4>
- License: MIT OR Unlicense
- Updated on crates.io: 2021-11-10
- Downloads: 13,048 total / 350 recent
- Description: KeePass KDBX4 file reader

Strengths:

- Focused KDBX4 reader.
- Permissive license.

Weaknesses / risks:

- Read-only.
- Low activity/usage.
- Does not solve write-capable CLI goal.

Assessment:

- Not suitable as core for Anahtar, except as reference/diagnostic reader.

### 3.6 `kdbx` crate / `daxartio/kdbx`

- Crate: `kdbx = "0.13.0"`
- Repository: <https://github.com/daxartio/kdbx>
- License: MIT
- Updated: 2026-04-29
- Description: A secure hole for your passwords (KeePass CLI)

Strengths:

- Existing CLI project may inform UX/commands.
- MIT license.

Weaknesses / risks:

- Need deeper inspection to know whether it is a library-quality core or primarily an app.
- Likely not the best dependency layer if `keepass` gives lower-level control.

Assessment:

- Good CLI reference, not primary core candidate yet.

## 4. TypeScript / JavaScript 후보

### 4.1 `kdbxweb`

- NPM: `kdbxweb@2.1.1`
- Repository: <https://github.com/keeweb/kdbxweb>
- License: MIT
- NPM modified: 2022-06-19
- GitHub: 455 stars, 66 forks, updated 2026-05-16, archived=false
- Dependencies: `@xmldom/xmldom`, `fflate`
- Description: Kdbx KeePass database reader for web

Strengths:

- Established in KeeWeb ecosystem.
- Browser/web/Electron/Tauri frontend integration is straightforward.
- MIT license.
- Likely one of the best JS options for KDBX manipulation.

Weaknesses / risks:

- NPM package itself has not been published recently.
- JS runtime memory handling is less attractive for a password manager core.
- If CLI core is JS, packaging secure single binaries across OSes becomes less clean.
- Need inspect write support and current KDBX4 compatibility in detail before adopting.

Assessment:

- Best fallback if Rust KDBX writing is too immature.
- Particularly attractive for fast GUI proof-of-concept.
- Not recommended as first long-term core unless Rust path fails.

## 5. Python 후보

### 5.1 `pykeepass`

- PyPI: `pykeepass 4.1.1.post1`
- Repository: <https://github.com/libkeepass/pykeepass>
- License: GPLv3
- GitHub: 501 stars, 104 forks, updated 2026-06-04
- PyPI description: “This library allows you to write entries to a KeePass database.”

Strengths:

- Mature and commonly used for automation.
- Write support is explicit.
- Great for quick scripts, migration, fixtures, compatibility testing.

Weaknesses / risks:

- GPLv3.
- Python packaging/runtime less ideal for a polished cross-platform password manager app.
- GUI integration would likely need PySide/PyQt/Tk/etc.; not the desired long-term path.

Assessment:

- Excellent research/prototyping/test helper.
- Not recommended as the main Anahtar core.

## 6. Existing open-source clients as references/alternatives

### KeePassXC

- Repository: <https://github.com/keepassxreboot/keepassxc>
- GitHub: 27,581 stars, 1,813 forks, updated 2026-06-10
- Cross-platform desktop app: macOS/Windows/Linux
- Mature, free, open-source.

Use for:

- Compatibility oracle.
- Fixture generation.
- CLI/UX/reference behavior.
- Potential daily driver during Anahtar development.

Not a direct dependency because it is a full app in C++/Qt rather than a reusable Rust core.

### KeePassium

- Repository: <https://github.com/keepassium/KeePassium>
- GitHub: 1,618 stars, 140 forks, updated 2026-06-09
- iOS KeePass client.

Use for:

- iOS UX/reference behavior later.
- File provider/cloud storage/AutoFill reference.

### KeePassDX

- Repository: <https://github.com/Kunzisoft/KeePassDX>
- GitHub: 6,882 stars, 370 forks, updated 2026-06-09
- Android KeePass client.

Use for:

- Android/reference only; not in first scope.

## 7. Recommended MVP scope after research

### MVP 0: read-only CLI

Purpose: prove that real existing vaults can be opened and navigated safely.

Commands:

```bash
anahtar version
anahtar inspect vault.kdbx
anahtar list vault.kdbx
anahtar search vault.kdbx github
anahtar show vault.kdbx <entry-id>
anahtar show vault.kdbx <entry-id> --reveal-password
```

Rules:

- Master password should be prompted through TTY, never command-line argument by default.
- Password fields hidden by default.
- Output should support `--json` for GUI/core integration tests.
- No write path yet.

### MVP 1: safe write path

Purpose: prove add/edit/delete/save without corrupting vaults.

Commands:

```bash
anahtar add vault.kdbx
anahtar edit vault.kdbx <entry-id>
anahtar delete vault.kdbx <entry-id>
anahtar save-as vault.kdbx output.kdbx
```

Mandatory safeguards:

- Default write mode should be `save-as` or `--in-place` with explicit confirmation.
- Before in-place save: create timestamped backup.
- Use temp file in same directory, fsync where possible, then atomic rename.
- After write: reopen saved file with same key and run basic integrity/list check.
- Never log master password, entry password, or raw XML.

### MVP 2: daily-use CLI

```bash
anahtar get github --username
anahtar get github --password --copy
anahtar generate
anahtar totp github
```

Features:

- Clipboard copy and delayed clear.
- Search ranking.
- Password generator.
- TOTP if library support is stable.
- Shell completion.
- Config file for default vault path.

### GUI alpha

Use the same `anahtar-core` API.

Features:

- Open DB
- Unlock
- Search
- Entry list/detail
- Copy username/password
- Add/edit/delete
- Save/save-as
- Backup indicator

Avoid initially:

- Browser extension
- iOS AutoFill
- Sync engine
- Shared vaults
- Passkeys
- Secure enclave abstractions beyond OS keychain unlock convenience

## 8. Recommended first technical spike

Before committing to a full codebase, run a focused compatibility spike.

### Spike A: `keepass` crate read/write proof

Use `keepass = { version = "0.13.8", features = ["save_kdbx4", "totp"] }`.

Test cases:

1. Create test KDBX4 database with KeePassXC.
2. Open with `keepass` using password only.
3. Enumerate groups and entries.
4. Read custom fields and notes.
5. Add a test entry.
6. Save as new `.kdbx`.
7. Reopen saved `.kdbx` with:
   - Anahtar prototype
   - KeePassXC
   - Strongbox if available
8. Verify no data loss for:
   - title
   - username
   - password
   - url
   - notes
   - custom fields
   - groups
   - icons if present
   - history if present
   - attachments if present

Pass criteria:

- Read-only works on real personal-style fixture.
- Saved output opens in KeePassXC.
- Original file remains untouched.
- Existing entries remain intact after roundtrip.

Fail/pivot criteria:

- KDBX4 save corrupts or drops common metadata.
- Strongbox/KeePassXC rejects output.
- Library mutation API is too limited for safe editing.

If fail, evaluate:

1. `kdbx-rs` despite GPL implications.
2. `kdbxweb` as JS core fallback.
3. Hybrid approach: initial CLI uses existing KeePassXC CLI automation only for write operations. This is less ideal but can avoid corruption while building UX.

## 9. Security posture for Anahtar

### Non-negotiables

- No custom cryptographic implementation.
- No master password in CLI args by default.
- No secrets in logs/errors/panic reports.
- Password reveal requires explicit flag/action.
- Clipboard clear timer for copied secrets.
- Save should be backup + atomic.
- Test vaults must not contain real secrets.

### Good early hardening

- Use `secrecy`/`zeroize` for internal secret wrappers where compatible.
- Minimize lifetime of plaintext passwords in memory.
- Avoid debug-printing database structs.
- Set panic hook to avoid dumping sensitive context.
- Keep fixtures synthetic.
- Add property/roundtrip tests for non-secret metadata.

### Later hardening

- OS keychain integration for caching unlock material or file-specific unlock tokens.
- Touch ID/Windows Hello integration in GUI.
- Lock timeout.
- Clipboard ownership checks where possible.
- Audit dependencies with `cargo audit`.
- Reproducible release pipeline.

## 10. Cost/effort estimate after research

Assuming Rust + `keepass` path passes the spike:

| Stage | Scope | Estimate |
|---|---|---:|
| Research/spike | library proof, fixtures, compatibility | 2-5 days |
| MVP 0 | read-only CLI | 3-7 days |
| MVP 1 | safe write CLI | 1-2 weeks |
| MVP 2 | daily-use CLI polish | 1-2 weeks |
| GUI alpha | Tauri desktop GUI | 3-6 weeks |
| Daily-driver hardening | backups, lock policies, UX polish, packaging | 1-2 months |

If Rust write support fails and we pivot to `kdbxweb` or `kdbx-rs`, add roughly 3-7 days for re-spike and architecture adjustment.

## 11. Decision recommendation

Recommended next step:

1. Keep the project language/core direction as **Rust-first**.
2. Use **`keepass` crate** for the first spike.
3. Treat KDBX writing as untrusted until compatibility tests pass.
4. Build read-only CLI first.
5. Only after read-only CLI is stable, add write commands behind strong safety defaults.

Recommended dependency posture:

```toml
keepass = { version = "0.13.8", features = ["save_kdbx4", "totp"] }
```

But for MVP 0, we may omit `save_kdbx4` until write work begins.

## 12. Open questions for later planning

These do not block research but should be decided before implementation:

1. License goal: purely personal/private, or potentially open-source/distributed?
2. Is GPL acceptable if the best write library requires it?
3. Target first platform for GUI: macOS only first, or cross-platform from day one?
4. Existing vault characteristics:
   - KDBX version?
   - password only or key file?
   - Argon2 or AES-KDF?
   - attachments/custom icons/TOTP/custom fields?
5. Strongbox-specific fields or metadata that must roundtrip?
6. Sync requirement: local file only initially, or cloud storage/Dropbox/Syncthing in MVP?

## Appendix A. Observed package metadata

| Candidate | Version | License | Updated | Recent usage | Notes |
|---|---:|---|---|---:|---|
| Rust `keepass` | 0.13.8 | MIT | 2026-06-07 | 23,446 recent downloads | Best primary candidate; experimental KDBX4.1 write |
| Rust `kdbx-rs` | 0.5.2 | GPL-3.0+ | 2024-10-08 | 434 recent downloads | Explicit write support; license concern |
| Rust `kdbx4` | 0.5.1 | MIT OR Unlicense | 2021-11-10 | 350 recent downloads | Read-only KDBX4 |
| Rust `rust-kpdb` | 0.6.0 | MIT/Apache-2.0 | 2026-03-31 | 138 recent downloads | Write API but likely modern KDBX4/Argon2 concern |
| Rust `keepass-rs` | 0.1.0 | MIT OR MulanPSL-2.0 | 2026-06-03 | 12 downloads | New/unverified despite broad claims |
| TS `kdbxweb` | 2.1.1 | MIT | NPM 2022-06-19 | n/a | Mature JS option; GitHub active |
| Python `pykeepass` | 4.1.1.post1 | GPLv3 | 2026-06-04 GitHub | n/a | Good automation/test helper |

## Appendix B. Research commands used

```bash
cargo search keepass kdbx --limit 10
cargo info keepass
cargo info kdbx-rs
cargo info kdbx4
cargo info rust-kpdb
cargo info keepass-rs
cargo info kdbx
npm view kdbxweb version license description repository.url time.modified dependencies
python3 -m pip index versions pykeepass
```

Also inspected local Cargo registry source for README/Cargo.toml/API symbols after `cargo info` fetched crates.
