# tracing 日志约定

## 初始化（src/logging.rs）

```rust
use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init(level: &str, json: bool) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("{level},tower_http=debug,sqlx=warn")));

    let registry = tracing_subscriber::registry().with(filter);

    if json {
        registry.with(fmt::layer().json()).init();   // 生产：结构化 JSON
    } else {
        registry.with(fmt::layer().pretty()).init(); // 开发：可读输出
    }
}
```

- 在 `main.rs` 中于创建连接池之前初始化，保证后续日志不丢失。
- 通过 `RUST_LOG` 覆盖级别；默认 `info`。
- 生产环境使用 JSON 输出，便于日志采集。

## 记录日志

```rust
use tracing::{debug, error, info, warn, instrument};

#[instrument(skip(pool), fields(user_id = %user_id))]
pub async fn find_user(pool: &PgPool, user_id: Uuid) -> AppResult<User> {
    debug!("querying user");
    let user = repository::find_by_id(pool, user_id).await?;
    info!(email = %user.email, "user loaded");
    Ok(user)
}
```

约定：

- 级别用法：`error!` 不可恢复故障与 5xx；`warn!` 可降级异常（重试、限流命中）；`info!` 关键业务事件与生命周期事件；`debug!` 排查细节（SQL 参数、缓存命中）；`trace!` 高频底层。
- 用结构化字段而非格式化字符串：`info!(user_id = %id, action = "login")`，不要 `info!("user {} login", id)`。
- 服务层公开函数加 `#[instrument]`，`skip` 掉 `pool`、`password` 等大对象或敏感字段。
- 禁止 `println!` / `eprintln!` / `dbg!` 出现在 `src/` 下（clippy 之外再人工检查）。

## 敏感信息

- 禁止记录：密码、token、完整身份证/银行卡号、私钥。
- 邮箱、手机号需脱敏后记录（如 `a***@example.com`）。
- `#[instrument]` 默认会记录所有参数，敏感参数必须 `skip`。

## 与 TraceLayer 配合

- `TraceLayer` 自动产出 `request` span（method / uri / status / latency）。
- 在 middleware 中通过 `Span::current().record("user_id", ...)` 把上下文补进请求 span，避免每次手动传。
- 每个响应日志都应携带 `x-request-id`，排查时用其串联。

## 常见模式

```rust
// 耗时埋点
let start = std::time::Instant::now();
let result = do_work().await;
info!(elapsed_ms = start.elapsed().as_millis() as u64, "work done");

// 失败只记一次：错误已在 AppError::into_response 记录时，service 层只 warn 上下文
match external_call().await {
    Err(e) => { warn!(error = %e, "external call failed, fallback used"); fallback() }
    Ok(v) => v,
}
```

- 避免同一错误在多层重复 `error!`；底层抛 `AppError`，顶层统一记录。
