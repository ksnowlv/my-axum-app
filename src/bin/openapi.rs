//! 导出 OpenAPI 文档到标准输出
//!
//! 用法: cargo run --bin openapi > openapi.json
//!       bash scripts/openapi.sh          # 同上，并输出生成结果提示

use utoipa::OpenApi;

use my_axum_app::docs::ApiDoc;

fn main() {
    match ApiDoc::openapi().to_pretty_json() {
        Ok(json) => println!("{json}"),
        Err(e) => {
            eprintln!("生成 OpenAPI 文档失败: {e}");
            std::process::exit(1);
        }
    }
}
