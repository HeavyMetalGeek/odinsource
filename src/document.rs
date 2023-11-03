use crate::tag::TagInputList;
use clap::Args;
use sqlx::{FromRow, SqlitePool};
use std::path::PathBuf;
use uuid::Uuid;
use serde::{Deserializer, Deserialize};

#[derive(FromRow, Debug, Hash)]
pub struct DatabaseDoc {
    pub id: u32,
    pub title: String,
    pub author: String,
    pub year: u16,
    pub publication: String,
    pub volume: u16,
    pub tags: String,
    pub doi: String,
    pub uuid: String,
}

impl std::convert::Into<Document> for DatabaseDoc {
    fn into(self) -> Document {
        let path = self
            .stored_path()
            .expect("CRITICAL: Failed to get stored path for database document");
        let DatabaseDoc {
            id,
            title,
            author,
            year,
            publication,
            volume,
            tags,
            doi,
            ..
        } = self;
        return Document {
            id: Some(id),
            title,
            author,
            year,
            publication,
            volume,
            tags,
            doi,
            path,
        };
    }
}

impl DatabaseDoc {
    pub fn is_stored(&self) -> bool {
        let fname = format!("{}.{}", self.uuid, "pdf");
        let path = std::path::PathBuf::from(std::env!("DOC_STORE_URL")).join(fname);
        return self.uuid.len() == 40 && path.exists();
    }

    pub fn stored_path(&self) -> anyhow::Result<PathBuf> {
        if !self.is_stored() {
            return Err(anyhow::anyhow!("Document is not stored: {:?}", self.title))?;
        }
        let fname = format!("{}.{}", self.uuid, "pdf");
        let path = std::path::PathBuf::from(std::env!("DOC_STORE_URL")).join(fname);
        return Ok(path);
    }

    pub async fn from_id(id: u32, pool: &SqlitePool) -> anyhow::Result<Option<Self>> {
        return Ok(sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM documents
            WHERE id=?1
            "#,
        )
        .bind(&id)
        .fetch_optional(pool)
        .await?);
    }

    pub async fn from_title(title: &str, pool: &SqlitePool) -> anyhow::Result<Option<Self>> {
        return Ok(sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM documents
            WHERE title=?1
            "#,
        )
        .bind(&title)
        .fetch_optional(pool)
        .await?);
    }

    // Check for exIf so, aisting tags
    pub async fn from_insert(doc: Document, pool: &SqlitePool) -> anyhow::Result<Self> {
        let Document {
            title,
            author,
            publication,
            volume,
            year,
            doi,
            tags,
            path,
            ..
        } = doc;

        // Check for existing document with the same title
        // If not, create UUID
        let uuid = match Self::from_title(&title, pool).await? {
            Some(dbd) => {
                println!("Document already in DB: {:?}", title);
                return Ok(dbd);
            }
            None => Uuid::new_v4().to_string(),
        };

        // Add entry to database
        sqlx::query(
            r#"
            INSERT INTO documents (
                title,
                author,
                publication,
                volume,
                year,
                uuid,
                doi,
                tags
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
        )
        .bind(&title)
        .bind(author)
        .bind(publication)
        .bind(volume)
        .bind(year)
        .bind(&uuid)
        .bind(doi)
        .bind(tags.to_lowercase())
        .execute(pool)
        .await?;

        // Store copy of pdf in documents folder
        let stored_path =
            std::path::PathBuf::from(std::env!("DOC_STORE_URL")).join(uuid.clone() + ".pdf");
        std::fs::copy(path.clone(), stored_path.clone())?;
        println!("Document {:?} stored as {:?}", path, stored_path);

        // Ensure entry properly inserted and document is correctly stored
        return match Self::from_title(&title, pool).await? {
            Some(dbd) => {
                // Add tags to tags table and return inserted DatabaseDoc
                for tag in TagInputList::from(tags.as_str()).as_tags() {
                    tag.insert(&pool).await?;
                }
                return Ok(dbd);
            }
            None => Err(anyhow::anyhow!(
                "Failed to create DatabaseDoc after insert."
            ))?,
        };
    }

    pub async fn delete(self, pool: &SqlitePool) -> anyhow::Result<()> {
        let DatabaseDoc { title, uuid, .. } = self;

        // Delete entry from database
        sqlx::query(
            r#"
            DELETE FROM documents
            WHERE title=?1
            "#,
        )
        .bind(&title)
        .execute(pool)
        .await?;

        // Remove stored file
        let fname = uuid + ".pdf";
        let asset_path = std::path::PathBuf::from(std::env!("DOC_STORE_URL")).join(fname);
        if std::fs::remove_file(asset_path.clone()).is_err() {
            println!("Could not delete {:?}.", asset_path);
        } else {
            println!("Document {:?} deleted.", asset_path);
        };

        return Ok(());
    }
}

impl std::fmt::Display for DatabaseDoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", "-".repeat(80))?;
        writeln!(f, "{:12} {}", "id:", self.id)?;
        writeln!(f, "{:12} {}", "title:", self.title)?;
        writeln!(f, "{:12} {}", "author:", self.author)?;
        writeln!(f, "{:12} {}", "publication:", self.publication)?;
        writeln!(f, "{:12} {}", "volume:", self.volume)?;
        writeln!(f, "{:12} {}", "year:", self.year)?;
        writeln!(f, "{:12} {}", "doi:", self.doi)?;
        writeln!(f, "{:12} {}", "tags:", self.tags)?;
        writeln!(f, "{:12} {}", "uuid:", self.uuid)?;
        writeln!(f, "{}", "-".repeat(80))
    }
}

/// Used for user interface (CLI, toml, etc.)
#[derive(Debug, Hash, Deserialize, Args)]
pub struct Document {
    #[arg(skip)]
    pub id: Option<u32>,
    #[arg(long, value_parser = Document::input_to_lowercase, required = true)]
    #[serde(default = "String::new", deserialize_with = "Document::value_to_lowercase")]
    pub title: String,
    #[arg(long, value_parser = Document::input_to_lowercase, default_value = "")]
    #[serde(default = "String::new", deserialize_with = "Document::value_to_lowercase")]
    pub author: String,
    #[arg(long, default_value = "0")]
    #[serde(default = "Document::default_u16")]
    pub year: u16,
    #[arg(long, value_parser = Document::input_to_lowercase, default_value = "")]
    #[serde(default = "String::new", deserialize_with = "Document::value_to_lowercase")]
    pub publication: String,
    #[arg(long, default_value = "0")]
    #[serde(default = "Document::default_u16")]
    pub volume: u16,
    #[arg(long, value_parser = Document::input_to_lowercase, default_value = "")]
    #[serde(default = "String::new", deserialize_with = "Document::value_to_lowercase")]
    pub tags: String,
    #[arg(long, default_value = "")]
    #[serde(default = "String::new")]
    pub doi: String,
    #[arg(long, value_parser = Document::verify_path, required = true)]
    pub path: PathBuf,
}

impl std::fmt::Display for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", "-".repeat(80))?;
        let id = self.id.map_or("None".to_string(), |v| v.to_string());
        writeln!(f, "{:12} {}", "id:", id)?;
        writeln!(f, "{:12} {}", "title:", self.title)?;
        writeln!(f, "{:12} {}", "author:", self.author)?;
        writeln!(f, "{:12} {}", "publication:", self.publication)?;
        writeln!(f, "{:12} {}", "volume:", self.volume)?;
        writeln!(f, "{:12} {}", "year:", self.year)?;
        writeln!(f, "{:12} {}", "doi:", self.doi)?;
        writeln!(f, "{:12} {}", "tags:", self.tags)?;
        writeln!(f, "{:12} {:?}", "path:", self.path)?;
        writeln!(f, "{}", "-".repeat(80))
    }
}

impl Document {
    pub fn value_to_lowercase<'de, D>(deserializer: D) -> Result<String, D::Error>
        where
            D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?.to_lowercase();
        return Ok(value);
    }

    pub fn input_to_lowercase(value: &str) -> anyhow::Result<String> {
        return Ok(value.to_lowercase());
    }

    pub fn verify_path(path: &str) -> anyhow::Result<PathBuf> {
        let path = PathBuf::from(path);
        if !path.is_file() || path.extension() != Some(&std::ffi::OsStr::new("pdf")) {
            return Err(anyhow::anyhow!(
                "Path does not reference a valid PDF: {:?}",
                path
            ))?;
        }
        return Ok(path);
    }

    pub fn default_u16() -> u16 {
        return 0;
    }
    // Must have, at minimum, a title and valid file path
    pub fn new(title: &str, path: &str) -> anyhow::Result<Self> {
        let path = PathBuf::from(path);
        if !path.is_file() || path.extension() != Some(&std::ffi::OsStr::new("pdf")) {
            return Err(anyhow::anyhow!(
                "Path does not reference a valid PDF: {:?}",
                path
            ))?;
        }
        return Ok(Self {
            id: None,
            title: title.to_lowercase(),
            author: String::new(),
            year: 0,
            publication: String::new(),
            volume: 0,
            tags: String::new(),
            doi: String::new(),
            path,
        });
    }

    pub async fn delete_from_title(title: &str, pool: &SqlitePool) -> anyhow::Result<()> {
        return Document {
            id: None,
            title: title.to_lowercase(),
            author: String::new(),
            year: 0,
            publication: String::new(),
            volume: 0,
            tags: String::new(),
            doi: String::new(),
            path: PathBuf::new(),
        }
        .delete(pool)
        .await;
    }

    pub async fn from_id(id: u32, pool: &SqlitePool) -> anyhow::Result<Self> {
        return match DatabaseDoc::from_id(id, pool).await? {
            Some(dbt) => Ok(dbt.into()),
            None => Err(anyhow::anyhow!("Document does not exist with id: {}", id))?,
        };
    }

    pub async fn from_title(title: &str, pool: &SqlitePool) -> anyhow::Result<Self> {
        return match DatabaseDoc::from_title(&title.to_lowercase(), pool).await? {
            Some(dbt) => Ok(dbt.into()),
            None => Err(anyhow::anyhow!(
                "Document does not exist with title: {}",
                title
            ))?,
        };
    }

    pub async fn stored_path(&self, pool: &SqlitePool) -> anyhow::Result<PathBuf> {
        return match DatabaseDoc::from_title(&self.title, pool).await? {
            Some(dbd) => dbd.stored_path(),
            None => Err(anyhow::anyhow!("Document is not stored: {:?}", self))?,
        };
    }

    pub async fn insert(self, pool: &SqlitePool) -> anyhow::Result<()> {
        let _ = DatabaseDoc::from_insert(self, pool).await?;
        return Ok(());
    }

    pub async fn delete(self, pool: &SqlitePool) -> anyhow::Result<()> {
        return match DatabaseDoc::from_title(&self.title, pool).await? {
            Some(dbd) => dbd.delete(pool).await,
            None => {
                println!("Document not in DB: {:?}", self);
                return Ok(());
            }
        };
    }

    //pub async fn from_prompts() -> anyhow::Result<Self> {
    //    let print!("\nEnter the author list (delim=';', first/last delim=','): ");
    //    std::io::stdout().flush()?;
    //    let mut author = String::new();
    //    std::io::stdin().read_line(&mut author)?;
    //    let author_names: Vec<String> = author
    //        .split(",")
    //        .take(2)
    //        .map(|n| n.trim().to_lowercase())
    //        .collect::<Vec<String>>();
    //    let [ref last, ref first] = author_names[..2] else {
    //        Err(anyhow::anyhow!("Bad author input"))?
    //    };

    //    print!("\nEnter the title: ");
    //    std::io::stdout().flush()?;
    //    let mut title = String::new();
    //    std::io::stdin().read_line(&mut title)?;

    //    print!("\nEnter the name of the publication: ");
    //    std::io::stdout().flush()?;
    //    let mut publication = String::new();
    //    std::io::stdin().read_line(&mut publication)?;

    //    print!("\nEnter the year of publication (YYYY): ");
    //    std::io::stdout().flush()?;
    //    let mut year_str = String::new();
    //    std::io::stdin().read_line(&mut year_str)?;
    //    let year: u16 = year_str.trim().parse().unwrap_or(0);

    //    print!("\nEnter the publication volume (default = 0): ");
    //    std::io::stdout().flush()?;
    //    let mut buf = String::new();
    //    std::io::stdin().read_line(&mut buf)?;
    //    let volume: u16 = buf.trim().parse().unwrap_or(0);

    //    print!("\nEnter document tags (e.g. \"rust, programming\"): ");
    //    std::io::stdout().flush()?;
    //    let mut buf = String::new();
    //    std::io::stdin().read_line(&mut buf)?;
    //    let tags: String = if buf == "" {
    //        String::new()
    //    } else {
    //        TagInputList::from(buf.trim().trim_matches('"'))
    //            .tag_values()
    //            .join(",")
    //    };

    //    let doc = Document {
    //        title: title.trim().to_lowercase(),
    //        author_last: last.to_owned(),
    //        author_first: first.to_owned(),
    //        publication: publication.trim().to_lowercase(),
    //        year,
    //        volume,
    //        tags,
    //        ..Default::default()
    //    };

    //    println!("Document Entry: {:?}", doc);
    //    print!("Does this look correct ((y)es, (n)o)? ");
    //    std::io::stdout().flush()?;
    //    let mut buf = String::new();
    //    std::io::stdin().read_line(&mut buf)?;
    //    match buf.trim() {
    //        "y" | "yes" => return Ok(doc),
    //        _ => return Err(anyhow::anyhow!("Entry cancelled.")),
    //    }
    //}
}

pub struct DocumentBuilder {
    title: String,
    author: String,
    publication: String,
    volume: u16,
    year: u16,
    tags: String,
    doi: String,
    path: PathBuf,
}

impl DocumentBuilder {
    fn new(title: &str, path: &str) -> Self {
        return Self {
            title: title.to_lowercase(),
            author: String::new(),
            publication: String::new(),
            volume: 0,
            year: 0,
            tags: String::new(),
            doi: String::new(),
            path: PathBuf::from(path),
        };
    }

    fn author(self, author: &str) -> Self {
        let DocumentBuilder {
            title,
            publication,
            volume,
            year,
            tags,
            path,
            doi,
            ..
        } = self;
        return Self {
            title,
            author: author.to_lowercase(),
            year,
            publication,
            volume,
            tags,
            doi,
            path,
        };
    }

    fn publication(self, publication: &str) -> Self {
        let DocumentBuilder {
            title,
            author,
            volume,
            year,
            tags,
            path,
            doi,
            ..
        } = self;
        return Self {
            title,
            author,
            year,
            publication: publication.to_lowercase(),
            volume,
            tags,
            doi,
            path,
        };
    }

    fn volume(self, volume: u16) -> Self {
        let DocumentBuilder {
            title,
            author,
            publication,
            year,
            tags,
            path,
            doi,
            ..
        } = self;
        return Self {
            title,
            author,
            year,
            publication,
            volume,
            tags,
            doi,
            path,
        };
    }

    fn year(self, year: u16) -> Self {
        let DocumentBuilder {
            title,
            author,
            publication,
            volume,
            tags,
            path,
            doi,
            ..
        } = self;
        return Self {
            title,
            author,
            year,
            publication,
            volume,
            tags,
            doi,
            path,
        };
    }

    fn tags(self, tags: &str) -> Self {
        let DocumentBuilder {
            title,
            author,
            year,
            publication,
            volume,
            path,
            doi,
            ..
        } = self;
        return Self {
            title,
            author,
            year,
            publication,
            volume,
            tags: tags.to_lowercase(),
            doi,
            path,
        };
    }

    fn doi(self, doi: &str) -> Self {
        let DocumentBuilder {
            title,
            author,
            year,
            publication,
            volume,
            tags,
            path,
            ..
        } = self;
        return Self {
            title,
            author,
            year,
            publication,
            volume,
            tags,
            doi: doi.to_lowercase(),
            path,
        };
    }

    fn path(self, path: &str) -> Self {
        let DocumentBuilder {
            title,
            author,
            year,
            publication,
            volume,
            tags,
            doi,
            ..
        } = self;
        return Self {
            title,
            author,
            year,
            publication,
            volume,
            tags,
            doi,
            path: PathBuf::from(path),
        };
    }

    fn build(self) -> anyhow::Result<Document> {
        let DocumentBuilder {
            title,
            author,
            year,
            publication,
            volume,
            tags,
            doi,
            path,
        } = self;
        if !path.is_file() || path.extension() != Some(&std::ffi::OsStr::new("pdf")) {
            return Err(anyhow::anyhow!(
                "Path does not reference a valid PDF: {:?}",
                path
            ))?;
        }
        return Ok(Document {
            id: None,
            title,
            author,
            year,
            publication,
            volume,
            tags,
            doi,
            path,
        });
    }
}

#[derive(Deserialize, Debug)]
pub struct TomlDocuments {
    pub documents: Vec<Document>,
}

impl TomlDocuments {
    pub async fn add_to_db(self, pool: &SqlitePool) -> anyhow::Result<()> {
        for doc in self.documents.into_iter() {
            doc.insert(pool).await?;
        }
        return Ok(());
    }
}

#[derive(Debug)]
pub struct DocList(pub Vec<DatabaseDoc>);

impl std::fmt::Display for DocList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return self.0.iter().fold(Ok(()), |result, doc| {
            result.and_then(|_| writeln!(f, "{}", doc))
        });
    }
}

impl std::ops::Deref for DocList {
    type Target = Vec<DatabaseDoc>;
    fn deref(&self) -> &Self::Target {
        return &self.0;
    }
}
