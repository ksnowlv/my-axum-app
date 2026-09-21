//! 全局日志初始化：tracing + tracing-subscriber
//!
//! - 级别：`RUST_LOG` 环境变量优先，未设置时回落到 `DEFAULT_FILTER`
//! - 格式：`LOG_JSON=1|true` 输出 JSON（生产/采集），否则输出可读格式（本地开发）

use tracing_subscriber::{EnvFilter, fmt};

/// `RUST_LOG` 未设置时的默认过滤规则
/// 例：`RUST_LOG=debug,sqlx=warn,my_axum_app=trace`
const DEFAULT_FILTER: &str = "info,tower_http=debug";

pub fn init() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER));

    let json = std::env::var("LOG_JSON")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

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
