#!/usr/bin/env bash
# 生成 axum 业务域骨架：src/features/<feature>/{mod,handler,service,repository,model}.rs + 迁移 SQL
# 用法: bash .codebuddy/skills/axum-backend/scripts/new_feature.sh <feature_snake_case>
# 示例: bash .codebuddy/skills/axum-backend/scripts/new_feature.sh user_profile
set -euo pipefail

FEATURE="${1:?用法: new_feature.sh <feature_snake_case>}"

if ! [[ "$FEATURE" =~ ^[a-z][a-z0-9]*(_[a-z0-9]+)*$ ]]; then
    echo "错误: 名称必须是 snake_case（小写字母、数字、下划线）" >&2
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SKILL_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
ROOT="$(cd "$SKILL_DIR/../.." && pwd)"
TPL="$SKILL_DIR/assets/templates"

# user_profile -> UserProfile
FEATURE_PASCAL="$(echo "$FEATURE" | awk -F_ '{for (i = 1; i <= NF; i++) printf "%s%s", toupper(substr($i, 1, 1)), substr($i, 2)}')"
FEATURE_PLURAL="${FEATURE}s"
TS="$(date +%Y%m%d%H%M%S)"

DEST="$ROOT/src/features/$FEATURE"
if [[ -d "$DEST" ]]; then
    echo "错误: $DEST 已存在，未做任何改动" >&2
    exit 1
fi

mkdir -p "$DEST" "$ROOT/migrations" "$ROOT/tests"

render() {
    sed -e "s/__TIMESTAMP__/${TS}/g" \
        -e "s/__features__/${FEATURE_PLURAL}/g" \
        -e "s/__Feature__/${FEATURE_PASCAL}/g" \
        -e "s/__feature__/${FEATURE}/g" \
        "$TPL/$1" >"$2"
    echo "  创建 $2"
}

echo "生成业务域: $FEATURE (${FEATURE_PASCAL})"
render mod.rs "$DEST/mod.rs"
render handler.rs "$DEST/handler.rs"
render service.rs "$DEST/service.rs"
render repository.rs "$DEST/repository.rs"
render model.rs "$DEST/model.rs"
render migration.sql "$ROOT/migrations/${TS}_create_${FEATURE_PLURAL}_table.sql"

cat <<EOF

骨架已生成，接下来请手工完成（脚本不会自动修改已有文件）：

1. 在 src/features/mod.rs 添加:  pub mod $FEATURE;
2. 在 src/router.rs 挂载:        .nest("/${FEATURE_PLURAL}", features::${FEATURE}::router())
3. 检查 migrations/${TS}_create_${FEATURE_PLURAL}_table.sql 的字段是否符合需求
   （复数形式按英文规则自动加 s，如需 y->ies 等变形请手动修正）
4. 运行: sqlx migrate run && cargo sqlx prepare
5. 在 tests/${FEATURE}.rs 补集成测试，在 src/docs.rs 的 #[openapi(paths(...), components(...))] 注册
6. 自检: bash .codebuddy/skills/axum-backend/scripts/check.sh
EOF
