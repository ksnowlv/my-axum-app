# 本地开发与热重载

## 热重载

监听源码变更 → 自动重新编译 → 自动重启服务。依赖 `cargo-watch`：

```bash
cargo install cargo-watch
```

两种入口：

```bash
cargo dev             # .cargo/config.toml 中定义的别名
bash scripts/dev.sh   # 自动安装 cargo-watch，并额外监听 config/ migrations/
```

`cargo dev` 展开为 `cargo watch -c -q -w src -w Cargo.toml -x run`：

| 参数 | 作用 |
| --- | --- |
| `-c` | 每次运行前清屏 |
| `-q` | 精简 cargo-watch 自身输出 |
| `-w <path>` | 追加监听路径（默认监听项目内所有文件，显式指定可避免 `target/` 抖动） |
| `-x run` | 变更后执行的命令 |

## 建议用法

- 开发时：`RUST_LOG=debug cargo dev`，配合 `tracing` 观察请求 span 与 SQL。
- 需要格式化与静态检查并行时另开终端运行 `bash .codebuddy/skills/axum-backend/scripts/check.sh`，避免与热重载抢占 `target/` 锁。
- 只跑检查不启动服务：`cargo watch -x "clippy --all-targets"`。
- 修改 `migrations/` 或 `config/` 后热重载会自动重启，无需手动干预（这两个目录已加入 `scripts/dev.sh` 的监听列表）。

## 与数据库配合

接入 sqlx 后，热重载的重启会复用同一连接池配置；注意：

- 迁移不随热重载自动执行，新增迁移后仍需手动 `sqlx migrate run`。
- 编译期宏（`query_as!`）需要 `DATABASE_URL` 或 `.sqlx` 离线缓存，热重载环境下建议常驻 `DATABASE_URL`，或提前 `cargo sqlx prepare` 并使用 `SQLX_OFFLINE=true`。

## 常见问题

- **端口被占用**：`lsof -nP -iTCP:3000 -sTCP:LISTEN` 找到遗留进程后 `kill <pid>`；通常是上一次未正常退出的 `cargo run`。
- **macOS 无 `timeout` 命令**：使用 `perl -e 'alarm shift; exec @ARGV'` 或 `brew install coreutils`（`gtimeout`）。
- **改一次触发多次重启**：编辑器保存会产生多次写入事件，可用 `--delay`（watchexec 参数）或在 `.gitignore` 中排除产物目录。
- **编译很慢**：为 `dev` profile 配置更快的链接器（`.cargo/config.toml` 中配置 `-C link-arg=-fuse-ld=lld`）或拆分 crate 减少重编译范围。
