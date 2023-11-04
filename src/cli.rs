use crate::{Document, Tag};
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct Cli {
    #[command(subcommand)]
    pub entity_type: EntityType,
}

#[derive(Debug, Subcommand)]
pub enum EntityType {
    /// Operations for document tags
    Tag(TagCmd),
    #[command(name = "doc")]
    /// Operations for documents
    Document(DocCmd),
}

#[derive(Debug, Args)]
pub struct TagCmd {
    #[command(subcommand)]
    pub command: TagSubCmd,
}

#[derive(Debug, Subcommand)]
pub enum TagSubCmd {
    /// Add one or multiple tags.  Tag lists must be comma separated.
    Add(AddTag),
    /// Modify stored tags.  (Unimplemented)
    Modify(ModifyTag),
    /// Delete stored tags.
    Delete(DeleteTag),
    /// List stored tags.
    List,
}

#[derive(Debug, Args)]
pub struct AddTag {
    pub name: String,
}

#[derive(Debug, Args)]
pub struct ModifyTag {
    #[command(subcommand)]
    pub method: ModifyTagSubCmd,
}

#[derive(Debug, Subcommand)]
pub enum ModifyTagSubCmd {
    /// Add one or multiple tags.  Tag lists must be comma separated.
    ById(ModifyTagById),
    /// Modify stored tags.  (Unimplemented)
    ByValue(ModifyTagByValue),
}

#[derive(Debug, Args)]
pub struct ModifyTagById {
    #[arg(long, required = true)]
    pub id: u32,
    #[arg(long, required = true, value_parser = Tag::input_to_lowercase)]
    pub new_value: String,
}

#[derive(Debug, Args)]
pub struct ModifyTagByValue {
    #[arg(long, required = true)]
    pub old_value: String,
    #[arg(long, required = true, value_parser = Tag::input_to_lowercase)]
    pub new_value: String,
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct DeleteTag {
    #[arg(long)]
    pub id: Option<u32>,
    #[arg(long)]
    pub name: Option<String>,
}

#[derive(Debug, Args)]
pub struct DocCmd {
    #[command(subcommand)]
    pub command: DocSubCmd,
}

#[derive(Debug, Subcommand)]
pub enum DocSubCmd {
    /// Add one or multiple documents.  Multiple documents must be added via toml file.
    Add(AddDoc),
    /// Modify stored document information.
    Modify(ModifyDoc),
    /// Delete stored documents.
    Delete(DeleteDoc),
    /// List stored documents.
    List,
    /// Open a stored document by id or title.
    Open(OpenDoc),
}

#[derive(Debug, Args)]
pub struct AddDoc {
    #[command(subcommand)]
    pub source: AddDocSubCmd,
}

#[derive(Debug, Subcommand)]
pub enum AddDocSubCmd {
    Single(SingleDoc),
    FromToml(AddDocTomlPath),
}

#[derive(Debug, Args)]
pub struct AddDocTomlPath {
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
    #[command(subcommand)]
    pub method: ModifyDocSubCmd,
}

#[derive(Debug, Subcommand)]
pub enum ModifyDocSubCmd {
    ById(ModifyFieldById),
    ByTitle(ModifyFieldByTitle),
}

#[derive(Debug, Args)]
pub struct ModifyFieldById {
    #[arg(required = true)]
    pub id: u16,
    #[arg(long, value_parser = Document::input_to_lowercase)]
    pub title: String,
    #[arg(long, value_parser = Document::input_to_lowercase)]
    pub author: String,
    #[arg(long)]
    pub year: u16,
    #[arg(long, value_parser = Document::input_to_lowercase)]
    pub publication: String,
    #[arg(long)]
    pub volume: u16,
    #[arg(long, value_parser = Document::input_to_lowercase)]
    pub tags: String,
    #[arg(long)]
    pub doi: String,
    #[arg(long, value_parser = Document::verify_path)]
    pub path: PathBuf,
}

#[derive(Debug, Args)]
pub struct ModifyFieldByTitle {
    #[arg(long, required = true, value_parser = Document::input_to_lowercase)]
    pub title: String,
    #[arg(long, value_parser = Document::input_to_lowercase)]
    pub author: String,
    #[arg(long)]
    pub year: u16,
    #[arg(long, value_parser = Document::input_to_lowercase)]
    pub publication: String,
    #[arg(long)]
    pub volume: u16,
    #[arg(long, value_parser = Document::input_to_lowercase)]
    pub tags: String,
    #[arg(long)]
    pub doi: String,
    #[arg(long, value_parser = Document::verify_path)]
    pub path: PathBuf,
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct DeleteDoc {
    #[arg(long)]
    pub id: Option<u32>,
    #[arg(long)]
    pub title: Option<String>,
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct OpenDoc {
    #[arg(long)]
    pub id: Option<u32>,
    #[arg(long)]
    pub title: Option<String>,
}
