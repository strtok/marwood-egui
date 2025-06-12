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
            input: r#"(define (filter pred lst)
              (cond
                ((null? lst) '())
                ((pred (car lst))
                 (cons (car lst) (filter pred (cdr lst))))
                (else
                 (filter pred (cdr lst)))))

            (define (sieve limit)
              (define (range start end)
                (if (> start end)
                    '()
                    (cons start (range (+ start 1) end))))

              (define (remove-multiples n lst)
                (filter (lambda (x) (not (= (modulo x n) 0))) lst))

              (define (sieve-helper numbers)
                (if (null? numbers)
                    '()
                    (cons (car numbers)
                          (sieve-helper (remove-multiples (car numbers) (cdr numbers))))))

              (sieve-helper (range 2 limit)))

            (sieve 100)"#
                .to_owned(),
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
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
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

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("λMarwood");

            let mut theme =
                egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());

            ui.collapsing("Theme", |ui| {
                ui.group(|ui| {
                    theme.ui(ui);
                    theme.clone().store_in_memory(ui.ctx());
                });
            });

            let mut layouter = |ui: &egui::Ui, buf: &str, wrap_width: f32| {
                let mut layout_job = egui_extras::syntax_highlighting::highlight(
                    ui.ctx(),
                    ui.style(),
                    &theme,
                    buf,
                    "lisp",
                );
                layout_job.wrap.max_width = wrap_width;
                ui.fonts(|f| f.layout_job(layout_job))
            };

            let code = &mut self.input;

            ui.push_id("input_scroll_area", |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(code)
                            .id_source("input")
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
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_enabled(
                        false,
                        egui::TextEdit::multiline(&mut self.output)
                            .id_source("output")
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
                                if cell != Cell::Void {
                                    self.output.insert_str(0, &format!("=> {}\n", cell));
                                }
                                input = rest;
                            }
                            Err(e) => {
                                self.output.insert_str(0, &format!("=> error: {}\n", e));
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
