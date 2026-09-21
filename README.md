# my-axum-app

基于 Axum 0.8 的 Rust 后端服务。

## 运行

```bash
cargo run
```

服务默认监听 `http://127.0.0.1:3000`。

## 热重载（开发）

```bash
cargo dev                 # 等价于 cargo watch -c -q -w src -w Cargo.toml -x run
# 或
bash scripts/dev.sh       # 自动安装 cargo-watch，并额外监听 config/ migrations/
```

首次使用需安装：

```bash
cargo install cargo-watch
```

保存源码后会触发重新编译并自动重启服务，日志级别默认 `debug,tower_http=debug`。

## 日志

| 环境变量 | 说明 |
| --- | --- |
| `RUST_LOG` | 日志过滤规则，默认 `info,tower_http=debug` |
| `LOG_JSON` | `1`/`true` 时输出 JSON 结构化日志 |

```bash
RUST_LOG=debug cargo run
LOG_JSON=1 RUST_LOG=info cargo run
```

每个请求会生成携带 `method`、`uri`、`request_id` 的 span，响应日志记录耗时与状态码，并回写 `x-request-id` 响应头。

## 接口

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/` | 示例接口 |
| GET | `/health` | 健康检查 |

## 开发约定

项目规范与脚手架见 `.codebuddy/skills/axum-backend/`，新增功能前请查阅其中文档。
