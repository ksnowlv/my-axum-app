//! __feature__ 域的数据模型：数据库实体 + 请求/响应 DTO

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// 数据库实体，对应表 `__features__`
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct __Feature__ {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 创建请求
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct Create__Feature__Request {
    #[validate(length(min = 1, max = 64))]
    pub name: String,
}

/// 更新请求：未提供的字段保持不变
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct Update__Feature__Request {
    #[validate(length(min = 1, max = 64))]
    pub name: Option<String>,
}

/// 列表查询参数
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct List__Feature__Query {
    #[validate(range(min = 1))]
    pub page: Option<i64>,
    #[validate(range(min = 1, max = 100))]
    pub page_size: Option<i64>,
}

impl List__Feature__Query {
    pub fn page(&self) -> i64 { self.page.unwrap_or(1) }
    pub fn page_size(&self) -> i64 { self.page_size.unwrap_or(20) }
    pub fn offset(&self) -> i64 { (self.page() - 1) * self.page_size() }
}
