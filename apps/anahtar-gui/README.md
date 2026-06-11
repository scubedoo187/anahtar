# Anahtar GUI Alpha

Tauri + TypeScript GUI shell for Anahtar Phase 6.

## Development

```bash
cd apps/anahtar-gui
npm install
npm run dev
```

## Checks

```bash
cd apps/anahtar-gui
npm run typecheck
npm run build:frontend
cargo check --manifest-path src-tauri/Cargo.toml
```

The GUI Rust side calls into `anahtar-app`; KDBX logic stays in `anahtar-core`.
