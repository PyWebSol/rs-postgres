use sqlx::{PgPool, Row};
use sqlx_postgres::PgPoolOptions;

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(url: impl ToString) -> Result<Self, String> {
        let pool = PgPoolOptions::new()
            .connect(&url.to_string())
            .await;

        if let Ok(pool) = pool {
            return Ok(Self {
                pool,
            });
        } else if let Err(e) = pool {
            return Err(e.to_string());
        }

        Err(String::from("Unknown error"))
    }

    pub async fn get_tables(&self) -> Result<Vec<String>, String> {
        let rows = sqlx::query("SELECT table_name FROM information_schema.tables")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
    
        let tables = rows
            .into_iter()
            .map(|row| row.try_get("table_name").map_err(|e| e.to_string()))
            .collect::<Result<Vec<String>, String>>()?;
    
        Ok(tables)
    }

    pub async fn get_table_columns(&self, table_name: &str) -> Result<Vec<String>, String> {
        let rows = sqlx::query("SELECT column_name FROM information_schema.columns WHERE table_name = $1")
            .bind(table_name)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let columns = rows
            .into_iter()
            .map(|row| row.try_get("column_name").map_err(|e| e.to_string()))
            .collect::<Result<Vec<String>, String>>()?;

        Ok(columns)
    }

    pub async fn execute_query(&self, query: &str) -> Result<Vec<String>, String> {
        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let data = rows
            .into_iter()
            .map(|row| row.try_get("column_name").map_err(|e| e.to_string()))
            .collect::<Result<Vec<String>, String>>()?;

        Ok(data)
    }
}
