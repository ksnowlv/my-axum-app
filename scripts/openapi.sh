#!/usr/bin/env bash
# 生成 OpenAPI 文档文件
#
# 用法:
#   bash scripts/openapi.sh                    # 输出到 openapi.json
#   bash scripts/openapi.sh docs/api.json      # 指定输出路径
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

OUTPUT="${1:-openapi.json}"

cargo run --quiet --bin openapi >"$OUTPUT"

echo "已生成: $OUTPUT"
