pub mod cli;
pub mod document;
pub mod tag;

use cli::*;
use document::*;
use tag::*;

use std::io::Write;
use uuid::Uuid;

use anyhow::Context;
//use serde::Deserialize;
use clap::Parser;
use sqlx::{migrate::MigrateDatabase, sqlite::SqliteQueryResult, Sqlite, SqlitePool};
use std::convert::From;
use std::path::PathBuf;

const DB_URL: &str = "sqlite://odinsource.db";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db = setup().await?;
    let args = Cli::parse();
    match args.entity_type {
        EntityType::Tag(cmd) => match cmd.command {
            TagSubCmd::Add(cmd) => {
                let tags = TagInputList::from(cmd.name.as_str());
                tags.add_to_db(&db).await.context("add tag(s)")?;
                let tags: Vec<Tag> = get_tags(&db).await?;
                println!("Tags:\n{:#?}", tags);
            }
            TagSubCmd::Modify(cmd) => unimplemented!(),
            TagSubCmd::Delete(cmd) => {
                let tags = TagInputList::from(cmd.name.as_str());
                tags.delete_from_db(&db).await.context("delete_tags")?;
                let tags: Vec<Tag> = get_tags(&db).await?;
                println!("Tags:\n{:?}", tags);
            }
            TagSubCmd::List => {
                let tags: Vec<Tag> = get_tags(&db).await?;
                println!("Tags:\n{:#?}", tags);
            }
        },
        EntityType::Document(cmd) => match cmd.command {
            DocSubCmd::Add(cmd) => {
                if let Some(path) = cmd.path {
                    if path.is_file() && path.extension() == Some(&std::ffi::OsStr::new("pdf")) {
                        Document::from_prompts().await?.add_to_db(path, &db).await?;
                        let docs: Vec<Document> = get_docs(&db).await?;
                        println!("Docs:\n{:?}", docs);
                    } else {
                        return Err(anyhow::anyhow!("Invalid document file: {:?}", path));
                    }
                } else if let Some(path) = cmd.toml {
                    unimplemented!()
                    //if path.is_file() && path.extension() == Some(&std::ffi::OsStr::new("pdf")) {
                    //    // TODO: copy file to doc_store_url
                    //    let new_doc = get_doc_info().await?;
                    //    add_doc(&db, &new_doc).await?;
                    //    let docs: Vec<Document> = get_docs(&db).await?;
                    //    println!("Docs:\n{:?}", docs);
                    //} else {
                    //    return Err(anyhow::anyhow!("Invalid document file: {:?}", path));
                    //}
                }
            }
            DocSubCmd::Modify(cmd) => unimplemented!(),
            DocSubCmd::Delete(cmd) => {
                if let Some(id) = cmd.id {
                    Document::from_id(id, &db)
                        .await?
                        .delete_from_db(&db)
                        .await?;
                }
                if let Some(title) = cmd.title {
                    Document::from_title(title, &db)
                        .await?
                        .delete_from_db(&db)
                        .await?;
                }
                let docs: Vec<Document> = get_docs(&db).await?;
                println!("Docs:\n{:?}", docs);
            }
            DocSubCmd::List => {
                let docs: Vec<Document> = get_docs(&db).await?;
                println!("Docs:\n{:#?}", docs);
            }
        },
    }
    return Ok(());
}

async fn setup() -> anyhow::Result<SqlitePool> {
    // Ensure the document storage directory exists
    let doc_store_url = std::path::PathBuf::from(std::env!("DOC_STORE_URL"));
    if !doc_store_url.exists() {
        std::fs::create_dir(doc_store_url)?;
    }
    // Ensure the database exists
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        println!("Creating database {}", DB_URL);
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => println!("Database creation successful."),
            Err(e) => panic!("error: {}", e),
        }
        let db = SqlitePool::connect(DB_URL).await?;
        let _doc_table_result = initialize_doc_table(&db).await?;
        let _tag_table_result = initialize_tag_table(&db).await?;
        return Ok(db);
    } else {
        return Ok(SqlitePool::connect(DB_URL).await?);
    };
}

async fn initialize_doc_table(pool: &SqlitePool) -> anyhow::Result<SqliteQueryResult> {
    return Ok(sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS documents
        (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL UNIQUE,
            author_firstname TEXT NOT NULL,
            author_lastname TEXT NOT NULL,
            year_published INTEGER NOT NULL,
            publication TEXT NOT NULL,
            volume INTEGER DEFAULT 0,
            uuid TEXT NOT NULL,
            tags TEXT DEFAULT ''
        );
        "#,
    )
    .execute(pool)
    .await?);
}

async fn initialize_tag_table(pool: &SqlitePool) -> anyhow::Result<SqliteQueryResult> {
    return Ok(sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tags
        (
            id INTEGER PRIMARY KEY,
            value TEXT NOT NULL UNIQUE
        );
        "#,
    )
    .execute(pool)
    .await?);
}

async fn get_tags(pool: &SqlitePool) -> anyhow::Result<Vec<Tag>> {
    return Ok(sqlx::query_as::<_, Tag>(
        r#"
        SELECT id, value
        FROM tags
        "#,
    )
    .fetch_all(pool)
    .await?);
}

async fn get_docs(pool: &SqlitePool) -> anyhow::Result<Vec<Document>> {
    return Ok(sqlx::query_as::<_, Document>(
        r#"
        SELECT id, title, author_firstname, author_lastname, year_published, publication, volume, uuid, tags
        FROM documents
        "#,
    )
    .fetch_all(pool)
    .await?);
}
