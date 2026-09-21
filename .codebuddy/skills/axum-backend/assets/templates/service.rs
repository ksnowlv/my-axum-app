//! __feature__ 域的业务层：业务规则与事务边界

use uuid::Uuid;

use sqlx::PgPool;
use tracing::instrument;

use crate::error::{AppError, AppResult};
use crate::response::Page;
use super::model::{Create__Feature__Request, List__Feature__Query, Update__Feature__Request, __Feature__};
use super::repository;

/// 分页查询
#[instrument(skip(pool))]
pub async fn list__Feature__s(
    pool: &PgPool,
    query: List__Feature__Query,
) -> AppResult<Page<__Feature__>> {
    let total = repository::count(pool).await?;
    let items = repository::list(pool, query.page_size(), query.offset()).await?;
    Ok(Page::new(items, total, query.page(), query.page_size()))
}

/// 查询详情
#[instrument(skip(pool))]
pub async fn get__Feature__(pool: &PgPool, id: Uuid) -> AppResult<__Feature__> {
    repository::find_by_id(pool, id).await
}

/// 创建：业务规则与唯一性校验在此完成
#[instrument(skip(pool))]
pub async fn create__Feature__(pool: &PgPool, req: Create__Feature__Request) -> AppResult<__Feature__> {
    // 示例规则：名称不允许重复
    if repository::exists_by_name(pool, &req.name).await? {
        return Err(AppError::Conflict(format!("name '{}' already exists", req.name)));
    }
    repository::create(pool, req).await
}

/// 更新
#[instrument(skip(pool))]
pub async fn update__Feature__(
    pool: &PgPool,
    id: Uuid,
    req: Update__Feature__Request,
) -> AppResult<__Feature__> {
    repository::update(pool, id, req).await
}

/// 删除
#[instrument(skip(pool))]
pub async fn delete__Feature__(pool: &PgPool, id: Uuid) -> AppResult<()> {
    repository::delete(pool, id).await
}

/// 事务示例：多步写入在 service 层开启事务
#[instrument(skip(pool))]
pub async fn batch_create(pool: &PgPool, names: Vec<String>) -> AppResult<Vec<__Feature__>> {
    let mut tx = pool.begin().await?;
    let mut created = Vec::with_capacity(names.len());

    for name in names {
        let item = repository::create_in_tx(
            &mut *tx,
            Create__Feature__Request { name },
        )
        .await?;
        created.push(item);
    }

    tx.commit().await?;
    Ok(created)
}
