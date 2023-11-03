use crate::Document;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

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
#[group(required = true, multiple = false)]
pub struct ModifyTag {
    #[arg(long)]
    pub id: u32,
    #[arg(long)]
    pub name: String,
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
    //#[arg(long)]
    //pub toml: Option<PathBuf>,
    //#[arg(long)]
    //pub path: Option<Document>,
}

#[derive(Debug, Subcommand)]
pub enum AddDocSubCmd {
    Single(Document),
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
pub struct ModifyDoc {
    pub title: String,
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
