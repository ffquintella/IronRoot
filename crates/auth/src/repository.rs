//! User repository — abstract trait plus a SQL implementation backed by
//! [`ironroot_dal::Pool`].

use async_trait::async_trait;
use ironroot_dal::{Backend, Pool};

use crate::{AuthError, User};

/// Storage abstraction for [`User`] records.
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Insert a new user and return the persisted record (with assigned id).
    async fn create(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User, AuthError>;

    /// Look up a user by username, returning `None` if absent.
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, AuthError>;

    /// Look up a user by id.
    async fn find_by_id(&self, id: i64) -> Result<Option<User>, AuthError>;

    /// Delete a user by id. No-op if the user does not exist.
    async fn delete(&self, id: i64) -> Result<(), AuthError>;
}

/// SQL implementation built on [`ironroot_dal::Pool`].
///
/// Call [`SqlUserRepository::ensure_schema`] once at startup to create the
/// `users` table if it does not already exist. The schema is a pragmatic
/// default — feel free to manage it yourself and skip the call.
pub struct SqlUserRepository {
    pool: Pool,
}

impl SqlUserRepository {
    /// Wrap an existing [`Pool`].
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Borrow the underlying pool.
    pub fn pool(&self) -> &Pool {
        &self.pool
    }

    /// Create the `users` table if it does not already exist.
    pub async fn ensure_schema(&self) -> Result<(), AuthError> {
        let ddl = schema_for(self.pool.backend());
        self.pool.execute(ddl).await?;
        Ok(())
    }
}

fn schema_for(backend: Backend) -> &'static str {
    match backend {
        Backend::Sqlite => {
            "CREATE TABLE IF NOT EXISTS users (\
               id INTEGER PRIMARY KEY AUTOINCREMENT,\
               username TEXT NOT NULL UNIQUE,\
               email TEXT NOT NULL,\
               password_hash TEXT NOT NULL,\
               created_at INTEGER NOT NULL\
             )"
        }
        Backend::MySql => {
            "CREATE TABLE IF NOT EXISTS users (\
               id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,\
               username VARCHAR(64) NOT NULL UNIQUE,\
               email VARCHAR(320) NOT NULL,\
               password_hash TEXT NOT NULL,\
               created_at BIGINT NOT NULL\
             )"
        }
        Backend::Postgres => {
            "CREATE TABLE IF NOT EXISTS users (\
               id BIGSERIAL PRIMARY KEY,\
               username VARCHAR(64) NOT NULL UNIQUE,\
               email VARCHAR(320) NOT NULL,\
               password_hash TEXT NOT NULL,\
               created_at BIGINT NOT NULL\
             )"
        }
    }
}

fn now_epoch() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn escape_sql(input: &str) -> String {
    input.replace('\'', "''")
}

#[async_trait]
impl UserRepository for SqlUserRepository {
    async fn create(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User, AuthError> {
        let created_at = now_epoch();
        let sql = format!(
            "INSERT INTO users (username, email, password_hash, created_at) \
             VALUES ('{}', '{}', '{}', {})",
            escape_sql(username),
            escape_sql(email),
            escape_sql(password_hash),
            created_at,
        );
        let r = self.pool.execute(&sql).await.map_err(map_dup)?;
        let id = r.last_insert_id.unwrap_or(0);
        Ok(User {
            id,
            username: username.to_string(),
            email: email.to_string(),
            password_hash: password_hash.to_string(),
            created_at,
        })
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, AuthError> {
        let sql = format!(
            "SELECT id, username, email, password_hash, created_at \
             FROM users WHERE username = '{}'",
            escape_sql(username)
        );
        let row = self.pool.fetch_optional(&sql).await?;
        Ok(match row {
            Some(r) => Some(User {
                id: r.try_get_i64("id")?,
                username: r.try_get_string("username")?,
                email: r.try_get_string("email")?,
                password_hash: r.try_get_string("password_hash")?,
                created_at: r.try_get_i64("created_at")?,
            }),
            None => None,
        })
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<User>, AuthError> {
        let sql = format!(
            "SELECT id, username, email, password_hash, created_at \
             FROM users WHERE id = {id}"
        );
        let row = self.pool.fetch_optional(&sql).await?;
        Ok(match row {
            Some(r) => Some(User {
                id: r.try_get_i64("id")?,
                username: r.try_get_string("username")?,
                email: r.try_get_string("email")?,
                password_hash: r.try_get_string("password_hash")?,
                created_at: r.try_get_i64("created_at")?,
            }),
            None => None,
        })
    }

    async fn delete(&self, id: i64) -> Result<(), AuthError> {
        let sql = format!("DELETE FROM users WHERE id = {id}");
        self.pool.execute(&sql).await?;
        Ok(())
    }
}

fn map_dup(err: ironroot_dal::DalError) -> AuthError {
    let msg = err.to_string().to_lowercase();
    if msg.contains("unique") || msg.contains("duplicate") {
        AuthError::AlreadyExists
    } else {
        AuthError::Dal(err)
    }
}

#[cfg(all(test, feature = "dal"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn sqlite_create_and_find() {
        let pool = Pool::connect("sqlite::memory:").await.unwrap();
        let repo = SqlUserRepository::new(pool);
        repo.ensure_schema().await.unwrap();

        let u = repo.create("ada", "ada@example.org", "PHC").await.unwrap();
        assert_eq!(u.username, "ada");
        assert!(u.id > 0);

        let found = repo.find_by_username("ada").await.unwrap().unwrap();
        assert_eq!(found.id, u.id);

        let by_id = repo.find_by_id(u.id).await.unwrap().unwrap();
        assert_eq!(by_id.username, "ada");

        let dup = repo.create("ada", "x@y.z", "PHC").await;
        assert!(matches!(dup, Err(AuthError::AlreadyExists)));

        repo.delete(u.id).await.unwrap();
        assert!(repo.find_by_id(u.id).await.unwrap().is_none());
    }
}
