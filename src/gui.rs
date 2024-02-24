pub mod api;
pub mod document;
pub mod tag;

use document::DatabaseDoc;
use eframe::egui;
use serde::{Deserialize, Serialize};
use sqlx::{migrate::MigrateDatabase, sqlite::SqliteQueryResult, Sqlite, SqlitePool};
use tag::DatabaseTag;
use tokio::runtime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let viewport = egui::ViewportBuilder::default()
        .with_inner_size(egui::vec2(1280.0, 960.0))
        .with_min_inner_size(egui::vec2(640.0, 360.0));
    let native_options = eframe::NativeOptions {
        viewport,
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        "OdinSource",
        native_options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::<OdinSource>::default()
        }),
    )?;
    Ok(())
}

//#[derive(Deserialize, Serialize)]
//#[serde(default)]
struct OdinSource {
    rt: runtime::Runtime,
    db: SqlitePool,
    tags: Vec<DatabaseTag>,
    docs: Vec<DatabaseDoc>,
    needs_update: bool,
}

impl OdinSource {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        //if let Some(storage) = cc.storage {
        //    return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        //}
        let rt = runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let db = rt.block_on(async { api::setup().await.unwrap() });
        Self {
            ..Default::default()
        }
    }

    fn doc_entry(&self, ui: &mut egui::Ui, title: &str, value: &str) {
        ui.horizontal(|ui| {
            ui.horizontal(|ui| {
                ui.monospace(title);
                ui.set_width(40.0);
                ui.visuals_mut().extreme_bg_color = egui::Color32::DARK_GREEN;
                ui.visuals_mut().extreme_bg_color = egui::Color32::RED;
                ui.visuals_mut().faint_bg_color = egui::Color32::RED;
            });
            ui.horizontal(|ui| {
                ui.label(value);
            });
        });
    }
}

impl Default for OdinSource {
    fn default() -> Self {
        let rt = runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let db = rt.block_on(async { api::setup().await.unwrap() });
        Self {
            rt: runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
            db,
            tags: Vec::new(),
            docs: Vec::new(),
            needs_update: true,
        }
    }
}

impl eframe::App for OdinSource {
    //fn save(&mut self, storage: &mut dyn eframe::Storage) {
    //    eframe::set_value(storage, eframe::APP_KEY, self);
    //}

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        //let mut style = (*ctx.style()).clone();
        //style.text_styles() = [

        //].into();
        //style
        //    .text_styles
        //    .entry(egui::TextStyle::Body)
        //    .and_modify(|font| *font = egui::FontId::new(12.5, egui::FontFamily::Monospace));
        //ctx.set_style(style);

        if self.needs_update {
            self.tags = self
                .rt
                .block_on(async { api::get_tags(&self.db).await.unwrap() });
            self.docs = self
                .rt
                .block_on(async { api::get_docs(&self.db).await.unwrap() });
            self.needs_update = false;
            ctx.request_repaint();
        }
        egui::TopBottomPanel::top("TopMenu").show(ctx, |ui| {
            ui.label("Top Stuff");
        });
        egui::SidePanel::left("ConfigMenu").show(ctx, |ui| {
            ui.label("Side Stuff");
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            let icon = egui::include_image!("/home/mmckenna/Pictures/icons/red_circle.png");
            ui.add(egui::Button::image_and_text(icon, "Center Stuff"));
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("Tags:").heading().underline());
                self.tags.iter().for_each(|tag| {
                    ui.label(format!("{}: {}", tag.id, tag.value));
                });
            });

            ui.vertical(|ui| {
                ui.label(egui::RichText::new("Documents:").heading().underline());
                egui::Grid::new("Documents")
                    .num_columns(1)
                    .min_col_width(200.0)
                    .spacing([20.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        self.docs.iter().for_each(|doc| {
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(&doc.title).strong());
                                ui.horizontal(|ui| {
                                    self.doc_entry(ui, "author:", &doc.author);
                                });
                                ui.horizontal(|ui| {
                                    ui.label(format!("{:20}", "year:"));
                                    ui.label(format!("{}", doc.year));
                                });
                                ui.horizontal(|ui| {
                                    ui.label(format!("{:20}", "publication:"));
                                    ui.label(&doc.publication);
                                });
                                ui.horizontal(|ui| {
                                    ui.label(format!("{:20}", "volume:"));
                                    ui.label(format!("{}", doc.volume));
                                });
                                ui.horizontal(|ui| {
                                    ui.label(format!("{:20}", "tags:"));
                                    ui.label(&doc.tags);
                                });
                                ui.horizontal(|ui| {
                                    ui.label(format!("{:20}", "doi:"));
                                    ui.label(&doc.doi);
                                });
                                ui.horizontal(|ui| {
                                    ui.label(format!("{:20}", "uuid:"));
                                    ui.label(&doc.uuid);
                                });
                            });
                            ui.end_row();
                        });
                    });
            });
        });
    }
}
