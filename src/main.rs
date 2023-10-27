use std::io::Write;

use anyhow::Context;
//use serde::Deserialize;
use clap::{ArgAction, Parser};
use sqlx::{migrate::MigrateDatabase, sqlite::SqliteQueryResult, FromRow, Sqlite, SqlitePool};
use std::convert::{From, TryFrom};

const DB_URL: &str = "sqlite://odinsource.db";

#[derive(Clone, Debug)]
enum Mode {
    AddTag,
    AddDoc,
    GetDoc,
    GetTags,
}

impl std::convert::From<&str> for Mode {
    fn from(value: &str) -> Self {
        match value {
            "addtag" => Self::AddTag,
            "adddoc" => Self::AddDoc,
            "gettags" => Self::GetTags,
            _ => Self::GetDoc,
        }
    }
}

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

#[derive(Clone, Debug)]
enum Action {
    Add,
    Delete,
    Modify,
    List,
}

impl From<&str> for Action {
    fn from(value: &str) -> Self {
        return match value {
            "add" => Action::Add,
            "delete" => Action::Delete,
            "modify" => Action::Modify,
            "list" => Action::List,
            _ => Action::List,
        };
    }
}

#[derive(Clone, Debug)]
enum ActionTarget {
    Tag,
    Document,
    Error,
}

impl From<&str> for ActionTarget {
    fn from(value: &str) -> Self {
        return match value {
            "t" | "tag" => ActionTarget::Tag,
            "d" | "doc" | "document" => ActionTarget::Document,
            _ => ActionTarget::Error,
        };
    }
}

#[derive(Parser, Debug)]
struct Arguments {
    //action: Action,
    //tag: ActionTarget,
    #[clap(long = "list-tags", action = ArgAction::SetTrue)]
    list_tags: bool,
    #[clap(long = "add-tags")]
    add_tags: Option<TagInputList>,
    #[clap(long = "delete-tags")]
    delete_tags: Option<TagInputList>,
    #[clap(long = "list-docs", action = ArgAction::SetTrue)]
    list_docs: bool,
    #[clap(long = "add-doc")]
    add_doc: Option<String>,
    #[clap(long = "delete-doc")]
    delete_doc: Option<String>,
    #[clap(long = "delete-doc-id")]
    delete_doc_id: Option<u16>,
}

impl Default for Arguments {
    fn default() -> Self {
        return Self {
            //action: Action::List,
            list_tags: false,
            add_tags: None,
            delete_tags: None,
            list_docs: false,
            add_doc: None,
            delete_doc: None,
            delete_doc_id: None,
        };
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
    } else {
        println!("Database already exists");
    }

    let db = SqlitePool::connect(DB_URL).await.unwrap();

    let doc_table_result = initialize_doc_table(&db).await?;
    println!("Create documents table result: {:?}", doc_table_result);

    let tag_table_result = initialize_tag_table(&db).await?;
    println!("Create tags table result: {:?}", tag_table_result);

    let args = Arguments::parse();
    dbg!(&args);
    if let Some(tags) = &args.add_tags {
        tags.add_to_db(&db).await.context("add_tags")?;
        let tags: Vec<Tag> = get_tags(&db).await?;
        println!("Tags:\n{:?}", tags);
    };

    if let Some(tags) = &args.delete_tags {
        tags.delete_from_db(&db).await.context("delete_tags")?;
        //delete_tags(&db, &args.delete_tags.0)
        //    .await
        //    .context("delete_tags")?;
        let tags: Vec<Tag> = get_tags(&db).await?;
        println!("Tags:\n{:?}", tags);
    };

    if let Some(path_str) = &args.add_doc {
        let path = std::path::PathBuf::from(path_str);
        if path.is_file() && path.extension() == Some(&std::ffi::OsStr::new("pdf")) {
            // TODO: copy file to doc_store_url
            let new_doc = get_doc_info().await?;
            add_doc(&db, &new_doc).await?;
            let docs: Vec<Document> = get_docs(&db).await?;
            println!("Docs:\n{:?}", docs);
        } else {
            return Err(anyhow::anyhow!("Invalid document file: {:?}", path));
        }
    };

    if let Some(title) = &args.delete_doc {
        delete_doc(&db, title.to_lowercase()).await?;
        let docs: Vec<Document> = get_docs(&db).await?;
        println!("Docs:\n{:?}", docs);
    };

    if let Some(id) = args.delete_doc_id {
        delete_doc_id(&db, id).await?;
        let docs: Vec<Document> = get_docs(&db).await?;
        println!("Docs:\n{:?}", docs);
    }

    if args.list_tags {
        let tags: Vec<Tag> = get_tags(&db).await?;
        println!("Tags:\n{:?}", tags);
    }

    if args.list_docs {
        let docs: Vec<Document> = get_docs(&db).await?;
        println!("Docs:\n{:?}", docs);
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
            filename TEXT DEFAULT 'EMPTY',
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
        SELECT id, title, author_firstname, author_lastname, year_published, publication, volume, filename, tags
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
            tags
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
    )
    .bind(&doc.title)
    .bind(&doc.author_last)
    .bind(&doc.author_first)
    .bind(doc.year)
    .bind(&doc.publication)
    .bind(doc.volume)
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
        TagInputList::from(buf.trim_matches('"')).0.join(",")
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
    };

    println!("Document Entry: {:?}", doc);
    print!("Does this look correct ((y)es, (n)o)? ");
    std::io::stdout().flush()?;
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    match buf.as_str() {
        "y" | "yes" => return Ok(doc),
        _ => return Err(anyhow::anyhow!("Entry cancelled.")),
    }
}

#[derive(FromRow, Debug)]
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
}

#[derive(FromRow, Debug)]
struct Tag {
    pub id: u32,
    pub value: String,
}
