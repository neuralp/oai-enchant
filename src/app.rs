use crate::model::{
    Components, FileFormat, Info, OpenApiSpec, Operation, Parameter, RefOr, Response, Schema,
};
use std::path::PathBuf;

// ── Selection ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Selection {
    #[default]
    None,
    Info,
    Servers,
    Tags,
    ExternalDocs,
    Path(String),
    Operation(String, String), // path, method
    Schema(String),
    RequestBody(String),
    ComponentResponse(String),
    ComponentParameter(String),
    Example(String),
    Header(String),
    SecurityScheme(String),
    RawEditor,
}

// ── Add-item buffers (for inline "new item" forms) ────────────────────────────

#[derive(Debug, Default)]
pub struct NewItemBuffers {
    pub path: String,
    pub schema_name: String,
    pub request_body_name: String,
    pub response_name: String,
    pub parameter_name: String,
    pub example_name: String,
    pub response_code: String,
}

// ── App ───────────────────────────────────────────────────────────────────────

pub struct App {
    pub spec: Option<OpenApiSpec>,
    pub current_file: Option<PathBuf>,
    pub format: FileFormat,
    pub selection: Selection,
    pub dirty: bool,
    pub status: String,
    pub new_item: NewItemBuffers,
    pub raw_editor_buf: String,
    pub raw_editor_err: String,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            spec: None,
            current_file: None,
            format: FileFormat::Yaml,
            selection: Selection::None,
            dirty: false,
            status: "Ready. Open or create an OpenAPI specification.".to_string(),
            new_item: NewItemBuffers::default(),
            raw_editor_buf: String::new(),
            raw_editor_err: String::new(),
        }
    }

    /// Serialize the current spec into the raw editor buffer and switch to the raw editor view.
    pub fn open_raw_editor(&mut self) {
        if let Some(spec) = &self.spec {
            self.raw_editor_buf = match self.format {
                FileFormat::Json => serde_json::to_string_pretty(spec).unwrap_or_default(),
                FileFormat::Yaml => serde_yaml::to_string(spec).unwrap_or_default(),
            };
            self.raw_editor_err = String::new();
        }
        self.selection = Selection::RawEditor;
    }

    pub fn new_spec(&mut self) {
        self.spec = Some(OpenApiSpec {
            openapi: "3.1.0".to_string(),
            info: Info {
                title: "New API".to_string(),
                version: "1.0.0".to_string(),
                ..Default::default()
            },
            ..Default::default()
        });
        self.current_file = None;
        self.dirty = true;
        self.selection = Selection::Info;
        self.status = "New specification created.".to_string();
    }

    pub fn open_file(&mut self) {
        let dialog = rfd::FileDialog::new()
            .set_title("Open OpenAPI Specification")
            .add_filter("OpenAPI (YAML/JSON)", &["yaml", "yml", "json"])
            .add_filter("YAML", &["yaml", "yml"])
            .add_filter("JSON", &["json"]);

        if let Some(path) = dialog.pick_file() {
            self.load_file(path);
        }
    }

    pub fn load_file(&mut self, path: PathBuf) {
        match std::fs::read_to_string(&path) {
            Err(e) => {
                self.status = format!("Error reading file: {e}");
            }
            Ok(content) => {
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                let fmt = if ext == "json" {
                    FileFormat::Json
                } else {
                    FileFormat::Yaml
                };

                let result: Result<OpenApiSpec, _> = if fmt == FileFormat::Json {
                    serde_json::from_str(&content).map_err(|e| e.to_string())
                } else {
                    serde_yaml::from_str(&content).map_err(|e| e.to_string())
                };

                match result {
                    Ok(spec) => {
                        self.status = format!("Opened: {}", path.display());
                        self.spec = Some(spec);
                        self.current_file = Some(path);
                        self.format = fmt;
                        self.dirty = false;
                        self.selection = Selection::Info;
                    }
                    Err(e) => {
                        self.status = format!("Parse error: {e}");
                    }
                }
            }
        }
    }

    pub fn save_file(&mut self) {
        if self.current_file.is_none() {
            self.save_as();
            return;
        }
        if let Some(spec) = &self.spec {
            let path = self.current_file.clone().unwrap();
            self.write_spec(spec.clone(), &path, self.format);
        }
    }

    pub fn save_as(&mut self) {
        let dialog = rfd::FileDialog::new()
            .set_title("Save OpenAPI Specification")
            .add_filter("YAML", &["yaml", "yml"])
            .add_filter("JSON", &["json"])
            .set_file_name(
                self.current_file
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("openapi.yaml"),
            );

        if let Some(path) = dialog.save_file() {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            let fmt = if ext == "json" {
                FileFormat::Json
            } else {
                FileFormat::Yaml
            };
            self.format = fmt;
            if let Some(spec) = &self.spec {
                let spec = spec.clone();
                self.write_spec(spec, &path, fmt);
                self.current_file = Some(path);
            }
        }
    }

    fn write_spec(&mut self, spec: OpenApiSpec, path: &PathBuf, fmt: FileFormat) {
        let result = match fmt {
            FileFormat::Json => serde_json::to_string_pretty(&spec).map_err(|e| e.to_string()),
            FileFormat::Yaml => serde_yaml::to_string(&spec).map_err(|e| e.to_string()),
        };
        match result {
            Ok(content) => match std::fs::write(path, content) {
                Ok(()) => {
                    self.dirty = false;
                    self.status = format!("Saved: {}", path.display());
                }
                Err(e) => {
                    self.status = format!("Write error: {e}");
                }
            },
            Err(e) => {
                self.status = format!("Serialization error: {e}");
            }
        }
    }

    pub fn ensure_components(&mut self) {
        if let Some(spec) = &mut self.spec {
            if spec.components.is_none() {
                spec.components = Some(Components::default());
            }
        }
    }

    // Helpers to add items ────────────────────────────────────────────────────

    pub fn add_path(&mut self, path: String) {
        if path.is_empty() {
            return;
        }
        if let Some(spec) = &mut self.spec {
            let key = if path.starts_with('/') {
                path.clone()
            } else {
                format!("/{path}")
            };
            spec.paths.entry(key.clone()).or_default();
            self.dirty = true;
            self.selection = Selection::Path(key);
        }
    }

    pub fn add_operation(&mut self, path: &str, method: &str) {
        if let Some(spec) = &mut self.spec {
            if let Some(item) = spec.paths.get_mut(path) {
                item.set_operation(method, Some(Operation::default()));
                self.dirty = true;
                self.selection =
                    Selection::Operation(path.to_string(), method.to_string());
            }
        }
    }

    pub fn add_schema(&mut self, name: String) {
        if name.is_empty() {
            return;
        }
        self.ensure_components();
        if let Some(spec) = &mut self.spec {
            let comps = spec.components.as_mut().unwrap();
            comps
                .schemas
                .entry(name.clone())
                .or_insert(RefOr::Item(Schema::default()));
            self.dirty = true;
            self.selection = Selection::Schema(name);
        }
    }

    pub fn add_request_body(&mut self, name: String) {
        if name.is_empty() {
            return;
        }
        self.ensure_components();
        use crate::model::RequestBody;
        if let Some(spec) = &mut self.spec {
            let comps = spec.components.as_mut().unwrap();
            comps
                .request_bodies
                .entry(name.clone())
                .or_insert(RefOr::Item(RequestBody::default()));
            self.dirty = true;
            self.selection = Selection::RequestBody(name);
        }
    }

    pub fn add_component_response(&mut self, name: String) {
        if name.is_empty() {
            return;
        }
        self.ensure_components();
        if let Some(spec) = &mut self.spec {
            let comps = spec.components.as_mut().unwrap();
            comps
                .responses
                .entry(name.clone())
                .or_insert(RefOr::Item(Response::default()));
            self.dirty = true;
            self.selection = Selection::ComponentResponse(name);
        }
    }

    pub fn add_component_parameter(&mut self, name: String) {
        if name.is_empty() {
            return;
        }
        self.ensure_components();
        if let Some(spec) = &mut self.spec {
            let comps = spec.components.as_mut().unwrap();
            comps
                .parameters
                .entry(name.clone())
                .or_insert(RefOr::Item(Parameter::default()));
            self.dirty = true;
            self.selection = Selection::ComponentParameter(name);
        }
    }

    pub fn add_example(&mut self, name: String) {
        if name.is_empty() {
            return;
        }
        self.ensure_components();
        use crate::model::OaExample;
        if let Some(spec) = &mut self.spec {
            let comps = spec.components.as_mut().unwrap();
            comps
                .examples
                .entry(name.clone())
                .or_insert(RefOr::Item(OaExample::default()));
            self.dirty = true;
            self.selection = Selection::Example(name);
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ── Menu bar ──────────────────────────────────────────────────────────
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        self.new_spec();
                        ui.close_menu();
                    }
                    if ui.button("Open…").clicked() {
                        self.open_file();
                        ui.close_menu();
                    }
                    ui.separator();
                    let can_save = self.spec.is_some();
                    if ui.add_enabled(can_save, egui::Button::new("Save")).clicked() {
                        self.save_file();
                        ui.close_menu();
                    }
                    if ui.add_enabled(can_save, egui::Button::new("Save As…")).clicked() {
                        self.save_as();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.separator();

                // Title + dirty indicator
                if let Some(spec) = &self.spec {
                    let title = format!(
                        "{} v{}{}",
                        spec.info.title,
                        spec.info.version,
                        if self.dirty { " *" } else { "" }
                    );
                    ui.label(title);
                } else {
                    ui.label("No file open");
                }

                // File path on right side
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(p) = &self.current_file {
                        ui.label(
                            egui::RichText::new(p.display().to_string())
                                .weak()
                                .small(),
                        );
                    }
                });
            });
        });

        // ── Status bar ────────────────────────────────────────────────────────
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&self.status).small().weak());
                if self.spec.is_some() {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(self.format.to_string())
                                .small()
                                .weak(),
                        );
                    });
                }
            });
        });

        if self.spec.is_none() {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("OAI Enchant");
                        ui.add_space(8.0);
                        ui.label("OpenAPI Specification Editor (up to v3.2)");
                        ui.add_space(20.0);
                        if ui.button("  New Specification  ").clicked() {
                            self.new_spec();
                        }
                        ui.add_space(8.0);
                        if ui.button("  Open File…  ").clicked() {
                            self.open_file();
                        }
                    });
                });
            });
            return;
        }

        // ── Sidebar ───────────────────────────────────────────────────────────
        egui::SidePanel::left("sidebar")
            .resizable(true)
            .default_width(250.0)
            .min_width(150.0)
            .max_width(400.0)
            .show(ctx, |ui| {
                crate::sidebar::show(ui, self);
            });

        // ── Editor ────────────────────────────────────────────────────────────
        let is_raw = self.selection == Selection::RawEditor;
        let fmt    = self.format;
        let mut raw_apply: Option<OpenApiSpec> = None;

        egui::CentralPanel::default().show(ctx, |ui| {
            if is_raw {
                raw_apply = crate::editors::show_raw_editor(
                    ui, fmt,
                    &mut self.raw_editor_buf,
                    &mut self.raw_editor_err,
                );
            } else if let Some(spec) = self.spec.as_mut() {
                let changed =
                    crate::editors::show(ui, spec, &self.selection, &mut self.new_item);
                if changed {
                    self.dirty = true;
                    // If a path was renamed, update the selection to the new key.
                    if let Some(new_path) = ui.data_mut(|d| {
                        d.remove_temp::<String>(egui::Id::new("oa_path_rename"))
                    }) {
                        self.selection = Selection::Path(new_path);
                    }
                }
            }
        });

        if let Some(new_spec) = raw_apply {
            self.spec = Some(new_spec);
            self.dirty = true;
            self.status = "Applied raw editor changes.".to_string();
        }
    }
}
