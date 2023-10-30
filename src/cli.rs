use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct Cli {
    #[command(subcommand)]
    pub entity_type: EntityType,
}

#[derive(Debug, Subcommand)]
pub enum EntityType {
    Tag(TagCmd),
    #[command(name = "doc")]
    Document(DocCmd),
}

#[derive(Debug, Args)]
pub struct TagCmd {
    #[command(subcommand)]
    pub  command: TagSubCmd,
}

#[derive(Debug, Subcommand)]
pub enum TagSubCmd {
    Add(AddTag),
    Modify(ModifyTag),
    Delete(DeleteTag),
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
    pub id: u32,
    #[arg(long)]
    pub name: String,
}

#[derive(Debug, Args)]
pub struct DocCmd {
    #[command(subcommand)]
    pub command: DocSubCmd,
}

#[derive(Debug, Subcommand)]
pub enum DocSubCmd {
    Add(AddDoc),
    Modify(ModifyDoc),
    Delete(DeleteDoc),
    List,
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
