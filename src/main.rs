pub mod cli;
pub mod document;
pub mod tag;

use cli::*;
use document::*;
use tag::*;

use anyhow::Context;
//use serde::Deserialize;
use clap::Parser;
use sqlx::{migrate::MigrateDatabase, sqlite::SqliteQueryResult, Sqlite, SqlitePool};
use std::convert::From;

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
                tags.delete_from_db(&db).await.context("delete tag(s)")?;
                let tags: Vec<Tag> = get_tags(&db).await?;
                println!("Tags:\n{:#?}", tags);
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
                        println!("Docs:\n{:#?}", docs);
                    } else {
                        return Err(anyhow::anyhow!("Invalid document file: {:?}", path));
                    }
                } else if let Some(path) = cmd.toml {
                    if path.is_file() && path.extension() == Some(&std::ffi::OsStr::new("toml")) {
                        let toml_str = std::fs::read_to_string(path)?;
                        let docs: TomlDocuments = toml::from_str(&toml_str)?;
                        docs.add_to_db(&db).await?;
                        let docs: Vec<Document> = get_docs(&db).await?;
                        println!("Docs:\n{:#?}", docs);
                    } else {
                        return Err(anyhow::anyhow!("Invalid document file: {:?}", path));
                    }
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
                    match Document::from_title(&title, &db).await {
                        Ok(doc_opt) => {
                            if let Some(doc) = doc_opt {
                                doc.delete_from_db(&db).await?;
                            }
                        }
                        Err(e) => return Err(e),
                    }
                }
                let docs: Vec<Document> = get_docs(&db).await?;
                println!("Docs:\n{:#?}", docs);
            }
            DocSubCmd::List => {
                let docs: Vec<Document> = get_docs(&db).await?;
                println!("Docs:\n{:#?}", docs);
            }
            DocSubCmd::Open(cmd) => {
                let mut doc = Document::default();
                if let Some(id) = cmd.id {
                    doc = Document::from_id(id, &db).await?;
                } else if let Some(title) = cmd.title {
                    doc = Document::from_title(&title, &db)
                        .await?
                        .ok_or(anyhow::anyhow!("Title not in DB: {}", title))?;
                }
                let path = doc.stored_path().await?;
                std::process::Command::new("xdg-open")
                    .arg(path)
                    .spawn()
                    .expect("Failed to open document");
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
            tags TEXT DEFAULT '',
            doi TEXT DEFAULT ''
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
        SELECT *
        FROM documents
        "#,
    )
    .fetch_all(pool)
    .await?);
}
