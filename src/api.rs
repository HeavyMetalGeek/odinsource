use crate::document::*;
use sqlx::{migrate::MigrateDatabase, sqlite::SqliteQueryResult, Sqlite, SqlitePool};
use std::convert::From;
use crate::tag::*;

const DB_URL: &str = "sqlite://odinsource.db";

pub async fn setup() -> anyhow::Result<SqlitePool> {
    // Ensure the document storage directory exists
    let doc_store_url = std::path::PathBuf::from(std::env!("DOC_STORE_URL"));
    if !doc_store_url.exists() {
        std::fs::create_dir(doc_store_url)?;
    }
    // Ensure the database exists
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        log::info!("Creating database {}", DB_URL);
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => log::info!("Database creation successful."),
            Err(e) => panic!("error: {}", e),
        }
        let db = SqlitePool::connect(DB_URL).await?;
        let _doc_table_result = initialize_doc_table(&db).await?;
        let _tag_table_result = initialize_tag_table(&db).await?;
        Ok(db)
    } else {
        Ok(SqlitePool::connect(DB_URL).await?)
    }
}

async fn initialize_doc_table(pool: &SqlitePool) -> anyhow::Result<SqliteQueryResult> {
    Ok(sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS documents
        (
            id          INTEGER PRIMARY KEY,
            title       TEXT NOT NULL UNIQUE,
            author      TEXT DEFAULT '',
            publication TEXT DEFAULT '',
            volume      INTEGER DEFAULT 0,
            year        INTEGER DEFAULT 0,
            uuid        TEXT NOT NULL,
            tags        TEXT DEFAULT '',
            doi         TEXT DEFAULT ''
        );
        "#,
    )
    .execute(pool)
    .await?)
}

async fn initialize_tag_table(pool: &SqlitePool) -> anyhow::Result<SqliteQueryResult> {
    Ok(sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tags
        (
            id INTEGER PRIMARY KEY,
            value TEXT NOT NULL UNIQUE
        );
        "#,
    )
    .execute(pool)
    .await?)
}

pub async fn get_tags(pool: &SqlitePool) -> anyhow::Result<Vec<DatabaseTag>> {
    Ok(sqlx::query_as::<_, DatabaseTag>(
        r#"
        SELECT *
        FROM tags
        "#,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn print_tags(pool: &SqlitePool) -> anyhow::Result<()> {
    println!("Tags:\n{}", TagList(get_tags(pool).await?));
    Ok(())
}

pub async fn get_docs(pool: &SqlitePool) -> anyhow::Result<Vec<DatabaseDoc>> {
    Ok(sqlx::query_as::<_, DatabaseDoc>(
        r#"
        SELECT *
        FROM documents
        "#,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn print_docs(pool: &SqlitePool) -> anyhow::Result<()> {
    let sep = "=".repeat(80);
    println!("{}", sep);
    println!(
        "Documents:\n{}\n{}{}",
        sep,
        DocList(get_docs(pool).await?),
        sep
    );
    Ok(())
}

pub async fn print_doc_list(doc_list: DocList) -> anyhow::Result<()> {
    let sep = "=".repeat(80);
    println!("{}", sep);
    println!("Documents:\n{}\n{}{}", sep, doc_list, sep);
    Ok(())
}
