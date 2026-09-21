#!/usr/bin/env bash
# 本地开发热重载：监听源码变更，自动重新编译并重启服务
#
# 用法:
#   bash scripts/dev.sh                  # 默认监听 src / Cargo.toml / config / migrations
#   RUST_LOG=info bash scripts/dev.sh    # 自定义日志级别（默认 debug,tower_http=debug）
#   LOG_JSON=1 bash scripts/dev.sh       # JSON 结构化日志
#
# 依赖 cargo-watch；未安装时自动安装。
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if ! command -v cargo-watch >/dev/null 2>&1; then
    echo "== 未检测到 cargo-watch，正在安装 =="
    cargo install cargo-watch
fi

: "${RUST_LOG:=debug,tower_http=debug}"
export RUST_LOG

WATCH_PATHS=(-w src -w Cargo.toml)
for dir in config migrations; do
    [[ -d "$dir" ]] && WATCH_PATHS+=(-w "$dir")
done

echo "== 热重载已启动 | RUST_LOG=$RUST_LOG | 监听: ${WATCH_PATHS[*]} =="

# -c 清屏  -q 精简 cargo-watch 自身输出  -x run 变更后执行 cargo run
exec cargo watch -c -q "${WATCH_PATHS[@]}" -x run
