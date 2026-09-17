use leona_core::Error;
use leona_core::Error::DbQueryErr;
use sqlx::{Connection, Row, SqliteConnection, sqlite::SqliteConnectOptions};
use std::str::FromStr;
use tokio::sync::RwLock;
use tokio::sync::RwLockWriteGuard;

#[derive(Debug)]
pub struct Db {
    pub connection: RwLock<SqliteConnection>,
    pub options: SqliteConnectOptions,
}

impl Db {
    pub async fn new(path: &str) -> Result<Self, Error> {
        let conn_opts = match SqliteConnectOptions::from_str(path) {
            Ok(opt) => opt.create_if_missing(true),
            Err(error) => return Err(Error::DbConnErr(error.to_string())),
        };
        Ok(Self {
            connection: Self::connect(&conn_opts).await?,
            options: conn_opts,
        })
    }

    pub async fn connect(
        options: &SqliteConnectOptions,
    ) -> Result<RwLock<SqliteConnection>, Error> {
        match SqliteConnection::connect_with(&options).await {
            Ok(conn) => Ok(RwLock::new(conn)),
            Err(error) => Err(Error::DbConnErr(error.to_string())),
        }
    }

    pub async fn get_tables(&self) -> Result<Vec<String>, Error> {
        let result = {
            let mut connection = self.connection.write().await;
            match sqlx::query(
                "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name;",
            )
            .fetch_all(&mut *connection)
            .await
            {
                Ok(result) => {
                    let names = result
                        .iter()
                        .map(|row| {
                            let name: String = row.get("name");
                            name
                        })
                        .collect::<Vec<String>>();
                    Ok(names)
                }
                Err(error) => Err(DbQueryErr(error.to_string())),
            }
        };
        result
    }

    pub async fn get_connection(&self) -> Result<RwLockWriteGuard<'_, SqliteConnection>, Error> {
        let mut connection = self.connection.write().await;
        if let Err(error) = connection.ping().await {
            return Err(Error::DbConnErr(error.to_string()));
        };
        Ok(connection)
    }
}

#[cfg(test)]
pub mod db_test {

    use super::*;
    use sqlx::prelude::FromRow;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_query() {
        #[derive(Debug, Clone, FromRow)]
        pub struct TestTable {
            pub id: u64,
            pub name: String,
        }
        let db = Db::new("/Users/jp-mac/Codes/leona-api/storage/test.db").await;
        assert!(db.is_ok());
        let db_client = Arc::new(db.expect("expecting a db instance"));

        {
            let mut connection = db_client
                .get_connection()
                .await
                .expect("expecting a connection instance");
            let query_stmt = "CREATE TABLE IF NOT EXISTS test_table(id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL)";
            let result = sqlx::query(query_stmt).execute(&mut *connection).await;
            assert!(result.is_ok());
        }

        let tables = db_client.get_tables().await.expect("expecting tables");
        assert!(tables.contains(&"test_table".to_string()));
        {
            let mut connection = db_client
                .get_connection()
                .await
                .expect("expecting a connection instance");
            let query_stmt = "INSERT INTO test_table(name) VALUES(?)";
            let result = sqlx::query(query_stmt)
                .bind("Juan dela Cruz".to_string())
                .execute(&mut *connection)
                .await;
            assert!(result.is_ok());
            let query_stmt = "SELECT * FROM test_table";
            let result = sqlx::query_as::<_, TestTable>(query_stmt)
                .fetch_all(&mut *connection)
                .await;
            assert!(result.is_ok());
            for item in result.unwrap() {
                assert!(item.id > 0);
                assert!(!item.name.is_empty());
            }
        }
    }
}
