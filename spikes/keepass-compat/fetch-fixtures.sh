#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
base="https://raw.githubusercontent.com/sseemayer/keepass-rs/master/tests/resources"
files=(
  test_db_with_password.kdbx
  test_db_kdbx4_with_password_aes.kdbx
  test_db_kdbx4_with_password_argon2.kdbx
  test_db_kdbx4_with_password_argon2_chacha20.kdbx
  test_db_kdbx4_with_password_argon2id.kdbx
  test_db_kdbx4_with_totp_entry.kdbx
  test_db_kdbx41_features.kdbx
)
mkdir -p fixtures
for f in "${files[@]}"; do
  echo "fetch $f"
  curl -fsSL "$base/$f" -o "fixtures/$f"
done
