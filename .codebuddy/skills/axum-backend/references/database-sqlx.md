# sqlx + PostgreSQL 数据访问

## 连接池

```rust
use sqlx::postgres::PgPoolOptions;

let pool = PgPoolOptions::new()
    .max_connections(settings.database.max_connections)   // 默认 10
    .min_connections(1)
    .acquire_timeout(std::time::Duration::from_secs(5))
    .idle_timeout(std::time::Duration::from_secs(600))
    .connect(&settings.database.url)
    .await?;
```

- 池只创建一次，放进 `AppState`；handler 通过 `State(state)` 取 `&state.pool`。
- 启动时执行 `sqlx::migrate!().run(&pool).await?`，禁止依赖外部手动执行迁移再启动。

## 迁移

```bash
sqlx migrate add -r create_users_table    # 生成 migrations/<ts>_create_users_table.sql（up/down 两份）
sqlx migrate run                          # 应用
sqlx migrate revert                       # 回滚
cargo sqlx prepare                        # 生成 .sqlx 离线缓存，提交进仓库
```

迁移文件命名：`YYYYMMDDHHMMSS_<action>_<table>.sql`（up）与 `.down.sql`（down）。
`down` 迁移必须真实可回滚，不允许留空。

建表示例：

```sql
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE users (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email       TEXT NOT NULL UNIQUE,
    name        TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_users_created_at ON users (created_at DESC);
```

约定：主键统一 `UUID`（`gen_random_uuid()`），时间统一 `TIMESTAMPTZ` 并带默认值。

## repository 写法

```rust
use sqlx::PgPool;
use crate::error::{AppError, AppResult};
use super::model::{CreateUserRequest, UpdateUserRequest, User};

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> AppResult<User> {
    sqlx::query_as!(
        User,
        r#"SELECT id, email, name, created_at, updated_at FROM users WHERE id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

pub async fn list(pool: &PgPool, limit: i64, offset: i64) -> AppResult<Vec<User>> {
    sqlx::query_as!(
        User,
        r#"SELECT id, email, name, created_at, updated_at
           FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2"#,
        limit,
        offset
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

pub async fn create(pool: &PgPool, req: CreateUserRequest) -> AppResult<User> {
    sqlx::query_as!(
        User,
        r#"INSERT INTO users (email, name) VALUES ($1, $2)
           RETURNING id, email, name, created_at, updated_at"#,
        req.email,
        req.name
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}
```

要点：

- 使用 `query_as!` / `query!` 编译期校验宏，禁止 `query_unchecked!`。
- 查询只 SELECT 需要的列，禁止 `SELECT *`（宏无法推断可空性，且易破坏类型映射）。
- 单条查询用 `fetch_optional` + `ok_or(AppError::NotFound)`，不要用 `fetch_one` 后靠 `RowNotFound` 转换。
- 可空列需显式 `AS "col: Option<T>"` 或 `col?` 标注。
- 所有函数第一个参数为 `&PgPool`，返回 `AppResult<T>`。
- 动态条件用 `sqlx::QueryBuilder`，禁止字符串拼接 SQL。

## 事务

跨表写入在 `service` 层开启事务，把 `&mut PgConnection`/`Transaction` 传给 repository：

```rust
pub async fn transfer(pool: &PgPool, from: Uuid, to: Uuid, amount: i64) -> AppResult<()> {
    let mut tx = pool.begin().await?;

    repository::deduct(&mut *tx, from, amount).await?;   // repository 接收 &mut PgConnection
    repository::add(&mut *tx, to, amount).await?;

    tx.commit().await?;
    Ok(())
}
```

- repository 中接收事务的函数签名统一为 `executor: impl sqlx::PgExecutor<'_>` 或 `&mut PgConnection`，以便复用。
- 事务内禁止调用外部 HTTP、发送邮件等慢操作。
- 出错时显式 `tx.rollback().await?`，或依赖 `Drop` 自动回滚（默认行为）。

## 离线编译与 CI

- 提交代码前运行 `cargo sqlx prepare`，确保 `.sqlx/` 与 `migrations/` 同步。
- CI 中设置 `SQLX_OFFLINE=true`，避免无数据库环境编译失败。
- 修改 SQL 后忘记 prepare 会导致 CI 报 `error: error returned from database`，属于预期失败。

## 测试数据库

- 集成测试使用独立数据库（`DATABASE_URL` 指向 `my_axum_app_test`），见 `testing-openapi.md`。
- 每个测试用例用事务包裹并在结束时回滚，或使用 `TRUNCATE ... RESTART IDENTITY CASCADE` 清理。
