//! 全局日志初始化：tracing + tracing-subscriber
//!
//! - 级别：`RUST_LOG` 优先，未设置时使用传入的默认级别
//! - 格式：`LOG_JSON=1|true` 输出 JSON（生产/采集），否则输出可读格式

use tracing_subscriber::{EnvFilter, fmt};

/// 附加过滤规则：框架日志比业务日志更吵，单独降噪。
/// 注意 `sqlx_postgres` 与 `sqlx` 是两个不同的 target，需分别指定。
const EXTRA_DIRECTIVES: &str = "tower_http=debug,sqlx=warn,sqlx_postgres=warn";

pub fn init(level: &str, json: bool) {
    // 环境变量优先，但降噪规则始终追加在后：
    // EnvFilter 中 target 更具体的指令优先级更高，因此 sqlx=warn 不会被 RUST_LOG=debug 覆盖
    let base = std::env::var("RUST_LOG").unwrap_or_else(|_| level.to_owned());
    let filter = EnvFilter::new(format!("{base},{EXTRA_DIRECTIVES}"));

    if json {
        fmt()
            .with_env_filter(filter)
            .json()
            .flatten_event(true)
            .init();
    } else {
        fmt().with_env_filter(filter).pretty().init();
    }

    tracing::info!(json, "logging initialized");
}
