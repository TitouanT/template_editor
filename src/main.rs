#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::egui;
use egui_dnd::dnd;

fn main() -> eframe::Result {
    eframe::run_native(
        "Template editor",
        eframe::NativeOptions::default(),
        Box::new(|creation_context| {
            creation_context.egui_ctx.set_theme(egui::Theme::Dark);
            Ok(Box::<MyApp>::default())
        }),
    )
}

struct MyApp {
    data_dir: Option<std::path::PathBuf>,
    templates: Vec<(String, usize)>,
    new_template: String,
    next_id: usize,
}

impl MyApp {
    fn get_data_dir() -> Option<std::path::PathBuf> {
        let path = dirs::data_dir()?;
        Some(path.join("template_editor"))
    }
    fn get_data_file() -> Option<std::path::PathBuf> {
        let dir = Self::get_data_dir()?;
        Some(dir.join("templates.json"))
    }
    fn read_data() -> Option<Vec<(String, usize)>> {
        let templates_file = Self::get_data_file()?;
        if !templates_file.exists() {
            return None;
        }
        let raw_template = std::fs::read_to_string(templates_file).ok()?;
        let value = serde_json::from_str::<serde_json::Value>(&raw_template).ok()?;
        let serde_templates = value["templates"].as_array()?;
        let values: Vec<(Option<&str>, usize)> = serde_templates.iter().enumerate().map(|(i, v)| (v.as_str(), i)).collect();
        if values.iter().all(|(v, _)| v.is_some()) {
            Some(values.iter().map(|(v, i)| (v.unwrap().to_owned(), *i)).collect())
        }
        else {
            None
        }

    }

    fn write_data(&self) -> Option<()> {
        let templates_dir = Self::get_data_dir()?;
        if !templates_dir.exists() {
            std::fs::create_dir(templates_dir).ok()?;
        }
        let templates_file = Self::get_data_file()?;
        let json_data = serde_json::json!({
            "templates": self.templates.iter().map(|(v, _)| v.clone()).collect::<Vec<String>>(),
        }).to_string();
        std::fs::write(templates_file, json_data).ok()?;
        Some(())
    }

    fn push_template(&mut self) {
        self.templates.push((self.new_template.clone(), self.next_id));
        self.new_template = String::new();
        self.next_id += 1;
        self.write_data();
    }
}

impl Default for MyApp {
    fn default() -> Self {
        let templates = &Self::read_data().unwrap_or_default();
        Self {
            data_dir: Self::get_data_dir(),
            templates: templates.to_vec(),
            new_template: "".to_owned(),
            next_id: templates.len(),
        }
    }
}


impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(path) = &self.data_dir {
                ui.horizontal(|ui| {
                    if ui.button("📋").clicked() {
                        ui.output_mut(|o| o.copied_text = path.clone().into_os_string().into_string().unwrap());
                    }
                    ui.label(format!("The data is saved here: {:?}", path));
                });
            }
            else {
                ui.label("Unable to locate a place to save data to");
            }

            ui.horizontal(|ui| {
                let name_label = ui.label("new template: ");
                let response = ui.text_edit_singleline(&mut self.new_template)
                    .labelled_by(name_label.id);
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.push_template();
                }
                if ui.button("+").clicked() {
                    self.push_template();
                }
            });

            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut del_id: Option<usize> = None;
                let mut need_write = false;
                let response = dnd(ui, "no idea").show_vec(
                    &mut self.templates,
                    |ui, item, handle, _state| {
                        ui.horizontal(|ui| {
                            if ui.button("📋").clicked() {
                                ui.output_mut(|o| o.copied_text = item.0.clone());
                            }
                            handle.ui(ui, |ui| {
                                ui.label("✋");
                            });
                            let response = ui.text_edit_multiline(&mut item.0);
                            if response.lost_focus() {
                                need_write = true;
                            }
                            if ui.button("🗑").clicked() {
                                del_id = Some(item.1);
                            }
                        });
                    }
                );

                if response.is_drag_finished() || need_write {
                    self.write_data();
                }

                if let Some(id_to_delete) = del_id {
                    if let Some(index) = self.templates.iter().position(|value| value.1 == id_to_delete) {
                        self.templates.swap_remove(index);
                        self.write_data();
                    }
                }
            });
        });
    }
}
