use crate::model::{
    Components, FileFormat, Info, MediaType, OpenApiSpec, Operation, Parameter, Ref, RefOr,
    RequestBody, Response, Schema,
};
use indexmap::IndexMap;
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
    /// Parsed original file preserved for structure-preserving saves.
    pub raw: Option<serde_yaml::Value>,
    pub status: String,
    pub new_item: NewItemBuffers,
    pub raw_editor_buf: String,
    pub raw_editor_err: String,
    pub show_exit_dialog: bool,
    pub search_query: String,
}

/// Merge `updated` into `original`, preserving the original mapping key order.
///
/// The merge is purely additive from `original`'s perspective:
/// - Keys present in both: value is updated recursively (in-place, original position kept).
/// - Keys only in `original`: kept as-is (handles `skip_serializing_if` fields that
///   disappeared from the typed-struct serialization, plus any `x-*` extensions).
/// - Keys only in `updated`: appended at the end (new items added by the user).
///
/// Sequences are replaced wholesale rather than merged element-by-element.
///
/// This avoids `Mapping::remove`, which internally uses swap-remove and would
/// silently reorder the surviving keys.
fn merge_yaml(original: &mut serde_yaml::Value, updated: &serde_yaml::Value) {
    let (orig_map, new_map) = match (original, updated) {
        (serde_yaml::Value::Mapping(o), serde_yaml::Value::Mapping(n)) => (o, n),
        (orig, new_val) => { *orig = new_val.clone(); return; }
    };

    // Build a string-keyed lookup into `updated` (all OpenAPI/JSON keys are strings).
    let new_lookup: std::collections::HashMap<&str, &serde_yaml::Value> = new_map
        .iter()
        .filter_map(|(k, v)| {
            if let serde_yaml::Value::String(s) = k { Some((s.as_str(), v)) } else { None }
        })
        .collect();

    // Collect original string keys so we can detect truly new additions later.
    let orig_keys: std::collections::HashSet<&str> = orig_map
        .iter()
        .filter_map(|(k, _)| {
            if let serde_yaml::Value::String(s) = k { Some(s.as_str()) } else { None }
        })
        .collect();

    // Rebuild the mapping in original key order, merging values from `updated`.
    // Keys absent from `updated` (empty skip_serializing_if fields, x-extensions,
    // etc.) are kept with their original values.
    let mut result = serde_yaml::Mapping::new();
    for (k, orig_v) in orig_map.iter() {
        if let serde_yaml::Value::String(s) = k {
            if let Some(&new_v) = new_lookup.get(s.as_str()) {
                let mut merged = orig_v.clone();
                merge_yaml(&mut merged, new_v);
                result.insert(k.clone(), merged);
            } else {
                result.insert(k.clone(), orig_v.clone());
            }
        }
    }

    // Append keys that are genuinely new (added by the user via the GUI).
    for (k, new_v) in new_map.iter() {
        if let serde_yaml::Value::String(s) = k {
            if !orig_keys.contains(s.as_str()) {
                result.insert(k.clone(), new_v.clone());
            }
        }
    }

    *orig_map = result;
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            spec: None,
            current_file: None,
            format: FileFormat::Yaml,
            selection: Selection::None,
            dirty: false,
            raw: None,
            status: "Ready. Open or create an OpenAPI specification.".to_string(),
            new_item: NewItemBuffers::default(),
            raw_editor_buf: String::new(),
            raw_editor_err: String::new(),
            show_exit_dialog: false,
            search_query: String::new(),
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
        self.raw = None;
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
                        self.raw = serde_yaml::from_str(&content).ok();
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
        // Serialize the typed struct to a value tree.
        let updated = match serde_yaml::to_value(&spec) {
            Ok(v) => v,
            Err(e) => {
                self.status = format!("Serialization error: {e}");
                return;
            }
        };

        // Merge updated values into the raw tree, preserving original key order
        // and any extension fields. For new specs (no raw), use updated directly.
        if let Some(raw) = &mut self.raw {
            merge_yaml(raw, &updated);
        } else {
            self.raw = Some(updated);
        }

        let result = match fmt {
            FileFormat::Yaml => serde_yaml::to_string(self.raw.as_ref().unwrap())
                .map_err(|e| e.to_string()),
            FileFormat::Json => serde_json::to_string_pretty(self.raw.as_ref().unwrap())
                .map_err(|e| e.to_string()),
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

    // Helpers to duplicate/delete items ──────────────────────────────────────

    fn unique_name(base: &str, exists: impl Fn(&str) -> bool) -> String {
        let candidate = format!("{base} copy");
        if !exists(&candidate) {
            return candidate;
        }
        let mut i = 2u32;
        loop {
            let candidate = format!("{base} copy {i}");
            if !exists(&candidate) {
                return candidate;
            }
            i += 1;
        }
    }

    fn unique_path_key(base: &str, exists: impl Fn(&str) -> bool) -> String {
        let candidate = format!("{base}_copy");
        if !exists(&candidate) {
            return candidate;
        }
        let mut i = 2u32;
        loop {
            let candidate = format!("{base}_copy{i}");
            if !exists(&candidate) {
                return candidate;
            }
            i += 1;
        }
    }

    pub fn delete_path(&mut self, path: &str) {
        if let Some(spec) = &mut self.spec {
            spec.paths.shift_remove(path);
            self.dirty = true;
        }
        let clear = match &self.selection {
            Selection::Path(p) => p == path,
            Selection::Operation(p, _) => p == path,
            _ => false,
        };
        if clear {
            self.selection = Selection::None;
        }
    }

    pub fn duplicate_path(&mut self, path: &str) {
        if let Some(spec) = &mut self.spec {
            if let Some(item) = spec.paths.get(path).cloned() {
                let new_key = Self::unique_path_key(path, |k| spec.paths.contains_key(k));
                spec.paths.insert(new_key.clone(), item);
                self.dirty = true;
                self.selection = Selection::Path(new_key);
            }
        }
    }

    pub fn delete_schema(&mut self, name: &str) {
        if let Some(spec) = &mut self.spec {
            if let Some(comps) = spec.components.as_mut() {
                comps.schemas.shift_remove(name);
                self.dirty = true;
            }
        }
        let clear = matches!(&self.selection, Selection::Schema(n) if n == name);
        if clear {
            self.selection = Selection::None;
        }
    }

    pub fn duplicate_schema(&mut self, name: &str) {
        if let Some(spec) = &mut self.spec {
            if let Some(comps) = spec.components.as_mut() {
                if let Some(item) = comps.schemas.get(name).cloned() {
                    let new_name = Self::unique_name(name, |k| comps.schemas.contains_key(k));
                    comps.schemas.insert(new_name.clone(), item);
                    self.dirty = true;
                    self.selection = Selection::Schema(new_name);
                }
            }
        }
    }

    pub fn delete_request_body(&mut self, name: &str) {
        if let Some(spec) = &mut self.spec {
            if let Some(comps) = spec.components.as_mut() {
                comps.request_bodies.shift_remove(name);
                self.dirty = true;
            }
        }
        let clear = matches!(&self.selection, Selection::RequestBody(n) if n == name);
        if clear {
            self.selection = Selection::None;
        }
    }

    pub fn duplicate_request_body(&mut self, name: &str) {
        if let Some(spec) = &mut self.spec {
            if let Some(comps) = spec.components.as_mut() {
                if let Some(item) = comps.request_bodies.get(name).cloned() {
                    let new_name =
                        Self::unique_name(name, |k| comps.request_bodies.contains_key(k));
                    comps.request_bodies.insert(new_name.clone(), item);
                    self.dirty = true;
                    self.selection = Selection::RequestBody(new_name);
                }
            }
        }
    }

    pub fn delete_component_response(&mut self, name: &str) {
        if let Some(spec) = &mut self.spec {
            if let Some(comps) = spec.components.as_mut() {
                comps.responses.shift_remove(name);
                self.dirty = true;
            }
        }
        let clear = matches!(&self.selection, Selection::ComponentResponse(n) if n == name);
        if clear {
            self.selection = Selection::None;
        }
    }

    pub fn duplicate_component_response(&mut self, name: &str) {
        if let Some(spec) = &mut self.spec {
            if let Some(comps) = spec.components.as_mut() {
                if let Some(item) = comps.responses.get(name).cloned() {
                    let new_name = Self::unique_name(name, |k| comps.responses.contains_key(k));
                    comps.responses.insert(new_name.clone(), item);
                    self.dirty = true;
                    self.selection = Selection::ComponentResponse(new_name);
                }
            }
        }
    }

    pub fn delete_component_parameter(&mut self, name: &str) {
        if let Some(spec) = &mut self.spec {
            if let Some(comps) = spec.components.as_mut() {
                comps.parameters.shift_remove(name);
                self.dirty = true;
            }
        }
        let clear = matches!(&self.selection, Selection::ComponentParameter(n) if n == name);
        if clear {
            self.selection = Selection::None;
        }
    }

    pub fn duplicate_component_parameter(&mut self, name: &str) {
        if let Some(spec) = &mut self.spec {
            if let Some(comps) = spec.components.as_mut() {
                if let Some(item) = comps.parameters.get(name).cloned() {
                    let new_name = Self::unique_name(name, |k| comps.parameters.contains_key(k));
                    comps.parameters.insert(new_name.clone(), item);
                    self.dirty = true;
                    self.selection = Selection::ComponentParameter(new_name);
                }
            }
        }
    }

    pub fn delete_example(&mut self, name: &str) {
        if let Some(spec) = &mut self.spec {
            if let Some(comps) = spec.components.as_mut() {
                comps.examples.shift_remove(name);
                self.dirty = true;
            }
        }
        let clear = matches!(&self.selection, Selection::Example(n) if n == name);
        if clear {
            self.selection = Selection::None;
        }
    }

    pub fn duplicate_example(&mut self, name: &str) {
        if let Some(spec) = &mut self.spec {
            if let Some(comps) = spec.components.as_mut() {
                if let Some(item) = comps.examples.get(name).cloned() {
                    let new_name = Self::unique_name(name, |k| comps.examples.contains_key(k));
                    comps.examples.insert(new_name.clone(), item);
                    self.dirty = true;
                    self.selection = Selection::Example(new_name);
                }
            }
        }
    }

    // ── Default CRUD paths ────────────────────────────────────────────────────

    pub fn add_default_paths_for_schema(&mut self, schema_name: String) {
        if self.spec.is_none() { return; }

        let (op_prefix, plural_kebab, plural_display) = schema_to_resource_parts(&schema_name);
        let collection = format!("/{plural_kebab}");
        let item       = format!("/{plural_kebab}/{{id}}");
        let schema_ref = format!("#/components/schemas/{schema_name}");

        // Shared id path parameter
        let id_param = RefOr::Item(Parameter {
            name: "id".to_string(),
            in_: "path".to_string(),
            required: Some(true),
            schema: Some(RefOr::Item(Schema {
                type_: Some(serde_json::Value::String("string".to_string())),
                ..Default::default()
            })),
            ..Default::default()
        });

        // Request body referencing the schema
        let make_body = |ref_str: &str| -> RefOr<RequestBody> {
            let mut content = IndexMap::new();
            content.insert(
                "application/json".to_string(),
                MediaType {
                    schema: Some(RefOr::Ref(Ref { ref_: ref_str.to_owned(), ..Default::default() })),
                    ..Default::default()
                },
            );
            RefOr::Item(RequestBody { content, required: Some(true), ..Default::default() })
        };

        // Response helpers
        let ok_resp = || {
            let mut content = IndexMap::new();
            content.insert(
                "application/json".to_string(),
                MediaType {
                    schema: Some(RefOr::Ref(Ref { ref_: schema_ref.clone(), ..Default::default() })),
                    ..Default::default()
                },
            );
            RefOr::Item(Response { description: "OK".to_string(), content, ..Default::default() })
        };
        let list_resp = || {
            let mut content = IndexMap::new();
            content.insert(
                "application/json".to_string(),
                MediaType {
                    schema: Some(RefOr::Item(Schema {
                        type_: Some(serde_json::Value::String("array".to_string())),
                        items: Some(Box::new(Schema {
                            ref_: Some(schema_ref.clone()),
                            ..Default::default()
                        })),
                        ..Default::default()
                    })),
                    ..Default::default()
                },
            );
            RefOr::Item(Response { description: "OK".to_string(), content, ..Default::default() })
        };
        let created_resp = || RefOr::Item(Response { description: "Created".to_string(), ..Default::default() });
        let no_content   = || RefOr::Item(Response { description: "No Content".to_string(), ..Default::default() });
        let not_found    = || RefOr::Item(Response { description: "Not Found".to_string(), ..Default::default() });

        let spec = self.spec.as_mut().unwrap();

        // ── Collection: GET + POST ─────────────────────────────────────────────
        {
            let e = spec.paths.entry(collection.clone()).or_default();

            if e.get.is_none() {
                let mut responses = IndexMap::new();
                responses.insert("200".to_string(), list_resp());
                e.get = Some(Operation {
                    operation_id: Some(format!("list{plural_display}", plural_display = plural_display.replace(' ', ""))),
                    summary: Some(format!("List {plural_display}")),
                    responses,
                    ..Default::default()
                });
            }

            if e.post.is_none() {
                let mut responses = IndexMap::new();
                responses.insert("201".to_string(), created_resp());
                e.post = Some(Operation {
                    operation_id: Some(format!("create{schema_name}")),
                    summary: Some(format!("Create {schema_name}")),
                    request_body: Some(make_body(&schema_ref)),
                    responses,
                    ..Default::default()
                });
            }
        }

        // ── Item: GET + PUT + PATCH + DELETE ──────────────────────────────────
        {
            let e = spec.paths.entry(item.clone()).or_default();

            if e.get.is_none() {
                let mut responses = IndexMap::new();
                responses.insert("200".to_string(), ok_resp());
                responses.insert("404".to_string(), not_found());
                e.get = Some(Operation {
                    operation_id: Some(format!("get{schema_name}")),
                    summary: Some(format!("Get {schema_name}")),
                    parameters: vec![id_param.clone()],
                    responses,
                    ..Default::default()
                });
            }

            if e.put.is_none() {
                let mut responses = IndexMap::new();
                responses.insert("200".to_string(), ok_resp());
                responses.insert("404".to_string(), not_found());
                e.put = Some(Operation {
                    operation_id: Some(format!("update{schema_name}")),
                    summary: Some(format!("Update {schema_name}")),
                    parameters: vec![id_param.clone()],
                    request_body: Some(make_body(&schema_ref)),
                    responses,
                    ..Default::default()
                });
            }

            if e.patch.is_none() {
                let mut responses = IndexMap::new();
                responses.insert("200".to_string(), ok_resp());
                responses.insert("404".to_string(), not_found());
                e.patch = Some(Operation {
                    operation_id: Some(format!("patch{schema_name}")),
                    summary: Some(format!("Partially Update {schema_name}")),
                    parameters: vec![id_param.clone()],
                    request_body: Some(make_body(&schema_ref)),
                    responses,
                    ..Default::default()
                });
            }

            if e.delete.is_none() {
                let mut responses = IndexMap::new();
                responses.insert("204".to_string(), no_content());
                responses.insert("404".to_string(), not_found());
                e.delete = Some(Operation {
                    operation_id: Some(format!("delete{schema_name}")),
                    summary: Some(format!("Delete {schema_name}")),
                    parameters: vec![id_param],
                    responses,
                    ..Default::default()
                });
            }
        }

        let _ = op_prefix;
        self.dirty = true;
        self.selection = Selection::Path(collection);
        self.status = format!("Added default CRUD paths for {schema_name}.");
    }
}

// ── Resource name helpers ─────────────────────────────────────────────────────

/// Converts a PascalCase schema name into resource name parts:
/// - `op_prefix`:      camelCase singular for operationId prefixes ("blogPost")
/// - `plural_kebab`:   URL-safe kebab-case plural ("/blog-posts")
/// - `plural_display`: title-case display name ("Blog Posts")
fn schema_to_resource_parts(name: &str) -> (String, String, String) {
    // PascalCase → kebab-case
    let mut kebab = String::new();
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            kebab.push('-');
        }
        for lower in ch.to_lowercase() {
            kebab.push(lower);
        }
    }

    let plural_kebab = pluralize_kebab(&kebab);

    // camelCase op_prefix (first letter lower)
    let op_prefix = {
        let mut s = name.to_owned();
        if let Some(first) = s.get_mut(0..1) {
            first.make_ascii_lowercase();
        }
        s
    };

    // "blog-posts" → "Blog Posts"
    let plural_display: String = plural_kebab
        .split('-')
        .map(|word| {
            let mut c = word.chars();
            match c.next() {
                None    => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    (op_prefix, plural_kebab, plural_display)
}

fn pluralize_kebab(s: &str) -> String {
    if s.ends_with('y') {
        let stem = &s[..s.len() - 1];
        let before_y = stem.chars().last().unwrap_or('_');
        if "aeiou".contains(before_y) {
            format!("{s}s")           // "day" → "days"
        } else {
            format!("{stem}ies")      // "category" → "categories"
        }
    } else if s.ends_with("sh") || s.ends_with("ch")
           || s.ends_with('x') || s.ends_with('z') || s.ends_with('s') {
        format!("{s}es")              // "bus" → "buses"
    } else {
        format!("{s}s")               // "user" → "users"
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ── Keyboard shortcuts ────────────────────────────────────────────────
        if ctx.input(|i| i.key_pressed(egui::Key::F) && i.modifiers.command) {
            if self.spec.is_some() {
                ctx.memory_mut(|m| m.request_focus(egui::Id::new("sidebar_search")));
            }
        }

        // ── Intercept OS close when there are unsaved changes ─────────────────
        if ctx.input(|i| i.viewport().close_requested()) && self.dirty {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.show_exit_dialog = true;
        }

        // ── Unsaved-changes dialog ────────────────────────────────────────────
        if self.show_exit_dialog {
            let file_name = self
                .current_file
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("this file")
                .to_owned();

            egui::Window::new("Unsaved Changes")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.set_min_width(320.0);
                    ui.label(format!(
                        "Do you want to save changes to \"{}\" before closing?",
                        file_name
                    ));
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new("Your changes will be lost if you don't save.")
                            .weak()
                            .small(),
                    );
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            self.save_file();
                            self.show_exit_dialog = false;
                            if !self.dirty {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        }
                        if ui
                            .button(
                                egui::RichText::new("Don't Save")
                                    .color(egui::Color32::from_rgb(220, 80, 80)),
                            )
                            .clicked()
                        {
                            self.dirty = false;
                            self.show_exit_dialog = false;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_exit_dialog = false;
                        }
                    });
                });
        }

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
                        if self.dirty {
                            self.show_exit_dialog = true;
                        } else {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        ui.close_menu();
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
                        ui.add_space(28.0);
                        crate::logo::draw_logo(ui);
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
            // Raw editor is the source of truth — parse it into both raw and spec.
            self.raw = serde_yaml::from_str(&self.raw_editor_buf).ok();
            self.spec = Some(new_spec);
            self.dirty = true;
            self.status = "Applied raw editor changes.".to_string();
        }
    }
}
