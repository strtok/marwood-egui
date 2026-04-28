use marwood::cell::Cell;

use crate::marwood::Marwood;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct MarwoodApp {
    input: String,
    output: String,
    #[serde(skip)]
    marwood: Marwood,
}

impl Default for MarwoodApp {
    fn default() -> Self {
        Self {
            input: String::new(),
            output: String::new(),
            marwood: Marwood::new(),
        }
    }
}

impl MarwoodApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}

impl eframe::App for MarwoodApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {}

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        egui::Panel::top("top_panel").show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("λMarwood");

            let mut theme =
                egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());

            ui.collapsing("Theme", |ui| {
                ui.group(|ui| {
                    theme.ui(ui);
                    theme.clone().store_in_memory(ui.ctx());
                });
            });

            let mut layouter = |ui: &egui::Ui, buf: &dyn egui::TextBuffer, wrap_width: f32| {
                let mut layout_job = egui_extras::syntax_highlighting::highlight(
                    ui.ctx(),
                    ui.style(),
                    &theme,
                    buf.as_str(),
                    "lisp",
                );
                layout_job.wrap.max_width = wrap_width;
                ui.fonts_mut(|f| f.layout_job(layout_job))
            };

            let code = &mut self.input;

            ui.push_id("input_scroll_area", |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(code)
                            .id_salt("input")
                            .font(egui::TextStyle::Monospace) // for cursor height
                            .code_editor()
                            .desired_rows(10)
                            .lock_focus(true)
                            .desired_width(f32::INFINITY)
                            .layouter(&mut layouter),
                    )
                })
            });

            ui.push_id("output_scroll_area", |ui| {
                let row_height = ui.text_style_height(&egui::TextStyle::Monospace);
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .max_height(row_height * 10.0)
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.add_enabled(
                            false,
                            egui::TextEdit::multiline(&mut self.output)
                                .id_salt("output")
                                .font(egui::TextStyle::Monospace) // for cursor height
                                .code_editor()
                                .desired_rows(10)
                                .lock_focus(true)
                                .desired_width(f32::INFINITY),
                        );
                    })
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                if ui.button("run").clicked() {
                    let mut input: Option<&str> = Some(&self.input);
                    while input.is_some() {
                        match self.marwood.vm.eval_text(input.unwrap()) {
                            Ok((cell, rest)) => {
                                let displayed =
                                    std::mem::take(&mut *self.marwood.display_buffer.borrow_mut());
                                if !displayed.is_empty() {
                                    self.output.push_str(&displayed);
                                    if !displayed.ends_with('\n') {
                                        self.output.push('\n');
                                    }
                                }
                                if cell != Cell::Void {
                                    self.output.push_str(&format!("=> {}\n", cell));
                                }
                                input = rest;
                            }
                            Err(e) => {
                                let displayed =
                                    std::mem::take(&mut *self.marwood.display_buffer.borrow_mut());
                                if !displayed.is_empty() {
                                    self.output.push_str(&displayed);
                                    if !displayed.ends_with('\n') {
                                        self.output.push('\n');
                                    }
                                }
                                self.output.push_str(&format!("=> error: {}\n", e));
                                break;
                            }
                        }
                    }
                }
            });

            ui.separator();

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_marwood(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_marwood(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("marwood", "https://github.com/strtok/marwood");
    });
}
