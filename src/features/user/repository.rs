//! 用户域数据访问层：本项目唯一允许编写 SQL 的位置

use sqlx::PgPool;
use uuid::Uuid;

use super::model::UserRow;
use crate::error::{AppError, AppResult};

const COLUMNS: &str =
    "id, email, password_hash, nickname, avatar_url, status, created_at, updated_at";

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> AppResult<UserRow> {
    sqlx::query_as!(
        UserRow,
        r#"SELECT id, email, password_hash, nickname, avatar_url, status, created_at, updated_at
           FROM users WHERE id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

pub async fn find_by_email(pool: &PgPool, email: &str) -> AppResult<Option<UserRow>> {
    sqlx::query_as!(
        UserRow,
        r#"SELECT id, email, password_hash, nickname, avatar_url, status, created_at, updated_at
           FROM users WHERE email = $1"#,
        email
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)
}

pub async fn create(
    pool: &PgPool,
    email: &str,
    password_hash: &str,
    nickname: &str,
) -> AppResult<UserRow> {
    sqlx::query_as!(
        UserRow,
        r#"INSERT INTO users (email, password_hash, nickname)
           VALUES ($1, $2, $3)
           RETURNING id, email, password_hash, nickname, avatar_url, status, created_at, updated_at"#,
        email,
        password_hash,
        nickname
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    nickname: Option<&str>,
    avatar_url: Option<&str>,
) -> AppResult<UserRow> {
    sqlx::query_as!(
        UserRow,
        r#"UPDATE users SET
               nickname   = COALESCE($2, nickname),
               avatar_url = COALESCE($3, avatar_url),
               updated_at = now()
           WHERE id = $1
           RETURNING id, email, password_hash, nickname, avatar_url, status, created_at, updated_at"#,
        id,
        nickname,
        avatar_url
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

/// 统计用户数，供分页与运营看板使用
pub async fn count(pool: &PgPool) -> AppResult<i64> {
    let row = sqlx::query!(r#"SELECT COUNT(*) AS "total!" FROM users"#)
        .fetch_one(pool)
        .await?;
    Ok(row.total)
}

/// 动态条件查询示例：使用 QueryBuilder，禁止字符串拼接 SQL
pub async fn search(pool: &PgPool, keyword: &str, limit: i64) -> AppResult<Vec<UserRow>> {
    let mut qb =
        sqlx::QueryBuilder::new(format!("SELECT {COLUMNS} FROM users WHERE nickname ILIKE "));
    qb.push_bind(format!("%{keyword}%"))
        .push(" ORDER BY created_at DESC LIMIT ")
        .push_bind(limit);

    qb.build_query_as::<UserRow>()
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
}
