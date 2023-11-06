pub mod cli;
pub mod document;
pub mod tag;

use cli::*;
use document::*;
use tag::*;

//use serde::Deserialize;
use clap::Parser;
use sqlx::{migrate::MigrateDatabase, sqlite::SqliteQueryResult, Sqlite, SqlitePool};
use std::convert::From;

const DB_URL: &str = "sqlite://odinsource.db";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let db = setup().await?;
    let args = Cli::parse();
    match args.entity_type {
        EntityType::Tag(cmd) => match cmd.command {
            TagSubCmd::Add(cmd) => {
                Tag::new(&cmd.value).insert(&db).await?;
                print_tags(&db).await?;
            }
            TagSubCmd::Modify(cmd) => {
                let tag_values = match cmd.method {
                    ModifyTagSubCmd::ById(input) => {
                        (Tag::from_id(input.id, &db).await?.value, input.new_value)
                    }
                    ModifyTagSubCmd::ByValue(input) => {
                        (input.old_value, input.new_value)
                    }
                };
                DocList::get_all(&db)
                    .await?
                    .modify_tag(&tag_values.0, &tag_values.1, &db)
                    .await?;
            },
            TagSubCmd::Delete(cmd) => {
                if let Some(name) = cmd.value {
                    Tag::new(&name).delete(&db).await?;
                } else if let Some(id) = cmd.id {
                    Tag::from_id(id, &db).await?.delete(&db).await?;
                }
                print_tags(&db).await?;
            }
            TagSubCmd::List => {
                print_tags(&db).await?;
            }
        },
        EntityType::Document(cmd) => match cmd.command {
            DocSubCmd::Add(cmd) => match cmd.source {
                AddDocSubCmd::Single(doc) => {
                    let doc: Document = doc.into();
                    doc.insert(&db).await?;
                    print_docs(&db).await?;
                }
                AddDocSubCmd::FromToml(toml) => {
                    if toml.path.is_file()
                        && toml.path.extension() == Some(&std::ffi::OsStr::new("toml"))
                    {
                        let toml_str = std::fs::read_to_string(toml.path)?;
                        let docs: TomlDocuments = toml::from_str(&toml_str)?;
                        docs.add_to_db(&db).await?;
                        print_docs(&db).await?;
                    } else {
                        return Err(anyhow::anyhow!("Invalid document file: {:?}", toml.path));
                    }
                }
            },
            DocSubCmd::Modify(cmd) => match cmd.method {
                ModifyDocSubCmd::ById(input) => {
                    input.update_doc(&db).await?;
                }
                ModifyDocSubCmd::ByTitle(input) => {
                    input.update_doc(&db).await?;
                }
            },
            DocSubCmd::Delete(cmd) => {
                if let Some(id) = cmd.id {
                    Document::from_id(id, &db).await?.delete(&db).await?;
                } else if let Some(title) = cmd.title {
                    Document::delete_from_title(&title, &db).await?;
                }
                print_docs(&db).await?;
            }
            DocSubCmd::List => {
                print_docs(&db).await?;
            }
            DocSubCmd::Open(cmd) => {
                let doc = match cmd {
                    OpenDoc { id: Some(id), .. } => Document::from_id(id, &db).await?,
                    OpenDoc {
                        title: Some(title), ..
                    } => Document::from_title(&title, &db).await?,
                    _ => Err(anyhow::anyhow!("Must provide ID or TITLE"))?,
                };
                let path = doc.stored_path(&db).await?;
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
        log::info!("Creating database {}", DB_URL);
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => log::info!("Database creation successful."),
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

async fn get_tags(pool: &SqlitePool) -> anyhow::Result<Vec<DatabaseTag>> {
    return Ok(sqlx::query_as::<_, DatabaseTag>(
        r#"
        SELECT *
        FROM tags
        "#,
    )
    .fetch_all(pool)
    .await?);
}

async fn print_tags(pool: &SqlitePool) -> anyhow::Result<()> {
    println!("Tags:\n{}", TagList(get_tags(pool).await?));
    return Ok(());
}

async fn get_docs(pool: &SqlitePool) -> anyhow::Result<Vec<DatabaseDoc>> {
    return Ok(sqlx::query_as::<_, DatabaseDoc>(
        r#"
        SELECT *
        FROM documents
        "#,
    )
    .fetch_all(pool)
    .await?);
}

async fn print_docs(pool: &SqlitePool) -> anyhow::Result<()> {
    let sep = "=".repeat(80);
    println!("{}", sep);
    println!(
        "Documents:\n{}\n{}{}",
        sep,
        DocList(get_docs(pool).await?),
        sep
    );
    return Ok(());
}
