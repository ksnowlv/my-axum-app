//! __feature__ 域的数据访问层：本项目唯一允许编写 SQL 的位置

use uuid::Uuid;

use sqlx::PgPool;

use crate::error::{AppError, AppResult};
use super::model::{Create__Feature__Request, Update__Feature__Request, __Feature__};

const COLUMNS: &str = "id, name, created_at, updated_at";

/// 按主键查询，不存在返回 `AppError::NotFound`
pub async fn find_by_id(pool: &PgPool, id: Uuid) -> AppResult<__Feature__> {
    sqlx::query_as!(
        __Feature__,
        r#"SELECT id, name, created_at, updated_at FROM __features__ WHERE id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

/// 分页列表
pub async fn list(pool: &PgPool, limit: i64, offset: i64) -> AppResult<Vec<__Feature__>> {
    sqlx::query_as!(
        __Feature__,
        r#"SELECT id, name, created_at, updated_at
           FROM __features__ ORDER BY created_at DESC LIMIT $1 OFFSET $2"#,
        limit,
        offset
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

/// 总数，用于分页
pub async fn count(pool: &PgPool) -> AppResult<i64> {
    let row = sqlx::query!(r#"SELECT COUNT(*) AS "total!" FROM __features__"#)
        .fetch_one(pool)
        .await?;
    Ok(row.total)
}

/// 名称是否已存在，供 service 层做唯一性校验
pub async fn exists_by_name(pool: &PgPool, name: &str) -> AppResult<bool> {
    let row = sqlx::query!(r#"SELECT EXISTS(SELECT 1 FROM __features__ WHERE name = $1) AS "exists!""#, name)
        .fetch_one(pool)
        .await?;
    Ok(row.exists)
}

/// 创建
pub async fn create(pool: &PgPool, req: Create__Feature__Request) -> AppResult<__Feature__> {
    sqlx::query_as!(
        __Feature__,
        r#"INSERT INTO __features__ (name) VALUES ($1)
           RETURNING id, name, created_at, updated_at"#,
        req.name
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}

/// 局部更新
pub async fn update(
    pool: &PgPool,
    id: Uuid,
    req: Update__Feature__Request,
) -> AppResult<__Feature__> {
    sqlx::query_as!(
        __Feature__,
        r#"UPDATE __features__ SET
               name = COALESCE($2, name),
               updated_at = now()
           WHERE id = $1
           RETURNING id, name, created_at, updated_at"#,
        id,
        req.name
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

/// 删除
pub async fn delete(pool: &PgPool, id: Uuid) -> AppResult<()> {
    let res = sqlx::query!(r#"DELETE FROM __features__ WHERE id = $1"#, id)
        .execute(pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

/// 事务示例：接收 `&mut PgConnection` 以便在 service 层复用事务
pub async fn create_in_tx(
    conn: &mut sqlx::PgConnection,
    req: Create__Feature__Request,
) -> AppResult<__Feature__> {
    sqlx::query_as!(
        __Feature__,
        r#"INSERT INTO __features__ (name) VALUES ($1)
           RETURNING id, name, created_at, updated_at"#,
        req.name
    )
    .fetch_one(conn)
    .await
    .map_err(AppError::from)
}

/// 动态条件查询使用 `QueryBuilder`，禁止字符串拼接
pub async fn search(pool: &PgPool, keyword: &str) -> AppResult<Vec<__Feature__>> {
    let mut qb = sqlx::QueryBuilder::new(format!("SELECT {COLUMNS} FROM __features__ WHERE 1 = 1"));
    if !keyword.is_empty() {
        qb.push(" AND name ILIKE ").push_bind(format!("%{keyword}%"));
    }
    qb.push(" ORDER BY created_at DESC");
    qb.build_query_as::<__Feature__>().fetch_all(pool).await.map_err(AppError::from)
}
