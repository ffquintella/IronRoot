//! Repository trait — uniform CRUD interface over [`crate::Pool`].

use async_trait::async_trait;

use crate::DalError;
#[cfg(doc)]
use crate::Pool;

/// Generic repository abstraction.
///
/// Implement this on your domain types to get a uniform CRUD interface that
/// works against any backend supported by the active [`Pool`].
///
/// The trait deliberately keeps the surface small — concrete implementations
/// drive their own SQL. For more advanced patterns (transactions, joins,
/// batch operations) drop down to [`Pool`] directly.
///
/// # Example
///
/// ```no_run
/// use async_trait::async_trait;
/// use ironroot_dal::{DalError, Pool, Repository, Row};
///
/// pub struct User {
///     pub id: i64,
///     pub name: String,
/// }
///
/// pub struct UserRepo<'a> {
///     pub pool: &'a Pool,
/// }
///
/// #[async_trait]
/// impl<'a> Repository for UserRepo<'a> {
///     type Entity = User;
///     type Id = i64;
///
///     async fn find_by_id(&self, id: i64) -> Result<Option<User>, DalError> {
///         let sql = format!("SELECT id, name FROM users WHERE id = {id}");
///         let row = self.pool.fetch_optional(&sql).await?;
///         row.map(|r| Ok(User {
///             id: r.try_get_i64("id")?,
///             name: r.try_get_string("name")?,
///         })).transpose()
///     }
///
///     async fn list(&self) -> Result<Vec<User>, DalError> {
///         let rows = self.pool.fetch_all("SELECT id, name FROM users").await?;
///         rows.into_iter()
///             .map(|r| Ok(User {
///                 id: r.try_get_i64("id")?,
///                 name: r.try_get_string("name")?,
///             }))
///             .collect()
///     }
///
///     async fn delete(&self, id: i64) -> Result<(), DalError> {
///         let sql = format!("DELETE FROM users WHERE id = {id}");
///         self.pool.execute(&sql).await?;
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait Repository: Send + Sync {
    /// The domain entity this repository deals in.
    type Entity: Send + Sync;
    /// The primary-key type used to look entities up.
    type Id: Send + Sync;

    /// Find a single entity by id, returning `None` if absent.
    async fn find_by_id(&self, id: Self::Id) -> Result<Option<Self::Entity>, DalError>;

    /// List all entities. Implementations should paginate for large tables.
    async fn list(&self) -> Result<Vec<Self::Entity>, DalError>;

    /// Delete by id.
    async fn delete(&self, id: Self::Id) -> Result<(), DalError>;
}

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use super::*;
    use crate::{Pool, Row};

    #[allow(dead_code)]
    struct Item {
        id: i64,
        name: String,
    }

    struct ItemRepo<'a> {
        pool: &'a Pool,
    }

    #[async_trait]
    impl<'a> Repository for ItemRepo<'a> {
        type Entity = Item;
        type Id = i64;

        async fn find_by_id(&self, id: i64) -> Result<Option<Item>, DalError> {
            let sql = format!("SELECT id, name FROM items WHERE id = {id}");
            match self.pool.fetch_optional(&sql).await? {
                Some(r) => Ok(Some(decode_item(&r)?)),
                None => Ok(None),
            }
        }

        async fn list(&self) -> Result<Vec<Item>, DalError> {
            let rows = self
                .pool
                .fetch_all("SELECT id, name FROM items ORDER BY id")
                .await?;
            rows.iter().map(decode_item).collect()
        }

        async fn delete(&self, id: i64) -> Result<(), DalError> {
            let sql = format!("DELETE FROM items WHERE id = {id}");
            self.pool.execute(&sql).await?;
            Ok(())
        }
    }

    fn decode_item(r: &Row) -> Result<Item, DalError> {
        Ok(Item {
            id: r.try_get_i64("id")?,
            name: r.try_get_string("name")?,
        })
    }

    #[tokio::test]
    async fn repository_crud_on_sqlite() {
        let pool = Pool::connect("sqlite::memory:").await.unwrap();
        pool.execute("CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT NOT NULL)")
            .await
            .unwrap();
        pool.execute("INSERT INTO items (id, name) VALUES (1, 'alpha'), (2, 'beta')")
            .await
            .unwrap();

        let repo = ItemRepo { pool: &pool };

        let one = repo.find_by_id(1).await.unwrap().unwrap();
        assert_eq!(one.name, "alpha");

        let missing = repo.find_by_id(99).await.unwrap();
        assert!(missing.is_none());

        let all = repo.list().await.unwrap();
        assert_eq!(all.len(), 2);

        repo.delete(1).await.unwrap();
        assert!(repo.find_by_id(1).await.unwrap().is_none());
    }
}
