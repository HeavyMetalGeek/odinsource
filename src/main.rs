use std::io::Write;
use uuid::Uuid;

use anyhow::Context;
//use serde::Deserialize;
use clap::{ArgAction, Args, Parser, Subcommand};
use sqlx::{migrate::MigrateDatabase, sqlite::SqliteQueryResult, FromRow, Sqlite, SqlitePool};
use std::convert::{From, TryFrom};
use std::path::PathBuf;

const DB_URL: &str = "sqlite://odinsource.db";

#[derive(Clone, Debug)]
struct TagInputList(Vec<String>);

impl Default for TagInputList {
    fn default() -> Self {
        return Self(Vec::new());
    }
}

impl std::fmt::Display for TagInputList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{:?}", self.0);
    }
}

impl std::convert::From<&str> for TagInputList {
    fn from(value: &str) -> Self {
        let tags = value
            .split(',')
            .filter(|v| *v != "")
            .map(|v| v.trim().to_lowercase().to_string())
            .collect();
        return Self(tags);
    }
}

impl TagInputList {
    async fn add_to_db(&self, pool: &SqlitePool) -> anyhow::Result<()> {
        for value in self.0.iter() {
            sqlx::query(
                r#"
                INSERT INTO tags (value)
                VALUES (?)
                "#,
            )
            .bind(value)
            .execute(pool)
            .await?;
        }
        return Ok(());
    }

    async fn delete_from_db(&self, pool: &SqlitePool) -> anyhow::Result<()> {
        for value in self.0.iter() {
            println!("Value to be deleted: {:?}", value);
            sqlx::query(
                r#"
            DELETE FROM tags
            WHERE value=?
            "#,
            )
            .bind(value)
            .execute(pool)
            .await?;
        }
        return Ok(());
    }
}

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    entity_type: EntityType,
}

#[derive(Debug, Subcommand)]
enum EntityType {
    Tag(TagCmd),
    #[command(name = "doc")]
    Document(DocCmd),
}

#[derive(Debug, Args)]
struct TagCmd {
    #[command(subcommand)]
    command: TagSubCmd,
}

#[derive(Debug, Subcommand)]
enum TagSubCmd {
    Add(AddTag),
    Modify(ModifyTag),
    Delete(DeleteTag),
    List,
}

#[derive(Debug, Args)]
pub struct AddTag {
    name: String,
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct ModifyTag {
    #[arg(long)]
    id: u16,
    #[arg(long)]
    name: String,
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct DeleteTag {
    #[arg(long)]
    id: u16,
    #[arg(long)]
    name: String,
}

#[derive(Debug, Args)]
struct DocCmd {
    #[command(subcommand)]
    command: DocSubCmd,
}

#[derive(Debug, Subcommand)]
enum DocSubCmd {
    Add(AddDoc),
    Modify(ModifyDoc),
    Delete(DeleteDoc),
    List,
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct AddDoc {
    #[arg(long)]
    toml: Option<PathBuf>,
    #[arg(long)]
    path: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct ModifyDoc {
    title: String,
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct DeleteDoc {
    #[arg(long)]
    id: Option<u16>,
    #[arg(long)]
    title: Option<String>,
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let install_root = std::env::var("CARGO_INSTALL_ROOT");
    println!("INSTALL: {}", install_root?);
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
                        let mut new_doc = get_doc_info().await?;
                        store_file(path, &mut new_doc).await?;
                        add_doc(&db, &new_doc).await?;
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
                    delete_doc_id(&db, id).await?;
                }
                if let Some(title) = cmd.title {
                    delete_doc(&db, title).await?;
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

async fn add_doc(pool: &SqlitePool, doc: &Document) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO documents (
            title,
            author_lastname,
            author_firstname,
            year_published,
            publication,
            volume,
            uuid,
            tags
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
    )
    .bind(&doc.title)
    .bind(&doc.author_last)
    .bind(&doc.author_first)
    .bind(doc.year)
    .bind(&doc.publication)
    .bind(doc.volume)
    .bind(&doc.uuid)
    .bind(&doc.tags)
    .execute(pool)
    .await?;
    return Ok(());
}

async fn add_docs(pool: &SqlitePool, docs: &Vec<Document>) -> anyhow::Result<()> {
    for doc in docs {
        add_doc(pool, doc).await?;
    }
    return Ok(());
}

async fn delete_doc(pool: &SqlitePool, title: String) -> anyhow::Result<()> {
    println!("Document to be deleted: {:?}", title);
    sqlx::query(
        r#"
        DELETE FROM documents
        WHERE title=?
        "#,
    )
    .bind(title)
    .execute(pool)
    .await?;
    return Ok(());
}

async fn delete_doc_id(pool: &SqlitePool, id: u16) -> anyhow::Result<()> {
    println!("Document ID to be deleted: {}", id);
    sqlx::query(
        r#"
        DELETE FROM documents
        WHERE id=?
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;
    return Ok(());
}

async fn get_doc_info() -> anyhow::Result<Document> {
    print!("\nEnter the author (last, first): ");
    std::io::stdout().flush()?;
    let mut author = String::new();
    std::io::stdin().read_line(&mut author)?;
    let author_names: Vec<String> = author
        .split(",")
        .take(2)
        .map(|n| n.trim().to_lowercase())
        .collect::<Vec<String>>();
    let [ref last, ref first] = author_names[..2] else {
        Err(anyhow::anyhow!("Bad author input"))?
    };

    print!("\nEnter the title: ");
    std::io::stdout().flush()?;
    let mut title = String::new();
    std::io::stdin().read_line(&mut title)?;

    print!("\nEnter the name of the publication: ");
    std::io::stdout().flush()?;
    let mut publication = String::new();
    std::io::stdin().read_line(&mut publication)?;

    print!("\nEnter the year of publication (YYYY): ");
    std::io::stdout().flush()?;
    let mut year_str = String::new();
    std::io::stdin().read_line(&mut year_str)?;
    let year: u16 = year_str.trim().parse().unwrap_or(0);

    print!("\nEnter the publication volume (default = 0): ");
    std::io::stdout().flush()?;
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    let volume: u16 = buf.trim().parse().unwrap_or(0);

    print!("\nEnter document tags (e.g. \"rust, programming\"): ");
    std::io::stdout().flush()?;
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    let tags: String = if buf == "" {
        String::new()
    } else {
        TagInputList::from(buf.trim().trim_matches('"')).0.join(",")
    };

    let doc = Document {
        id: 0,
        title: title.trim().to_lowercase(),
        author_last: last.to_owned(),
        author_first: first.to_owned(),
        publication: publication.trim().to_lowercase(),
        year,
        volume,
        tags,
        uuid: String::new(),
    };

    println!("Document Entry: {:?}", doc);
    print!("Does this look correct ((y)es, (n)o)? ");
    std::io::stdout().flush()?;
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    match buf.trim() {
        "y" | "yes" => return Ok(doc),
        _ => return Err(anyhow::anyhow!("Entry cancelled.")),
    }
}

#[derive(FromRow, Debug, Hash)]
struct Document {
    pub id: u32,
    pub title: String,
    #[sqlx(rename = "author_firstname")]
    pub author_first: String,
    #[sqlx(rename = "author_lastname")]
    pub author_last: String,
    #[sqlx(rename = "year_published")]
    pub year: u16,
    pub publication: String,
    pub volume: u16,
    pub tags: String,
    pub uuid: String,
}

#[derive(FromRow, Debug)]
struct Tag {
    pub id: u32,
    pub value: String,
}

async fn store_file(path: PathBuf, doc: &mut Document) -> anyhow::Result<()> {
    let uuid = Uuid::new_v4().to_string();
    doc.uuid = uuid.clone();
    let asset_path = std::path::PathBuf::from(std::env!("DOC_STORE_URL")).join(uuid + ".pdf");
    std::fs::copy(path.clone(), asset_path.clone())?;
    println!("Document {:?} stored as {:?}", path, asset_path);
    return Ok(());
}

async fn delete_file(doc: &Document) -> anyhow::Result<()> {
    let fname = doc.uuid.clone() + ".pdf";
    let asset_path = std::path::PathBuf::from(std::env!("DOC_STORE_URL")).join(fname);
    std::fs::remove_file(asset_path.clone())?;
    println!("Document {:?} deleted.", asset_path);
    return Ok(());
}
