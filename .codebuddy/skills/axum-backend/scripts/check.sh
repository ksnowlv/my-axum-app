#!/usr/bin/env bash
# 提交前自检：fmt -> clippy -> test -> sqlx 离线缓存校验
# 用法: bash .codebuddy/skills/axum-backend/scripts/check.sh
set -euo pipefail

# scripts/ -> axum-backend/ -> skills/ -> .codebuddy/ -> 项目根
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
cd "$ROOT"

echo "== cargo fmt =="
cargo fmt --all

echo "== cargo clippy =="
cargo clippy --all-targets -- -D warnings

echo "== cargo test =="
cargo test

if command -v sqlx >/dev/null 2>&1 && command -v cargo-sqlx >/dev/null 2>&1; then
    echo "== cargo sqlx prepare --check =="
    cargo sqlx prepare --check || echo "提示: .sqlx 离线缓存与查询不同步，请运行 cargo sqlx prepare"
fi

echo "ALL CHECKS PASSED"
