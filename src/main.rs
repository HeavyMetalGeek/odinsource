pub mod cli;
pub mod document;
pub mod tag;
pub mod api;

use cli::*;
use document::*;
use tag::*;
use api::*;

//use serde::Deserialize;
use clap::Parser;
use sqlx::{migrate::MigrateDatabase, sqlite::SqliteQueryResult, Sqlite, SqlitePool};
use std::convert::From;

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
            DocSubCmd::List(cmd) => {
                if let Some(value) = cmd.tag {
                    log::debug!("Search tag: {}", value);
                    let doc_list = DocList::from_tag(&value, &db).await?;
                    print_doc_list(doc_list).await?;
                } else {
                    print_docs(&db).await?;
                };
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
