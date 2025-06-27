use dotenv::dotenv;
use sqlx::PgPool;
use std::env;

pub async fn establish_connection() -> std::result::Result<PgPool, Box<dyn std::error::Error>> {
    dotenv().ok();

    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL environment variable is not set");
    let pool = PgPool::connect(&database_url).await?;

    println!("Successfully connected to the database...");

    let row: (i32,) = sqlx::query_as("SELECT 1").fetch_one(&pool).await?;

    println!("Query result: {}", row.0);

    Ok(pool)
}

pub async fn save_url_to_db(pool: &PgPool, url: String) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO urls (address) VALUES ($1)")
        .bind(url)
        .execute(pool)
        .await?;

    Ok(())
}
