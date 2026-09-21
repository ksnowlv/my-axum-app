# 配置与环境变量

## 目录与加载顺序

```text
config/
├── default.toml        # 通用默认值，进仓库
├── development.toml    # 本地默认值，进仓库
└── production.toml     # 生产默认值，进仓库（不含密钥）
.env                    # 本地密钥覆盖，不进仓库（写入 .gitignore）
```

优先级（后者覆盖前者）：`default.toml` → `{APP_ENV}.toml` → `APP_*` 环境变量 → `.env`。

## Settings 定义

```rust
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub log_level: String,
    pub log_json: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

impl ServerConfig {
    pub fn addr(&self) -> String { format!("{}:{}", self.host, self.port) }
}

impl Settings {
    pub fn load() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();                       // 载入 .env（存在即生效）
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".into());

        let s: Self = config::Config::builder()
            .add_source(config::File::with_name("config/default"))
            .add_source(config::File::with_name(&format!("config/{env}")).required(false))
            .add_source(config::Environment::with_prefix("APP").separator("__"))
            .build()?
            .try_deserialize()?;

        Ok(s)
    }
}
```

环境变量映射示例：

| 环境变量 | 配置路径 |
| --- | --- |
| `APP_ENV` | 运行环境（development / production / test） |
| `APP_SERVER__PORT` | `server.port` |
| `APP_DATABASE__URL` | `database.url` |
| `APP_LOG_LEVEL` | `log_level` |

## config/default.toml

```toml
log_level = "info"
log_json = false

[server]
host = "127.0.0.1"
port = 3000

[database]
url = "postgres://postgres:postgres@localhost:5432/my_axum_app"
max_connections = 10
```

## 约定

- 密钥（数据库密码、JWT secret、第三方 API Key）**只能**通过环境变量注入，`default.toml` 中只放本地开发占位值。
- `.env` 必须写入 `.gitignore`，同时提供 `.env.example` 标注所有必需变量。
- 新增配置项时同步更新三处：`Settings` 结构体、`config/default.toml`、`README` 的环境变量表。
- 禁止在业务代码中调用 `std::env::var`，统一从 `Settings` 读取。
- 启动即校验必要配置（如 `database.url` 非空），失败直接退出，不要带病运行。
- `sqlx` 编译期校验读取的是 `DATABASE_URL`，与运行时的 `APP_DATABASE__URL` 分开设置；`main.rs` 中若发现 `DATABASE_URL` 缺失，应给出明确报错。
