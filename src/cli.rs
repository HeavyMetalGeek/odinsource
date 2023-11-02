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
    pub  command: TagSubCmd,
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
#[group(required = true, multiple = false)]
pub struct AddDoc {
    #[arg(long)]
    pub toml: Option<PathBuf>,
    #[arg(long)]
    pub path: Option<PathBuf>,
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
