# my-axum-app

基于 Axum 0.8 + PostgreSQL(sqlx) 的 Rust 后端服务。

## 运行

```bash
cp .env.example .env      # 至少配置 DATABASE_URL 与 JWT_SECRET
cargo sqlx migrate run    # 首次运行需执行迁移
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
| `RUST_LOG` | 日志过滤规则，默认 `info,tower_http=debug,sqlx=warn` |
| `LOG_JSON` | `1`/`true` 时输出 JSON 结构化日志 |

```bash
RUST_LOG=debug cargo run
LOG_JSON=1 RUST_LOG=info cargo run
```

每个请求会生成携带 `method`、`uri`、`request_id` 的 span，响应日志记录耗时与状态码，并回写 `x-request-id` 响应头。

## 配置

配置从 `.env` / 环境变量读取（`src/config.rs`）：

| 变量 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `DATABASE_URL` | 是 | - | PostgreSQL 连接串 |
| `JWT_SECRET` | 是 | - | JWT 密钥，长度至少 16 |
| `JWT_EXPIRATION_HOURS` | 否 | `24` | token 有效期 |
| `APP_ADDR` | 否 | `127.0.0.1:3000` | 监听地址 |
| `RUST_LOG` | 否 | `info` | 日志级别 |
| `LOG_JSON` | 否 | - | `1`/`true` 启用 JSON 日志 |

## 接口

### 示例 / 系统

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/` | 示例接口，返回统一包装 `{ code, message, data }` |
| GET | `/health` | 健康检查 |

### 用户

| 方法 | 路径 | 鉴权 | 说明 |
| --- | --- | --- | --- |
| POST | `/api/v1/users/register` | 否 | 用户注册，返回 token 与用户信息 |
| POST | `/api/v1/users/login` | 否 | 用户登录，返回 token 与用户信息 |
| GET | `/api/v1/users/{id}` | 否 | 查询指定用户信息 |
| GET | `/api/v1/users/me` | 是 | 查询当前登录用户信息 |
| PATCH | `/api/v1/users/me` | 是 | 更新当前用户昵称 / 头像 |

鉴权方式：`Authorization: Bearer <token>`。

请求示例：

```bash
# 注册
curl -X POST http://127.0.0.1:3000/api/v1/users/register \
  -H 'content-type: application/json' \
  -d '{"email":"alice@example.com","password":"password123","nickname":"Alice"}'

# 登录
curl -X POST http://127.0.0.1:3000/api/v1/users/login \
  -H 'content-type: application/json' \
  -d '{"email":"alice@example.com","password":"password123"}'

# 查询当前用户
curl http://127.0.0.1:3000/api/v1/users/me \
  -H 'authorization: Bearer <token>'

# 更新资料（字段可选，未提供则保持不变）
curl -X PATCH http://127.0.0.1:3000/api/v1/users/me \
  -H 'authorization: Bearer <token>' \
  -H 'content-type: application/json' \
  -d '{"nickname":"Bob","avatar_url":"https://example.com/a.png"}'

# 查询指定用户
curl http://127.0.0.1:3000/api/v1/users/<user_id>
```

响应统一包装：

```json
{ "code": 0, "message": "ok", "data": { } }
```

错误响应：HTTP 状态码 + `{ "code": <status>, "message": "<原因>" }`。常见取值：400 参数校验失败、401 未登录/凭证错误、404 用户不存在、409 邮箱已注册。

## API 文档

文档由 `utoipa` 从代码注解自动生成（OpenAPI 3.1）。

在线访问（服务启动后）：

- Swagger UI：<http://127.0.0.1:3000/swagger-ui/>
- OpenAPI JSON：<http://127.0.0.1:3000/api-docs/openapi.json>

导出为文件（CI / 分享用）：

```bash
bash scripts/openapi.sh                 # 生成 openapi.json
bash scripts/openapi.sh docs/api.json   # 指定路径
cargo run --bin openapi > openapi.json  # 等价写法
```

新增接口后需要同步三处，否则不会出现在文档中：

1. handler 上添加 `#[utoipa::path(...)]`
2. 在 `src/docs.rs` 的 `paths(...)` 中注册该 handler
3. 在 `components(schemas(...))` 中注册请求/响应结构体

## 目录结构

```text
src/
├── lib.rs                 # 模块声明
├── main.rs                # 启动入口
├── config.rs              # 配置加载
├── logging.rs             # tracing 初始化
├── error.rs               # AppError / ErrorBody
├── response.rs            # ApiResponse<T>
├── state.rs               # AppState
├── router.rs              # 根路由 + 中间件栈
├── middleware.rs          # Bearer Token 鉴权
├── security.rs            # 密码哈希 / JWT
├── extractor.rs           # ValidatedJson / CurrentUser
├── handlers.rs            # 全局处理器
├── docs.rs                # OpenAPI 定义
└── features/user/         # 用户域：handler / service / repository / model
migrations/                # sqlx 迁移
```

## 开发约定

项目规范与脚手架见 `.codebuddy/skills/axum-backend/`，新增功能前请查阅其中文档。
