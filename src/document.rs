use crate::tag::{Tag, TagInputList};
use serde::Deserialize;
use sqlx::{FromRow, SqlitePool};
use std::io::Write;
use std::path::{PathBuf, Path};
use uuid::Uuid;

#[derive(FromRow, Debug, Hash, Deserialize)]
#[serde(default)]
pub struct Document {
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
    pub doi: String,
    pub uuid: String,
    #[sqlx(skip)]
    pub original_path: PathBuf,
    #[sqlx(skip)]
    pub stored_path: PathBuf,
}

impl Default for Document {
    fn default() -> Self {
        return Self {
            id: 0,
            title: String::new(),
            author_first: String::new(),
            author_last: String::new(),
            year: 0,
            publication: String::new(),
            volume: 0,
            tags: String::new(),
            doi: String::new(),
            uuid: String::new(),
            original_path: PathBuf::new(),
            stored_path: PathBuf::new(),
        };
    }
}

impl Document {
    pub async fn from_id(id: u32, pool: &SqlitePool) -> anyhow::Result<Self> {
        let doc = sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM documents
            WHERE id=?1
            "#,
        )
        .bind(&id)
        .fetch_one(pool)
        .await?;

        return Ok(doc);
    }

    pub async fn from_title(title: &str, pool: &SqlitePool) -> anyhow::Result<Option<Self>> {
        let mut matches = sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM documents
            WHERE title=?1
            "#,
        )
        .bind(&title)
        .fetch_all(pool)
        .await?;

        return match matches.len() {
            0 => Ok(None),
            1 => Ok(matches.pop()),
            _ => Err(anyhow::anyhow!("Duplicate entries detected!")),
        };
    }

    pub async fn stored_path(&mut self) -> anyhow::Result<&Path> {
        if self.stored_path.is_file() {
            return Ok(self.stored_path.as_path());
        }
        if self.uuid.len() == 0 {
            return Err(anyhow::anyhow!("UUID is not available."));
        }
        let fname = self.uuid.clone() + ".pdf";
        let path = std::path::PathBuf::from(std::env!("DOC_STORE_URL"))
            .join(fname);
        if !path.is_file() {
            return Err(anyhow::anyhow!("{} is not currently stored", self.title));
        }
        self.stored_path = path;
        return Ok(self.stored_path.as_path());
    }

    // Check for existing tags
    pub async fn add_to_db(self, path: PathBuf, pool: &SqlitePool) -> anyhow::Result<()> {
        let Document {
            title,
            author_first,
            author_last,
            year,
            publication,
            volume,
            tags,
            ..
        } = self;

        match Document::from_title(&title, pool).await {
            Ok(doc_opt) => {
                if let Some(doc) = doc_opt {
                    println!("Document already in database: {}", doc.title);
                    return Ok(());
                }
            }
            Err(e) => return Err(e),
        }

        // Store copy of pdf in documents folder
        let uuid = Uuid::new_v4().to_string();
        let original_path = path;
        let stored_path =
            std::path::PathBuf::from(std::env!("DOC_STORE_URL")).join(uuid.clone() + ".pdf");
        std::fs::copy(original_path.clone(), stored_path.clone())?;
        println!("Document {:?} stored as {:?}", original_path, stored_path);

        // Add tags to database
        for tag in TagInputList::from(tags.as_str()).as_tags() {
            tag.add_to_db(&pool).await?;
        }

        // Add entry to database
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
        .bind(title.to_lowercase())
        .bind(author_last.to_lowercase())
        .bind(author_first.to_lowercase())
        .bind(year)
        .bind(publication.to_lowercase())
        .bind(volume)
        .bind(uuid)
        .bind(tags.to_lowercase())
        .execute(pool)
        .await?;
        return Ok(());
    }

    pub async fn delete_from_db(self, pool: &SqlitePool) -> anyhow::Result<()> {
        let Document { title, uuid, .. } = self;
        let fname = uuid + ".pdf";
        let asset_path = std::path::PathBuf::from(std::env!("DOC_STORE_URL")).join(fname);
        if std::fs::remove_file(asset_path.clone()).is_err() {
            println!("Could not delete {:?}.", asset_path);
        } else {
            println!("Document {:?} deleted.", asset_path);
        };

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
        return Ok(());
    }

    pub async fn from_prompts() -> anyhow::Result<Self> {
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
            TagInputList::from(buf.trim().trim_matches('"'))
                .tag_values()
                .join(",")
        };

        let doc = Document {
            title: title.trim().to_lowercase(),
            author_last: last.to_owned(),
            author_first: first.to_owned(),
            publication: publication.trim().to_lowercase(),
            year,
            volume,
            tags,
            ..Default::default()
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
}

#[derive(Deserialize, Debug)]
pub struct TomlDocuments {
    pub documents: Vec<Document>,
}

impl TomlDocuments {
    pub async fn add_to_db(self, pool: &SqlitePool) -> anyhow::Result<()> {
        for mut doc in self.documents.into_iter() {
            doc.title = doc.title.to_lowercase();
            let path = std::mem::take(&mut doc.original_path);
            doc.add_to_db(path, pool).await?;
        }
        return Ok(());
    }
}
