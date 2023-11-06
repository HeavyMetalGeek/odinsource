use crate::{
    document::DatabaseDoc,
    tag::{DatabaseTag, TagInputList, TagList},
    Document, Tag,
};
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct Cli {
    #[command(subcommand)]
    pub entity_type: EntityType,
}

#[derive(Debug, Subcommand)]
pub enum EntityType {
    /// Operations for document tag records.
    Tag(TagCmd),
    #[command(name = "doc")]
    /// Operations for document records.
    Document(DocCmd),
}

#[derive(Debug, Args)]
pub struct TagCmd {
    /// Operation to execute on document tag records.
    #[command(subcommand)]
    pub command: TagSubCmd,
}

#[derive(Debug, Subcommand)]
pub enum TagSubCmd {
    /// Add one or multiple tags.  Tag lists must be comma separated.
    Add(AddTag),
    /// Modify a tag record.
    Modify(ModifyTag),
    /// Delete a tag record.
    Delete(DeleteTag),
    /// List all tag records.
    List,
}

#[derive(Debug, Args)]
pub struct AddTag {
    /// Tag value to add to the tags database.
    pub value: String,
}

#[derive(Debug, Args)]
pub struct ModifyTag {
    /// Method for identifying which tag record to modify.
    #[command(subcommand)]
    pub method: ModifyTagSubCmd,
}

#[derive(Debug, Subcommand)]
pub enum ModifyTagSubCmd {
    /// Identify the tag record to be modified by its ID.
    ById(ModifyTagById),
    /// Identify the tag record to be modified by its value.
    ByValue(ModifyTagByValue),
}

#[derive(Debug, Args)]
pub struct ModifyTagById {
    /// Tag database record ID.
    #[arg(long, required = true)]
    pub id: u32,
    /// Updated value.  Also updates in all documents containing the tag.
    #[arg(long, required = true, value_parser = Tag::input_to_lowercase)]
    pub new_value: String,
}

#[derive(Debug, Args)]
pub struct ModifyTagByValue {
    /// Tag record value currently in database.
    #[arg(long, required = true)]
    pub old_value: String,
    /// Updated tag record value in tags database.  Also updates in all document records containing the tag.
    #[arg(long, required = true, value_parser = Tag::input_to_lowercase)]
    pub new_value: String,
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct DeleteTag {
    /// ID of tag record in database.
    #[arg(long)]
    pub id: Option<u32>,
    /// Value of tag record in database.
    #[arg(long)]
    pub value: Option<String>,
}

#[derive(Debug, Args)]
pub struct DocCmd {
    /// Operation to execute on document records.
    #[command(subcommand)]
    pub command: DocSubCmd,
}

#[derive(Debug, Subcommand)]
pub enum DocSubCmd {
    /// Add one or multiple document records.  Multiple records must be added via TOML file.
    Add(AddDoc),
    /// Modify stored document information.
    Modify(ModifyDoc),
    /// Delete a document record.  Also deletes the reference copy of the PDF.
    Delete(DeleteDoc),
    /// List all document records.
    List,
    /// Open a stored document by id or title.
    Open(OpenDoc),
}

#[derive(Debug, Args)]
pub struct AddDoc {
    /// Method for adding document records.
    #[command(subcommand)]
    pub source: AddDocSubCmd,
}

#[derive(Debug, Subcommand)]
pub enum AddDocSubCmd {
    /// Add a single document by entering field values through CLI options.
    Single(SingleDoc),
    /// Add one or multiple documents from specifications in a TOML document.
    FromToml(AddDocTomlPath),
}

#[derive(Debug, Args)]
pub struct AddDocTomlPath {
    /// Location of the TOML file to be parsed for document record information.
    pub path: PathBuf,
}

impl std::convert::From<&str> for AddDocTomlPath {
    fn from(value: &str) -> Self {
        return Self {
            path: PathBuf::from(value),
        };
    }
}

#[derive(Debug, Args)]
pub struct SingleDoc {
    #[arg(long, value_parser = Document::input_to_lowercase, required = true)]
    pub title: String,
    #[arg(long, value_parser = Document::input_to_lowercase, default_value = "")]
    pub author: String,
    #[arg(long, default_value = "0")]
    pub year: u16,
    #[arg(long, value_parser = Document::input_to_lowercase, default_value = "")]
    pub publication: String,
    #[arg(long, default_value = "0")]
    pub volume: u16,
    #[arg(long, value_parser = Document::input_to_lowercase, default_value = "")]
    pub tags: String,
    #[arg(long, default_value = "")]
    pub doi: String,
    #[arg(long, value_parser = Document::verify_path, required = true)]
    pub path: PathBuf,
}

impl std::convert::Into<Document> for SingleDoc {
    fn into(self) -> Document {
        return Document {
            id: None,
            title: self.title,
            author: self.author,
            year: self.year,
            publication: self.publication,
            volume: self.volume,
            tags: self.tags,
            doi: self.doi,
            path: self.path,
        };
    }
}

#[derive(Debug, Args)]
pub struct ModifyDoc {
    /// Method for identifying which document record to modify.
    #[command(subcommand)]
    pub method: ModifyDocSubCmd,
}

#[derive(Debug, Subcommand)]
pub enum ModifyDocSubCmd {
    /// Specify which document record to modify by its ID.
    ById(ModifyFieldById),
    /// Specify which document record to modify by its title.
    ByTitle(ModifyFieldByTitle),
}

#[derive(Debug, Args)]
pub struct ModifyFieldById {
    #[arg(required = true)]
    pub id: u32,
    #[arg(long, value_parser = Document::input_to_lowercase)]
    pub title: Option<String>,
    #[arg(long, value_parser = Document::input_to_lowercase)]
    pub author: Option<String>,
    #[arg(long)]
    pub year: Option<u16>,
    #[arg(long, value_parser = Document::input_to_lowercase)]
    pub publication: Option<String>,
    #[arg(long)]
    pub volume: Option<u16>,
    #[arg(long, value_parser = Document::input_to_lowercase)]
    pub tags: Option<String>,
    #[arg(long)]
    pub doi: Option<String>,
}

impl ModifyFieldById {
    pub async fn update_doc(self, pool: &sqlx::SqlitePool) -> anyhow::Result<()> {
        let mut doc = match DatabaseDoc::from_id(self.id, &pool).await? {
            Some(doc) => doc,
            None => {
                log::error!("Invalid ID: not found in database");
                return Err(anyhow::anyhow!("No document with ID: {}", self.id));
            }
        };
        if let Some(title) = self.title {
            doc.title = title;
        }
        if let Some(author) = self.author {
            doc.author = author;
        }
        if let Some(year) = self.year {
            doc.year = year;
        }
        if let Some(publication) = self.publication {
            doc.publication = publication;
        }
        if let Some(volume) = self.volume {
            doc.volume = volume;
        }
        if let Some(tags) = self.tags {
            let input_tags = TagInputList::from(tags.as_str());
            let db_tags = TagList::get_all(&pool).await?;
            for itag in input_tags.0.into_iter() {
                if db_tags.0.iter().find(|dbt| dbt.value == itag).is_none() {
                    log::info!("Adding tag to DB: {}", itag);
                    let new_tag = Tag::new(&itag);
                    DatabaseTag::from_insert(new_tag, pool).await?;
                } else {
                    log::debug!("Tag already exists in DB: {}", itag);
                }
            }
            doc.tags = tags;
        }
        if let Some(doi) = self.doi {
            doc.doi = doi;
        }
        return doc.update(pool).await;
    }
}

#[derive(Debug, Args)]
pub struct ModifyFieldByTitle {
    #[arg(long, required = true, value_parser = Document::input_to_lowercase)]
    pub title: String,
    #[arg(long, value_parser = Document::input_to_lowercase)]
    pub author: Option<String>,
    #[arg(long)]
    pub year: Option<u16>,
    #[arg(long, value_parser = Document::input_to_lowercase)]
    pub publication: Option<String>,
    #[arg(long)]
    pub volume: Option<u16>,
    #[arg(long, value_parser = Document::input_to_lowercase)]
    pub tags: Option<String>,
    #[arg(long)]
    pub doi: Option<String>,
}

impl ModifyFieldByTitle {
    pub async fn update_doc(self, pool: &sqlx::SqlitePool) -> anyhow::Result<()> {
        let mut doc = match DatabaseDoc::from_title(&self.title, &pool).await? {
            Some(doc) => doc,
            None => {
                log::error!("No document found with title: {}", self.title);
                return Err(anyhow::anyhow!(
                    "No document found with title: {}",
                    self.title
                ));
            }
        };
        if let Some(author) = self.author {
            doc.author = author;
        }
        if let Some(year) = self.year {
            doc.year = year;
        }
        if let Some(publication) = self.publication {
            doc.publication = publication;
        }
        if let Some(volume) = self.volume {
            doc.volume = volume;
        }
        if let Some(tags) = self.tags {
            let input_tags = TagInputList::from(tags.as_str());
            let db_tags = TagList::get_all(&pool).await?;
            for itag in input_tags.0.into_iter() {
                if db_tags.0.iter().find(|dbt| dbt.value == itag).is_none() {
                    log::info!("Adding tag to DB: {}", itag);
                    let new_tag = Tag::new(&itag);
                    DatabaseTag::from_insert(new_tag, pool).await?;
                } else {
                    log::debug!("Tag already exists in DB: {}", itag);
                }
            }
            doc.tags = tags;
        }
        if let Some(doi) = self.doi {
            doc.doi = doi;
        }
        return doc.update(pool).await;
    }
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct DeleteDoc {
    /// ID of the document record to be deleted.
    #[arg(long)]
    pub id: Option<u32>,
    /// Title of the document record to be deleted.
    #[arg(long)]
    pub title: Option<String>,
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct OpenDoc {
    /// ID of the document record to open.
    #[arg(long)]
    pub id: Option<u32>,
    /// Title of the document record to open.
    #[arg(long)]
    pub title: Option<String>,
}
