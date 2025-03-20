use sqlx::PgPool;
use sqlx_postgres::PgPoolOptions;

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
}
