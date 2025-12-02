use async_trait::async_trait;
use sqlx::FromRow;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::auth::models::User;

#[async_trait]
pub trait AuthDb: Send + Sync
{
    async fn create_user(&self, user: User) -> Result<(), DbError>;
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, DbError>;
    // async fn get_user_by_id(&self, id: UserId) -> Result<Option<User>, DbError>;
    // async fn update_user(&self, user: User) -> Result<(), DbError>;

    async fn delete_user(&self, email: &str) -> Result<(), DbError>;
}

#[derive(Debug)]
pub enum DbError
{
    NotFound,
    UserInactive,
    InvalidPassword,
    UserAlreadyExists,
    HashError(String),
    DbError(String),
}

pub struct SqliteUserDb
{
    pool: SqlitePool,
}

impl SqliteUserDb
{
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error>
    {
        let pool = SqlitePool::connect(database_url).await?;
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<(), sqlx::Error>
    {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }
}

#[derive(FromRow)]
struct UserRow
{
    id: String,
    email: String,
    password_hash: String,
    role: String,
    assets: String,
    provider: String,
    is_active: i32,
}

impl TryFrom<UserRow> for User
{
    type Error = DbError;

    fn try_from(row: UserRow) -> Result<Self, Self::Error>
    {
        Ok(User {
            id: Uuid::parse_str(&row.id)
                .map_err(|e| DbError::DbError(format!("Invalid UUID: {}", e)))?,
            email: row.email,
            password_hash: Some(row.password_hash),
            role: row.role.parse().map_err(|e| DbError::DbError(e))?,
            assets: serde_json::from_str(&row.assets)
                .map_err(|e| DbError::DbError(format!("Invalid assets JSON: {}", e)))?,
            provider: row.provider.parse().map_err(|e| DbError::DbError(e))?,
            is_active: row.is_active != 0,
        })
    }
}

#[async_trait]
impl AuthDb for SqliteUserDb
{
    async fn create_user(&self, user: User) -> Result<(), DbError>
    {
        let assets_json =
            serde_json::to_string(&user.assets).map_err(|e| DbError::DbError(e.to_string()))?;
        sqlx::query(
            r#"
                INSERT INTO users (id, email, password_hash, role, assets, provider, is_active)
                VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(user.id.to_string())
        .bind(user.email.to_string())
        .bind(user.password_hash)
        .bind(user.role.to_string())
        .bind(assets_json)
        .bind(user.provider.to_string())
        .bind(user.is_active as i32)
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(ref db_err) if db_err.is_unique_violation() => {
                DbError::UserAlreadyExists
            }
            _ => DbError::DbError(e.to_string()),
        })?;

        Ok(())
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, DbError>
    {
        let row: Option<UserRow> = sqlx::query_as("SELECT id, email, password_hash, role, assets, provider, is_active FROM users WHERE email = ?")
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DbError::DbError(e.to_string()))?;

        row.map(User::try_from).transpose()
    }

    // async fn update_user(&self, user: User) -> Result<(), DbError>
    // {
    //     todo!()
    // }

    async fn delete_user(&self, email: &str) -> Result<(), DbError>
    {
        let result = sqlx::query("DELETE FROM users WHERE email = ?")
            .bind(email)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::DbError(e.to_string()))?;
        if result.rows_affected() == 0 {
            return Err(DbError::NotFound);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::auth::models::UserRole;

    async fn setup_test_db() -> SqliteUserDb
    {
        // In-memory SQLite for tests
        let db = SqliteUserDb::new("sqlite::memory:").await.unwrap();
        db.migrate().await.unwrap();
        db
    }

    fn create_test_user(email: &str, password: &str) -> User
    {
        User::new(
            Uuid::new_v4(),
            email.to_string(),
            Some(password.to_string()),
            UserRole::Technician,
            vec!["asset-1".to_string()],
            crate::auth::provider::Provider::Local,
            true,
        )
        .unwrap()
    }

    #[tokio::test]
    async fn test_create_and_get_user()
    {
        let db = setup_test_db().await;
        let user = create_test_user("test@example.com", "password123");

        db.create_user(user.clone()).await.unwrap();

        let retrieved = db.get_user_by_email("test@example.com").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().email, "test@example.com");
    }

    #[tokio::test]
    async fn test_create_duplicate_user_fails()
    {
        let db = setup_test_db().await;
        let user = create_test_user("test@example.com", "password123");

        db.create_user(user.clone()).await.unwrap();

        let result = db.create_user(user).await;
        assert!(matches!(result, Err(DbError::UserAlreadyExists)));
    }

    #[tokio::test]
    async fn test_delete_user()
    {
        let db = setup_test_db().await;
        let user = create_test_user("test@example.com", "password123");

        db.create_user(user).await.unwrap();
        db.delete_user("test@example.com").await.unwrap();

        let result = db.get_user_by_email("test@example.com").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_user()
    {
        let db = setup_test_db().await;

        let result = db.delete_user("nonexistent@example.com").await;
        assert!(matches!(result, Err(DbError::NotFound)));
    }
}
