use crate::app::{NewItemBuffers, Selection};
use crate::model::*;
use egui::{Grid, RichText, ScrollArea, Ui};
use indexmap::IndexMap;
use serde_json::Value;

// ── Entry point ───────────────────────────────────────────────────────────────

/// Returns true if any value was changed.
pub fn show(
    ui: &mut Ui,
    spec: &mut OpenApiSpec,
    selection: &Selection,
    new_item: &mut NewItemBuffers,
) -> bool {
    // Cache component ref paths for pickers used throughout the sub-editors.
    {
        let comps = spec.components.as_ref();
        let mk = |prefix: &str, keys: Vec<&str>| -> Vec<String> {
            keys.into_iter().map(|k| format!("{prefix}{k}")).collect()
        };
        let schema_refs   = comps.map(|c| mk("#/components/schemas/",       c.schemas.keys().map(|k| k.as_str()).collect())).unwrap_or_default();
        let param_refs    = comps.map(|c| mk("#/components/parameters/",    c.parameters.keys().map(|k| k.as_str()).collect())).unwrap_or_default();
        let resp_refs     = comps.map(|c| mk("#/components/responses/",     c.responses.keys().map(|k| k.as_str()).collect())).unwrap_or_default();
        let rb_refs       = comps.map(|c| mk("#/components/requestBodies/", c.request_bodies.keys().map(|k| k.as_str()).collect())).unwrap_or_default();
        let example_refs  = comps.map(|c| mk("#/components/examples/",      c.examples.keys().map(|k| k.as_str()).collect())).unwrap_or_default();
        ui.data_mut(|d| {
            d.insert_temp(egui::Id::new("oa_schema_refs"),   schema_refs);
            d.insert_temp(egui::Id::new("oa_param_refs"),    param_refs);
            d.insert_temp(egui::Id::new("oa_resp_refs"),     resp_refs);
            d.insert_temp(egui::Id::new("oa_rb_refs"),       rb_refs);
            d.insert_temp(egui::Id::new("oa_example_refs"),  example_refs);
        });
    }

    let mut changed = false;
    ScrollArea::vertical().id_salt("editor_scroll").show(ui, |ui| {
        ui.add_space(4.0);
        match selection {
            Selection::None => {
                ui.centered_and_justified(|ui| {
                    ui.label(RichText::new("Select an item from the sidebar to edit it.").weak());
                });
            }
            Selection::Info => changed = edit_info(ui, spec),
            Selection::Servers => changed = edit_servers(ui, spec),
            Selection::Tags => changed = edit_tags(ui, spec),
            Selection::Tag(name) => {
                let name = name.clone();
                changed = edit_tag_view(ui, spec, &name);
            }
            Selection::ExternalDocs => changed = edit_external_docs(ui, spec),
            Selection::Path(p) => {
                let path = p.clone();
                changed = edit_path(ui, spec, &path, new_item);
            }
            Selection::Operation(p, m) => {
                let (path, method) = (p.clone(), m.clone());
                changed = edit_operation(ui, spec, &path, &method, new_item);
            }
            Selection::Schema(name) => {
                let name = name.clone();
                changed = edit_schema_by_name(ui, spec, &name);
            }
            Selection::RequestBody(name) => {
                let name = name.clone();
                changed = edit_request_body_by_name(ui, spec, &name);
            }
            Selection::ComponentResponse(name) => {
                let name = name.clone();
                changed = edit_component_response_by_name(ui, spec, &name);
            }
            Selection::ComponentParameter(name) => {
                let name = name.clone();
                changed = edit_component_parameter_by_name(ui, spec, &name);
            }
            Selection::Example(name) => {
                let name = name.clone();
                changed = edit_example_by_name(ui, spec, &name);
            }
            Selection::Header(name) => {
                let name = name.clone();
                changed = edit_header_by_name(ui, spec, &name);
            }
            Selection::SecurityScheme(name) => {
                let name = name.clone();
                changed = edit_security_scheme_by_name(ui, spec, &name);
            }
            // Handled in app.rs before editors::show is called.
            Selection::RawEditor => {}
        }
    });
    changed
}

// ── Raw text editor ───────────────────────────────────────────────────────────

/// Show the full-text editor with syntax highlighting.
/// Returns Some(spec) when the user successfully applies changes.
pub fn show_raw_editor(
    ui: &mut Ui,
    format: FileFormat,
    buf: &mut String,
    err: &mut String,
) -> Option<OpenApiSpec> {
    let mut result: Option<OpenApiSpec> = None;

    // ── Header bar ────────────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.strong(format!("Raw Editor  ({})", format));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let apply = ui.button("Apply Changes");
            if !err.is_empty() {
                ui.label(
                    RichText::new("parse error \u{26a0}")
                        .color(egui::Color32::from_rgb(220, 80, 80))
                        .small(),
                );
            }
            if apply.clicked() {
                let parsed: Result<OpenApiSpec, String> = match format {
                    FileFormat::Json => {
                        serde_json::from_str(buf).map_err(|e| e.to_string())
                    }
                    FileFormat::Yaml => {
                        serde_yaml::from_str(buf).map_err(|e| e.to_string())
                    }
                };
                match parsed {
                    Ok(spec) => { *err = String::new(); result = Some(spec); }
                    Err(e)   => { *err = e; }
                }
            }
        });
    });
    ui.separator();

    // ── Text editor ───────────────────────────────────────────────────────────
    let is_json = format == FileFormat::Json;
    let err_height  = if err.is_empty() { 0.0 } else { 56.0 };
    let editor_height = (ui.available_height() - err_height).max(80.0);

    let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
        let mut job = if is_json {
            highlight_json(ui, text)
        } else {
            highlight_yaml(ui, text)
        };
        job.wrap.max_width = wrap_width;
        ui.fonts(|f| f.layout_job(job))
    };

    egui::ScrollArea::both()
        .id_salt("raw_ed_scroll")
        .max_height(editor_height)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(buf)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .desired_rows(40)
                    .layouter(&mut layouter),
            );
        });

    // ── Parse-error panel ─────────────────────────────────────────────────────
    if !err.is_empty() {
        ui.separator();
        egui::ScrollArea::vertical()
            .id_salt("raw_ed_err_scroll")
            .max_height(48.0)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(err.as_str())
                        .color(egui::Color32::from_rgb(220, 80, 80))
                        .small()
                        .monospace(),
                );
            });
    }

    result
}

// ── Syntax highlighting ───────────────────────────────────────────────────────

fn push_tok(
    job: &mut egui::text::LayoutJob,
    text: &str,
    color: egui::Color32,
    font: &egui::FontId,
) {
    if text.is_empty() { return; }
    job.append(
        text,
        0.0,
        egui::text::TextFormat { font_id: font.clone(), color, ..Default::default() },
    );
}

// ── YAML ──────────────────────────────────────────────────────────────────────

fn highlight_yaml(ui: &egui::Ui, code: &str) -> egui::text::LayoutJob {
    let size   = ui.text_style_height(&egui::TextStyle::Monospace);
    let font   = egui::FontId::monospace(size);
    let normal = ui.visuals().text_color();

    // VS Code Dark+ palette (readable on both themes)
    let c_comment = egui::Color32::from_rgb(106, 153,  85);
    let c_key     = egui::Color32::from_rgb(156, 220, 254);
    let c_ref_key = egui::Color32::from_rgb(197, 134, 192);
    let c_string  = egui::Color32::from_rgb(206, 145, 120);
    let c_number  = egui::Color32::from_rgb(181, 206, 168);
    let c_keyword = egui::Color32::from_rgb( 86, 156, 214);
    let c_marker  = egui::Color32::from_rgb(128, 128, 128);

    let mut job = egui::text::LayoutJob::default();

    for line in code.split('\n') {
        let trimmed    = line.trim_start();
        let indent_len = line.len() - trimmed.len();

        if trimmed.starts_with('#') {
            push_tok(&mut job, line, c_comment, &font);
            push_tok(&mut job, "\n", normal, &font);
            continue;
        }
        if trimmed == "---" || trimmed == "..." {
            push_tok(&mut job, line, c_marker, &font);
            push_tok(&mut job, "\n", normal, &font);
            continue;
        }

        push_tok(&mut job, &line[..indent_len], normal, &font);

        // List marker
        let content = if let Some(rest) = trimmed.strip_prefix("- ") {
            push_tok(&mut job, "- ", c_marker, &font);
            rest
        } else if trimmed == "-" {
            push_tok(&mut job, "-", c_marker, &font);
            push_tok(&mut job, "\n", normal, &font);
            continue;
        } else {
            trimmed
        };

        if let Some(colon) = yaml_key_colon(content) {
            let key  = &content[..colon];
            let rest = &content[colon + 1..];            // everything after ':'
            let kc   = if key == "$ref" { c_ref_key } else { c_key };
            push_tok(&mut job, key, kc, &font);
            push_tok(&mut job, ":", normal, &font);

            if let Some(val_start) = rest.find(|c: char| !c.is_whitespace()) {
                let leading = &rest[..val_start];
                let value   = &rest[val_start..];
                push_tok(&mut job, leading, normal, &font);

                let (val_part, cmt_part) = yaml_split_comment(value);
                let vc = if key == "$ref" {
                    c_ref_key
                } else {
                    yaml_value_color(val_part, c_string, c_number, c_keyword, normal)
                };
                push_tok(&mut job, val_part, vc, &font);
                push_tok(&mut job, cmt_part, c_comment, &font);
            }
        } else {
            // Bare scalar (list item value, multiline continuation, etc.)
            push_tok(&mut job, content, yaml_value_color(content, c_string, c_number, c_keyword, normal), &font);
        }

        push_tok(&mut job, "\n", normal, &font);
    }

    job
}

/// Find the byte offset of the `:` that ends a YAML key (not inside quotes).
fn yaml_key_colon(s: &str) -> Option<usize> {
    let mut in_sq = false;
    let mut in_dq = false;
    for (i, c) in s.char_indices() {
        match c {
            '\'' if !in_dq => in_sq = !in_sq,
            '"'  if !in_sq => in_dq = !in_dq,
            ':' if !in_sq && !in_dq => {
                let after = &s[i + 1..];
                if after.is_empty() || after.starts_with(' ') || after.starts_with('\t') {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Split `value # comment` → ("value", " # comment").
fn yaml_split_comment(s: &str) -> (&str, &str) {
    let b = s.as_bytes();
    let mut in_sq = false;
    let mut in_dq = false;
    for i in 0..b.len() {
        match b[i] {
            b'\'' if !in_dq => in_sq = !in_sq,
            b'"'  if !in_sq => in_dq = !in_dq,
            b'#'  if !in_sq && !in_dq && i > 0 && b[i - 1] == b' ' => {
                return (s[..i - 1].trim_end(), &s[i - 1..]);
            }
            _ => {}
        }
    }
    (s, "")
}

fn yaml_value_color(
    val: &str,
    c_string: egui::Color32,
    c_number: egui::Color32,
    c_keyword: egui::Color32,
    normal: egui::Color32,
) -> egui::Color32 {
    if val.starts_with('\'') || val.starts_with('"') { return c_string; }
    match val.trim_end() {
        "true" | "false" | "null" | "~" | "yes" | "no" | "on" | "off" => return c_keyword,
        _ => {}
    }
    let t = val.trim_end();
    let first = t.chars().next().unwrap_or(' ');
    if first.is_ascii_digit()
        || (first == '-' && t.chars().nth(1).map(|c| c.is_ascii_digit()).unwrap_or(false))
    {
        if t.chars().all(|c| c.is_ascii_digit() || "._eE+-".contains(c)) {
            return c_number;
        }
    }
    normal
}

// ── JSON ──────────────────────────────────────────────────────────────────────

fn highlight_json(ui: &egui::Ui, code: &str) -> egui::text::LayoutJob {
    let size   = ui.text_style_height(&egui::TextStyle::Monospace);
    let font   = egui::FontId::monospace(size);
    let normal = ui.visuals().text_color();

    let c_key   = egui::Color32::from_rgb(156, 220, 254);
    let c_str   = egui::Color32::from_rgb(206, 145, 120);
    let c_num   = egui::Color32::from_rgb(181, 206, 168);
    let c_bool  = egui::Color32::from_rgb( 86, 156, 214);
    let c_punct = egui::Color32::from_rgb(204, 204, 204);

    let mut job  = egui::text::LayoutJob::default();
    let bytes    = code.as_bytes();
    let n        = bytes.len();
    let mut i    = 0;
    let mut seg  = 0; // start of current non-string segment

    while i < n {
        if bytes[i] == b'"' {
            // Flush non-string segment
            if i > seg {
                json_nonstring(&mut job, &code[seg..i], normal, c_num, c_bool, c_punct, &font);
            }
            let str_start = i;
            i += 1;
            while i < n {
                if bytes[i] == b'\\' { i += 2; continue; }
                if bytes[i] == b'"' { i += 1; break; }
                i += 1;
            }
            // Is this string a key? (next non-whitespace char is ':')
            let is_key = code[i..].trim_start_matches(|c: char| c.is_whitespace()).starts_with(':');
            push_tok(&mut job, &code[str_start..i], if is_key { c_key } else { c_str }, &font);
            seg = i;
        } else {
            i += 1;
        }
    }
    if seg < n {
        json_nonstring(&mut job, &code[seg..], normal, c_num, c_bool, c_punct, &font);
    }

    job
}

fn json_nonstring(
    job: &mut egui::text::LayoutJob,
    text: &str,
    normal: egui::Color32,
    c_num: egui::Color32,
    c_bool: egui::Color32,
    c_punct: egui::Color32,
    font: &egui::FontId,
) {
    let b = text.as_bytes();
    let mut i = 0;
    while i < b.len() {
        let c = b[i] as char;
        if "{}[]:,".contains(c) {
            push_tok(job, &text[i..i + 1], c_punct, font);
            i += 1;
        } else if text[i..].starts_with("true") {
            push_tok(job, "true",  c_bool, font); i += 4;
        } else if text[i..].starts_with("false") {
            push_tok(job, "false", c_bool, font); i += 5;
        } else if text[i..].starts_with("null") {
            push_tok(job, "null",  c_bool, font); i += 4;
        } else if c.is_ascii_digit()
            || (c == '-' && i + 1 < b.len() && (b[i + 1] as char).is_ascii_digit())
        {
            let start = i;
            i += 1;
            while i < b.len() && "0123456789.eE+-".contains(b[i] as char) {
                i += 1;
            }
            push_tok(job, &text[start..i], c_num, font);
        } else {
            let len = c.len_utf8();
            push_tok(job, &text[i..i + len], normal, font);
            i += len;
        }
    }
}

// ── Form helpers ──────────────────────────────────────────────────────────────

fn row_str(ui: &mut Ui, label: &str, val: &mut String) -> bool {
    ui.label(label);
    let r = ui.text_edit_singleline(val).changed();
    ui.end_row();
    r
}

fn row_opt_str(ui: &mut Ui, label: &str, val: &mut Option<String>) -> bool {
    let mut s = val.clone().unwrap_or_default();
    ui.label(label);
    let r = ui.text_edit_singleline(&mut s).changed();
    ui.end_row();
    if r {
        *val = if s.is_empty() { None } else { Some(s) };
    }
    r
}

fn row_opt_multiline(ui: &mut Ui, label: &str, val: &mut Option<String>) -> bool {
    let mut s = val.clone().unwrap_or_default();
    ui.label(label);
    let r = ui
        .add(egui::TextEdit::multiline(&mut s).desired_rows(4).desired_width(f32::INFINITY))
        .changed();
    ui.end_row();
    if r {
        *val = if s.is_empty() { None } else { Some(s) };
    }
    r
}

fn row_opt_bool(ui: &mut Ui, label: &str, val: &mut Option<bool>) -> bool {
    ui.label(label);
    let mut b = val.unwrap_or(false);
    let r = toggle_switch(ui, &mut b);
    ui.end_row();
    if r.changed() {
        *val = Some(b);
        true
    } else {
        false
    }
}

fn row_opt_u64(ui: &mut Ui, label: &str, val: &mut Option<u64>) -> bool {
    let mut s = val.map(|v| v.to_string()).unwrap_or_default();
    ui.label(label);
    let r = ui.text_edit_singleline(&mut s).changed();
    ui.end_row();
    if r {
        *val = s.parse().ok();
    }
    r
}

fn row_opt_f64(ui: &mut Ui, label: &str, val: &mut Option<f64>) -> bool {
    let mut s = val.map(|v| v.to_string()).unwrap_or_default();
    ui.label(label);
    let r = ui.text_edit_singleline(&mut s).changed();
    ui.end_row();
    if r {
        *val = s.parse().ok();
    }
    r
}

fn toggle_switch(ui: &mut Ui, on: &mut bool) -> egui::Response {
    let size = egui::vec2(36.0, 20.0);
    let (rect, mut response) = ui.allocate_exact_size(size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    if ui.is_rect_visible(rect) {
        let t = ui.ctx().animate_bool(response.id, *on);
        let off = ui.visuals().widgets.inactive.bg_fill;
        let on_c = ui.visuals().selection.bg_fill;
        let bg = egui::Color32::from_rgb(
            (off.r() as f32 + (on_c.r() as f32 - off.r() as f32) * t) as u8,
            (off.g() as f32 + (on_c.g() as f32 - off.g() as f32) * t) as u8,
            (off.b() as f32 + (on_c.b() as f32 - off.b() as f32) * t) as u8,
        );
        ui.painter().rect_filled(rect, size.y / 2.0, bg);
        let r = size.y / 2.0 - 2.0;
        let cx = egui::lerp((rect.left() + r + 2.0)..=(rect.right() - r - 2.0), t);
        ui.painter().circle_filled(egui::pos2(cx, rect.center().y), r, egui::Color32::WHITE);
    }
    response
}

/// Edit an Option<Value> as a plain text field. String values shown without JSON quotes.
fn opt_json_field(ui: &mut Ui, val: &mut Option<Value>, width: f32, hint: &str) -> bool {
    let mut s = match val.as_ref() {
        None => String::new(),
        Some(Value::String(s)) => s.clone(),
        Some(v) => serde_json::to_string(v).unwrap_or_default(),
    };
    let r = ui.add(egui::TextEdit::singleline(&mut s).desired_width(width).hint_text(hint)).changed();
    if r {
        *val = if s.is_empty() {
            None
        } else {
            serde_json::from_str(&s).ok().or_else(|| Some(Value::String(s.clone())))
        };
    }
    r
}

/// Edit an exclusive bound (OAS 3.0 bool or 3.1 number) as a plain text field.
fn excl_bound_field(ui: &mut Ui, val: &mut Option<Value>, width: f32, hint: &str) -> bool {
    let mut s = match val.as_ref() {
        None => String::new(),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::Bool(b)) => b.to_string(),
        Some(v) => serde_json::to_string(v).unwrap_or_default(),
    };
    let r = ui.add(egui::TextEdit::singleline(&mut s).desired_width(width).hint_text(hint)).changed();
    if r {
        *val = if s.is_empty() {
            None
        } else if let Ok(n) = s.parse::<f64>() {
            serde_json::Number::from_f64(n).map(Value::Number)
        } else if let Ok(b) = s.parse::<bool>() {
            Some(Value::Bool(b))
        } else {
            None
        };
    }
    r
}

/// Text field + dropdown picker for a $ref value.
/// `refs_key` is the egui temp-data key holding a `Vec<String>` of candidate paths.
/// Returns true if the value changed.
fn ref_picker(ui: &mut Ui, ref_val: &mut Option<String>, combo_id: egui::Id, refs_key: egui::Id) -> bool {
    let mut r = ref_val.clone().unwrap_or_default();
    let refs: Vec<String> = ui.data(|d| d.get_temp::<Vec<String>>(refs_key).unwrap_or_default());
    let mut ch = false;

    if ui
        .add(egui::TextEdit::singleline(&mut r).desired_width(200.0).hint_text("#/components/…"))
        .changed()
    {
        *ref_val = if r.is_empty() { None } else { Some(r.clone()) };
        ch = true;
    }

    if !refs.is_empty() {
        let selected_name = refs
            .iter()
            .find(|p| **p == r)
            .and_then(|p| p.split('/').last())
            .unwrap_or("pick…");
        egui::ComboBox::from_id_salt(combo_id)
            .selected_text(selected_name)
            .show_ui(ui, |ui| {
                // empty / clear option
                if ui.selectable_label(r.is_empty(), "— clear —").clicked() {
                    *ref_val = None;
                    ch = true;
                }
                for ref_path in &refs {
                    let name = ref_path.split('/').last().unwrap_or(ref_path.as_str());
                    if ui.selectable_label(r == *ref_path, name).clicked() {
                        *ref_val = Some(ref_path.clone());
                        ch = true;
                    }
                }
            });
    }
    ch
}

fn format_options(type_str: &str) -> &'static [&'static str] {
    match type_str {
        "string" => &[
            "", "date", "date-time", "time", "duration",
            "email", "idn-email", "hostname", "idn-hostname",
            "ipv4", "ipv6", "uri", "uri-reference", "iri", "iri-reference",
            "uuid", "uri-template", "json-pointer", "relative-json-pointer",
            "regex", "byte", "binary", "password",
        ],
        "number"  => &["", "float", "double"],
        "integer" => &["", "int32", "int64"],
        _         => &[""],
    }
}

fn section_header(ui: &mut Ui, label: &str) {
    ui.add_space(8.0);
    ui.separator();
    ui.label(RichText::new(label).strong());
    ui.add_space(2.0);
}

fn form_grid(ui: &mut Ui, id: &str, f: impl FnOnce(&mut Ui) -> bool) -> bool {
    let mut changed = false;
    Grid::new(id)
        .num_columns(2)
        .spacing([12.0, 6.0])
        .min_col_width(120.0)
        .show(ui, |ui| {
            changed = f(ui);
        });
    changed
}

// ── Editors ───────────────────────────────────────────────────────────────────

fn edit_info(ui: &mut Ui, spec: &mut OpenApiSpec) -> bool {
    ui.heading("API Info");
    let mut ch = false;
    ch |= form_grid(ui, "info_grid", |ui| {
        let mut c = false;
        c |= row_str(ui, "OpenAPI Version:", &mut spec.openapi);
        c |= row_str(ui, "Title:", &mut spec.info.title);
        c |= row_opt_str(ui, "Summary:", &mut spec.info.summary);
        c |= row_str(ui, "API Version:", &mut spec.info.version);
        c |= row_opt_str(ui, "Terms of Service:", &mut spec.info.terms_of_service);
        c
    });

    section_header(ui, "Description");
    ch |= {
        let mut s = spec.info.description.clone().unwrap_or_default();
        let r = ui
            .add(egui::TextEdit::multiline(&mut s).desired_rows(6).desired_width(f32::INFINITY))
            .changed();
        if r {
            spec.info.description = if s.is_empty() { None } else { Some(s) };
        }
        r
    };

    section_header(ui, "Contact");
    let contact = spec.info.contact.get_or_insert_with(Contact::default);
    ch |= form_grid(ui, "contact_grid", |ui| {
        let mut c = false;
        c |= row_opt_str(ui, "Name:", &mut contact.name);
        c |= row_opt_str(ui, "URL:", &mut contact.url);
        c |= row_opt_str(ui, "Email:", &mut contact.email);
        c
    });

    section_header(ui, "License");
    let license = spec.info.license.get_or_insert_with(License::default);
    ch |= form_grid(ui, "license_grid", |ui| {
        let mut c = false;
        c |= row_str(ui, "Name:", &mut license.name);
        c |= row_opt_str(ui, "SPDX Identifier:", &mut license.identifier);
        c |= row_opt_str(ui, "URL:", &mut license.url);
        c
    });

    ch
}

fn edit_servers(ui: &mut Ui, spec: &mut OpenApiSpec) -> bool {
    ui.heading("Servers");
    let mut ch = false;
    let mut to_remove: Option<usize> = None;

    for (i, srv) in spec.servers.iter_mut().enumerate() {
        egui::CollapsingHeader::new(
            RichText::new(format!("Server {}: {}", i, srv.url)).monospace(),
        )
        .id_salt(format!("srv_{i}"))
        .default_open(i == 0)
        .show(ui, |ui| {
            ch |= form_grid(ui, &format!("srv_grid_{i}"), |ui| {
                let mut c = false;
                c |= row_str(ui, "URL:", &mut srv.url);
                c |= row_opt_str(ui, "Description:", &mut srv.description);
                c
            });
            if ui.small_button("🗑 Remove").clicked() {
                to_remove = Some(i);
            }
        });
    }

    if let Some(idx) = to_remove {
        spec.servers.remove(idx);
        ch = true;
    }

    if ui.button("+ Add Server").clicked() {
        spec.servers.push(Server::default());
        ch = true;
    }
    ch
}

fn edit_tags(ui: &mut Ui, spec: &mut OpenApiSpec) -> bool {
    ui.heading("Tags");
    let mut ch = false;
    let mut to_remove: Option<usize> = None;

    for (i, tag) in spec.tags.iter_mut().enumerate() {
        egui::CollapsingHeader::new(RichText::new(&tag.name))
            .id_salt(format!("tag_{i}"))
            .default_open(i == 0)
            .show(ui, |ui| {
                ch |= form_grid(ui, &format!("tag_grid_{i}"), |ui| {
                    let mut c = false;
                    c |= row_str(ui, "Name:", &mut tag.name);
                    c |= row_opt_multiline(ui, "Description:", &mut tag.description);
                    c
                });
                if ui.small_button("🗑 Remove").clicked() {
                    to_remove = Some(i);
                }
            });
    }

    if let Some(idx) = to_remove {
        spec.tags.remove(idx);
        ch = true;
    }

    if ui.button("+ Add Tag").clicked() {
        spec.tags.push(Tag { name: format!("tag{}", spec.tags.len() + 1), ..Default::default() });
        ch = true;
    }
    ch
}

fn edit_tag_view(ui: &mut Ui, spec: &mut OpenApiSpec, name: &str) -> bool {
    ui.heading(format!("Tag: {name}"));
    ui.add_space(4.0);

    let Some(idx) = spec.tags.iter().position(|t| t.name == name) else {
        ui.label(RichText::new("Tag not found.").weak());
        return false;
    };

    let mut ch = false;

    // ── Editor ────────────────────────────────────────────────────────────────
    let old_name = spec.tags[idx].name.clone();
    ch |= form_grid(ui, "tag_view_grid", |ui| {
        let mut c = false;
        c |= row_str(ui, "Name:", &mut spec.tags[idx].name);
        c |= row_opt_multiline(ui, "Description:", &mut spec.tags[idx].description);
        c
    });

    let new_name = spec.tags[idx].name.clone();
    if new_name != old_name {
        for path_item in spec.paths.values_mut() {
            for method in ["GET","PUT","POST","DELETE","OPTIONS","HEAD","PATCH","TRACE"] {
                if let Some(op) = path_item.operation_mut(method) {
                    for t in op.tags.iter_mut() {
                        if t.as_str() == old_name { *t = new_name.clone(); }
                    }
                }
            }
        }
        ui.data_mut(|d| d.insert_temp(egui::Id::new("oa_tag_renamed"), new_name.clone()));
    }

    if ui.small_button("🗑 Delete Tag").clicked() {
        for path_item in spec.paths.values_mut() {
            for method in ["GET","PUT","POST","DELETE","OPTIONS","HEAD","PATCH","TRACE"] {
                if let Some(op) = path_item.operation_mut(method) {
                    op.tags.retain(|t| t.as_str() != name);
                }
            }
        }
        spec.tags.remove(idx);
        ch = true;
        ui.data_mut(|d| d.insert_temp(egui::Id::new("oa_tag_deleted"), true));
        return ch;
    }

    ui.separator();

    // ── Operations that use this tag ──────────────────────────────────────────
    section_header(ui, "Operations");

    let display_name = new_name.as_str();

    let method_color = |m: &str| match m {
        "GET"    => egui::Color32::from_rgb( 97, 175,  95),
        "POST"   => egui::Color32::from_rgb(100, 149, 237),
        "PUT"    => egui::Color32::from_rgb(229, 152,  61),
        "DELETE" => egui::Color32::from_rgb(220,  80,  80),
        "PATCH"  => egui::Color32::from_rgb(180, 120, 220),
        _        => egui::Color32::GRAY,
    };

    let mut any = false;
    for (path_key, path_item) in &spec.paths {
        for (method, op) in path_item.operations() {
            if !op.tags.iter().any(|t| t == display_name) { continue; }
            any = true;
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("{method:7}"))
                        .monospace()
                        .color(method_color(method)),
                );
                ui.label(RichText::new(path_key).monospace());
                if let Some(s) = &op.summary {
                    ui.label(RichText::new(format!("— {s}")).weak());
                }
                if ui.link("Edit").clicked() {
                    ui.data_mut(|d| d.insert_temp(
                        egui::Id::new("oa_navigate_operation"),
                        (path_key.to_string(), method.to_string()),
                    ));
                }
            });
        }
    }

    if !any {
        ui.label(RichText::new("No operations use this tag.").weak().italics());
    }

    ch
}

fn edit_external_docs(ui: &mut Ui, spec: &mut OpenApiSpec) -> bool {
    ui.heading("External Docs");
    let docs = spec.external_docs.get_or_insert_with(ExternalDocs::default);
    form_grid(ui, "ext_docs_grid", |ui| {
        let mut c = false;
        c |= row_str(ui, "URL:", &mut docs.url);
        c |= row_opt_multiline(ui, "Description:", &mut docs.description);
        c
    })
}

fn edit_path(ui: &mut Ui, spec: &mut OpenApiSpec, path: &str, _new_item: &mut NewItemBuffers) -> bool {
    // ── Editable path ─────────────────────────────────────────────────────────
    // Use a tracking key so the buffer resets whenever the user navigates to a
    // different path (avoids stale text from the previous selection).
    let buf_id     = egui::Id::new("path_edit_buf");
    let tracked_id = egui::Id::new("path_edit_tracked");
    let tracked: String = ui.data(|d| d.get_temp(tracked_id).unwrap_or_default());
    if tracked != path {
        ui.data_mut(|d| {
            d.insert_temp(tracked_id, path.to_string());
            d.insert_temp(buf_id,     path.to_string());
        });
    }
    let mut buf: String = ui.data(|d| d.get_temp(buf_id).unwrap_or_else(|| path.to_string()));

    ui.heading("Path Item");
    let mut renamed_to: Option<String> = None;
    ui.horizontal(|ui| {
        ui.label("Path:");
        let resp = ui.add(
            egui::TextEdit::singleline(&mut buf)
                .font(egui::TextStyle::Monospace)
                .desired_width(340.0)
                .hint_text("/new-path"),
        );
        ui.data_mut(|d| d.insert_temp(buf_id, buf.clone()));

        if resp.lost_focus() && !buf.trim().is_empty() && buf != path {
            let new_key = if buf.starts_with('/') { buf.clone() } else { format!("/{buf}") };
            if spec.paths.contains_key(&new_key) {
                // Conflict — reset buffer
                ui.data_mut(|d| d.insert_temp(buf_id, path.to_string()));
                ui.label(RichText::new("path already exists").color(egui::Color32::from_rgb(220, 80, 80)).small());
            } else {
                renamed_to = Some(new_key);
            }
        }
    });

    // Apply rename, preserving IndexMap insertion order
    if let Some(new_key) = renamed_to {
        let reordered: Vec<(String, PathItem)> = std::mem::take(&mut spec.paths)
            .into_iter()
            .map(|(k, v)| if k == path { (new_key.clone(), v) } else { (k, v) })
            .collect();
        spec.paths = reordered.into_iter().collect();
        // Signal app.rs to update the Selection so the sidebar stays in sync
        ui.data_mut(|d| d.insert_temp(egui::Id::new("oa_path_rename"), new_key));
        return true; // next frame re-enters with the new path
    }

    ui.add_space(4.0);

    let Some(item) = spec.paths.get_mut(path) else {
        ui.label("Path not found.");
        return false;
    };

    let mut ch = false;
    ch |= form_grid(ui, "path_grid", |ui| {
        let mut c = false;
        c |= row_opt_str(ui, "Summary:", &mut item.summary);
        c |= row_opt_multiline(ui, "Description:", &mut item.description);
        c
    });

    section_header(ui, "Operations");
    let methods = ["GET","POST","PUT","DELETE","PATCH","OPTIONS","HEAD","TRACE"];
    for method in methods {
        let has_op = item.operation_mut(method).is_some();
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("{method:8}"))
                    .monospace()
                    .color(if has_op { egui::Color32::from_rgb(97, 175, 95) } else { egui::Color32::GRAY }),
            );
            if has_op {
                if ui.small_button("🗑 Remove").clicked() {
                    item.set_operation(method, None);
                    ch = true;
                }
                if ui.link("Edit").clicked() {
                    ui.data_mut(|d| d.insert_temp(
                        egui::Id::new("oa_navigate_operation"),
                        (path.to_string(), method.to_string()),
                    ));
                }
            } else if ui.small_button("+ Add").clicked() {
                item.set_operation(method, Some(Operation::default()));
                ch = true;
            }
        });
    }
    ch
}

fn edit_operation(
    ui: &mut Ui,
    spec: &mut OpenApiSpec,
    path: &str,
    method: &str,
    new_item: &mut NewItemBuffers,
) -> bool {
    ui.heading(format!("{method} {path}"));
    ui.add_space(4.0);

    // Collect read-only data from spec before taking the mutable path borrow.
    let defined_tags: Vec<String> = spec.tags.iter().map(|t| t.name.clone()).collect();
    let security_scheme_names: Vec<String> = spec.components.as_ref()
        .map(|c| c.security_schemes.keys().cloned().collect())
        .unwrap_or_default();
    let global_security = spec.security.clone();

    let Some(item) = spec.paths.get_mut(path) else {
        ui.label("Path not found.");
        return false;
    };
    let Some(op) = item.operation_mut(method) else {
        ui.label("Operation not found.");
        return false;
    };

    let mut ch = false;

    // Basic fields
    ch |= form_grid(ui, "op_basic_grid", |ui| {
        let mut c = false;
        c |= row_opt_str(ui, "Operation ID:", &mut op.operation_id);
        c |= row_opt_str(ui, "Summary:", &mut op.summary);

        // Tag multi-select: chips for assigned tags, dropdown to add more
        ui.label("Tags:");
        let mut remove_idx: Option<usize> = None;
        let mut add_tag: Option<String> = None;
        ui.horizontal_wrapped(|ui| {
            // Assigned tag chips — click to remove
            for (i, tag) in op.tags.iter().enumerate() {
                if ui
                    .selectable_label(true, RichText::new(format!("{tag}  \u{00d7}")).small())
                    .on_hover_text("Click to remove")
                    .clicked()
                {
                    remove_idx = Some(i);
                }
            }
            // Dropdown of unassigned document tags
            let unassigned: Vec<&str> = defined_tags
                .iter()
                .filter(|t| !op.tags.iter().any(|ot| ot == *t))
                .map(|t| t.as_str())
                .collect();
            if !unassigned.is_empty() {
                ui.menu_button(RichText::new("+ tag").small(), |ui| {
                    for name in &unassigned {
                        if ui.button(*name).clicked() {
                            add_tag = Some(name.to_string());
                            ui.close_menu();
                        }
                    }
                });
            } else if defined_tags.is_empty() {
                ui.label(RichText::new("(define tags in the Tags section)").weak().small());
            }
        });
        ui.end_row();
        if let Some(i) = remove_idx { op.tags.remove(i); c = true; }
        if let Some(name) = add_tag { op.tags.push(name); c = true; }

        c |= row_opt_bool(ui, "Deprecated:", &mut op.deprecated);
        c
    });

    section_header(ui, "Description");
    ch |= {
        let mut s = op.description.clone().unwrap_or_default();
        let r = ui
            .add(egui::TextEdit::multiline(&mut s).desired_rows(4).desired_width(f32::INFINITY))
            .changed();
        if r {
            op.description = if s.is_empty() { None } else { Some(s) };
        }
        r
    };

    // Security
    section_header(ui, "Security");
    ch |= edit_operation_security(ui, op, &security_scheme_names, &global_security);

    // Parameters
    section_header(ui, "Parameters");
    let mut to_remove_param: Option<usize> = None;
    for (i, p) in op.parameters.iter_mut().enumerate() {
        match p {
            RefOr::Ref(r) => {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("$ref: {}", r.ref_)).weak());
                    if ui.small_button("🗑").clicked() {
                        to_remove_param = Some(i);
                    }
                });
            }
            RefOr::Item(param) => {
                egui::CollapsingHeader::new(format!("  {} ({})", param.name, param.in_))
                    .id_salt(format!("op_param_{i}"))
                    .show(ui, |ui| {
                        ch |= edit_parameter_inline(ui, param, i);
                        if ui.small_button("🗑 Remove").clicked() {
                            to_remove_param = Some(i);
                        }
                    });
            }
        }
    }
    if let Some(idx) = to_remove_param {
        op.parameters.remove(idx);
        ch = true;
    }

    ui.horizontal(|ui| {
        if ui.small_button("+ Add Parameter").clicked() {
            op.parameters.push(RefOr::Item(Parameter {
                in_: "query".to_string(),
                ..Default::default()
            }));
            ch = true;
        }
        ui.add(
            egui::TextEdit::singleline(&mut new_item.parameter_name)
                .hint_text("$ref path…")
                .desired_width(160.0),
        );
        // Dropdown picker for existing component parameters
        let param_refs: Vec<String> = ui.data(|d| d.get_temp(egui::Id::new("oa_param_refs")).unwrap_or_default());
        if !param_refs.is_empty() {
            let cur = new_item.parameter_name.clone();
            let sel_name = param_refs.iter().find(|p| **p == cur)
                .and_then(|p| p.split('/').last()).unwrap_or("pick…");
            egui::ComboBox::from_id_salt("op_param_ref_pick")
                .selected_text(sel_name)
                .show_ui(ui, |ui| {
                    for ref_path in &param_refs {
                        let name = ref_path.split('/').last().unwrap_or(ref_path.as_str());
                        if ui.selectable_label(cur == *ref_path, name).clicked() {
                            new_item.parameter_name = ref_path.clone();
                        }
                    }
                });
        }
        if ui.small_button("+ Add $ref").clicked() && !new_item.parameter_name.is_empty() {
            let r = new_item.parameter_name.clone();
            new_item.parameter_name.clear();
            op.parameters.push(RefOr::Ref(Ref { ref_: r, ..Default::default() }));
            ch = true;
        }
    });

    // Request Body
    section_header(ui, "Request Body");
    match op.request_body.as_mut() {
        None => {
            if ui.button("+ Add Request Body").clicked() {
                op.request_body = Some(RefOr::Item(RequestBody::default()));
                ch = true;
            }
        }
        Some(rb) => {
            ch |= edit_request_body_ref_or(ui, rb, "op_rb");
            if ui.small_button("🗑 Remove Request Body").clicked() {
                op.request_body = None;
                ch = true;
            }
        }
    }

    // Responses
    section_header(ui, "Responses");
    let resp_codes: Vec<String> = op.responses.keys().cloned().collect();
    let mut to_remove_resp: Option<String> = None;

    for code in &resp_codes {
        if let Some(resp_ref) = op.responses.get_mut(code) {
            egui::CollapsingHeader::new(format!("  {code}"))
                .id_salt(format!("op_resp_{code}"))
                .show(ui, |ui| {
                    ch |= edit_response_ref_or(ui, resp_ref, &format!("op_resp_{code}_inner"));
                    if ui.small_button("🗑 Remove").clicked() {
                        to_remove_resp = Some(code.clone());
                    }
                });
        }
    }
    if let Some(code) = to_remove_resp {
        op.responses.shift_remove(&code);
        ch = true;
    }

    ui.menu_button("+ Add Response", |ui| {
        for &(code, desc) in &[
            ("200", "OK"),
            ("201", "Created"),
            ("202", "Accepted"),
            ("204", "No Content"),
            ("301", "Moved Permanently"),
            ("302", "Found"),
            ("304", "Not Modified"),
            ("400", "Bad Request"),
            ("401", "Unauthorized"),
            ("403", "Forbidden"),
            ("404", "Not Found"),
            ("405", "Method Not Allowed"),
            ("409", "Conflict"),
            ("410", "Gone"),
            ("422", "Unprocessable Entity"),
            ("429", "Too Many Requests"),
            ("500", "Internal Server Error"),
            ("502", "Bad Gateway"),
            ("503", "Service Unavailable"),
            ("default", "Default"),
        ] {
            let already = op.responses.contains_key(code);
            ui.add_enabled_ui(!already, |ui| {
                if ui.button(format!("{code}  {desc}")).clicked() {
                    op.responses.insert(code.to_string(), RefOr::Item(Response {
                        description: desc.to_string(),
                        ..Default::default()
                    }));
                    ch = true;
                    ui.close_menu();
                }
            });
        }
    });

    ch
}

// ── Operation security editor ─────────────────────────────────────────────────

fn edit_operation_security(
    ui: &mut Ui,
    op: &mut Operation,
    available_schemes: &[String],
    global_security: &[IndexMap<String, Vec<String>>],
) -> bool {
    use indexmap::IndexMap;
    let mut ch = false;

    let state: usize = match &op.security {
        None                        => 0,
        Some(v) if v.is_empty()     => 1,
        Some(_)                     => 2,
    };

    // ── State selector ────────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        if ui.radio(state == 0, "Inherit global").on_hover_text(
            "Use the spec-level security declaration. Most operations should use this."
        ).clicked() && state != 0 {
            op.security = None;
            ch = true;
        }
        if ui.radio(state == 1, "Disabled").on_hover_text(
            "Explicitly require no authentication for this operation (sets security: [])."
        ).clicked() && state != 1 {
            op.security = Some(vec![]);
            ch = true;
        }
        if ui.radio(state == 2, "Custom").on_hover_text(
            "Override security for this operation with specific schemes."
        ).clicked() && state != 2 {
            let init_req = if !available_schemes.is_empty() {
                let mut m = IndexMap::new();
                m.insert(available_schemes[0].clone(), vec![]);
                m
            } else {
                IndexMap::new()
            };
            op.security = Some(vec![init_req]);
            ch = true;
        }
    });

    // ── State-specific UI ─────────────────────────────────────────────────────
    match state {
        0 => {
            // Show what global security resolves to.
            if global_security.is_empty() {
                ui.label(
                    RichText::new("  ↳ No global security defined — this operation is open.")
                        .weak().small().italics(),
                );
            } else {
                let preview: Vec<String> = global_security.iter().map(|req| {
                    req.keys().cloned().collect::<Vec<_>>().join(" + ")
                }).collect();
                ui.label(
                    RichText::new(format!("  ↳ Global: {}", preview.join("  |  ")))
                        .weak().small(),
                );
            }
        }

        1 => {
            ui.label(
                RichText::new("  ↳ No authentication required for this operation.")
                    .weak().small().italics(),
            );
        }

        _ => {
            // Custom requirements editor.
            if available_schemes.is_empty() {
                ui.label(
                    RichText::new("  No security schemes defined in Components → Security Schemes.")
                        .color(egui::Color32::from_rgb(220, 160, 60)).small(),
                );
            }

            if let Some(requirements) = op.security.as_mut() {
                let mut req_to_remove: Option<usize> = None;

                let req_count = requirements.len();
                for (req_idx, requirement) in requirements.iter_mut().enumerate() {
                    ui.push_id(req_idx, |ui| {
                        ui.add_space(4.0);

                        // Row label for multiple OR options.
                        let row_label = if req_count > 1 {
                            format!("Option {}  (OR):", req_idx + 1)
                        } else {
                            "Schemes:".to_owned()
                        };
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new(&row_label).small().weak());

                            // Scheme chips — click to remove.
                            let scheme_names: Vec<String> = requirement.keys().cloned().collect();
                            let mut to_remove_scheme: Option<String> = None;

                            for scheme_name in &scheme_names {
                                if ui.selectable_label(
                                    true,
                                    RichText::new(format!("{scheme_name}  ×"))
                                        .small()
                                        .color(egui::Color32::from_rgb(80, 140, 220)),
                                ).on_hover_text("Click to remove").clicked() {
                                    to_remove_scheme = Some(scheme_name.clone());
                                    ch = true;
                                }
                            }
                            if let Some(name) = to_remove_scheme {
                                requirement.shift_remove(&name);
                            }

                            // Dropdown to add a scheme not yet in this requirement.
                            let unused: Vec<&str> = available_schemes.iter()
                                .filter(|s| !requirement.contains_key(s.as_str()))
                                .map(|s| s.as_str())
                                .collect();
                            if !unused.is_empty() {
                                ui.menu_button(RichText::new("+ scheme").small(), |ui| {
                                    for name in &unused {
                                        if ui.button(*name).clicked() {
                                            requirement.insert(name.to_string(), vec![]);
                                            ch = true;
                                            ui.close_menu();
                                        }
                                    }
                                });
                            }

                            if ui.small_button("🗑").on_hover_text("Remove this option").clicked() {
                                req_to_remove = Some(req_idx);
                                ch = true;
                            }
                        });

                        // Scope editor — one compact line per scheme.
                        for (scheme_name, scopes) in requirement.iter_mut() {
                            ui.horizontal(|ui| {
                                ui.add_space(16.0);
                                ui.label(
                                    RichText::new(format!("{scheme_name} scopes:"))
                                        .small().weak(),
                                );
                                let mut scope_str = scopes.join(", ");
                                let r = ui.add(
                                    egui::TextEdit::singleline(&mut scope_str)
                                        .hint_text("scope1, scope2 … (leave empty if unused)")
                                        .desired_width(230.0),
                                );
                                if r.changed() {
                                    *scopes = scope_str
                                        .split(',')
                                        .map(|s| s.trim().to_owned())
                                        .filter(|s| !s.is_empty())
                                        .collect();
                                    ch = true;
                                }
                            });
                        }
                    });
                }

                if let Some(idx) = req_to_remove {
                    requirements.remove(idx);
                }

                // Add another OR option.
                ui.add_space(2.0);
                if ui.small_button("+ Add OR option").on_hover_text(
                    "Add an alternative set of schemes (any one option is sufficient)."
                ).clicked() {
                    let new_req = if !available_schemes.is_empty() {
                        let mut m = IndexMap::new();
                        m.insert(available_schemes[0].clone(), vec![]);
                        m
                    } else {
                        IndexMap::new()
                    };
                    requirements.push(new_req);
                    ch = true;
                }
            }
        }
    }

    ch
}

fn edit_parameter_inline(ui: &mut Ui, param: &mut Parameter, idx: usize) -> bool {
    form_grid(ui, &format!("param_inner_{idx}"), |ui| {
        let mut c = false;
        c |= row_str(ui, "Name:", &mut param.name);

        // `in` dropdown
        ui.label("In:");
        egui::ComboBox::from_id_salt(format!("param_in_{idx}"))
            .selected_text(param.in_.as_str())
            .show_ui(ui, |ui| {
                for loc in ["query", "header", "path", "cookie"] {
                    if ui.selectable_label(param.in_ == loc, loc).clicked() {
                        param.in_ = loc.to_string();
                        c = true;
                    }
                }
            });
        ui.end_row();

        c |= row_opt_str(ui, "Description:", &mut param.description);
        c |= row_opt_bool(ui, "Required:", &mut param.required);
        c |= row_opt_bool(ui, "Deprecated:", &mut param.deprecated);
        c
    })
}

// ── Request Body editor ───────────────────────────────────────────────────────

fn edit_request_body_ref_or(ui: &mut Ui, rb: &mut RefOr<RequestBody>, id: &str) -> bool {
    match rb {
        RefOr::Ref(r) => {
            ui.label(format!("$ref: {}", r.ref_));
            false
        }
        RefOr::Item(body) => edit_request_body_item(ui, body, id),
    }
}

fn edit_request_body_item(ui: &mut Ui, body: &mut RequestBody, id: &str) -> bool {
    let mut ch = false;
    ch |= form_grid(ui, &format!("{id}_grid"), |ui| {
        let mut c = false;
        c |= row_opt_multiline(ui, "Description:", &mut body.description);
        c |= row_opt_bool(ui, "Required:", &mut body.required);
        c
    });

    ui.label(RichText::new("Content Types:").strong());
    let ct_keys: Vec<String> = body.content.keys().cloned().collect();
    let mut to_remove: Option<String> = None;

    for ct in &ct_keys {
        if let Some(media) = body.content.get_mut(ct) {
            egui::CollapsingHeader::new(format!("  {ct}"))
                .id_salt(format!("{id}_ct_{ct}"))
                .show(ui, |ui| {
                    ch |= edit_media_type(ui, media, &format!("{id}_ct_{ct}_mt"));
                    if ui.small_button("🗑 Remove").clicked() {
                        to_remove = Some(ct.clone());
                    }
                });
        }
    }
    if let Some(k) = to_remove {
        body.content.shift_remove(&k);
        ch = true;
    }

    ui.menu_button("+ Add Content Type", |ui| {
        let json_already = body.content.contains_key("application/json");
        ui.add_enabled_ui(!json_already, |ui| {
            ui.menu_button("application/json", |ui| {
                if ui.button("Inline Object").clicked() {
                    let mut media = MediaType::default();
                    let mut schema = Schema::default();
                    schema.set_type_str("object");
                    media.schema = Some(RefOr::Item(schema));
                    body.content.insert("application/json".to_string(), media);
                    ch = true;
                    ui.close_menu();
                }
                if ui.button("Schema Reference").clicked() {
                    let mut media = MediaType::default();
                    let mut schema = Schema::default();
                    schema.ref_ = Some(String::new());
                    media.schema = Some(RefOr::Item(schema));
                    body.content.insert("application/json".to_string(), media);
                    ch = true;
                    ui.close_menu();
                }
            });
        });
        for &ct in &["application/x-www-form-urlencoded", "multipart/form-data"] {
            let already = body.content.contains_key(ct);
            ui.add_enabled_ui(!already, |ui| {
                if ui.button(ct).clicked() {
                    let mut media = MediaType::default();
                    let mut schema = Schema::default();
                    schema.set_type_str("object");
                    media.schema = Some(RefOr::Item(schema));
                    body.content.insert(ct.to_string(), media);
                    ch = true;
                    ui.close_menu();
                }
            });
        }
    });

    ch
}

fn edit_media_type(ui: &mut Ui, media: &mut MediaType, id: &str) -> bool {
    let mut ch = false;

    // ── Schema ────────────────────────────────────────────────────────────────
    ui.label("Schema:");
    match &media.schema {
        None => {
            if ui.small_button("+ Set Schema").clicked() {
                media.schema = Some(RefOr::Item(Schema::default()));
                ch = true;
            }
        }
        Some(s) => {
            let s_str = s.ref_str().map(|r| format!("$ref: {r}"))
                .or_else(|| s.as_item().filter(|sc| !sc.type_str().is_empty()).map(|sc| sc.type_str().to_string()))
                .unwrap_or_else(|| "(inline schema)".to_string());
            ui.label(s_str);
            if let Some(schema_ref) = media.schema.as_mut() {
                if let Some(schema) = schema_ref.as_item_mut() {
                    ch |= edit_schema_inline(ui, schema, id, 0);
                }
            }
        }
    }

    // ── Examples ──────────────────────────────────────────────────────────────
    let example_refs: Vec<String> = ui.data(|d| {
        d.get_temp::<Vec<String>>(egui::Id::new("oa_example_refs")).unwrap_or_default()
    });

    ui.separator();
    ui.label(RichText::new("Examples:").strong());

    let ex_keys: Vec<String> = media.examples.keys().cloned().collect();
    let mut to_remove_ex: Option<String> = None;
    for ex_key in &ex_keys {
        ui.horizontal(|ui| {
            if ui.small_button("🗑").clicked() { to_remove_ex = Some(ex_key.clone()); }
            ui.label(RichText::new(ex_key).monospace());
            if let Some(ex_val) = media.examples.get(ex_key) {
                match ex_val {
                    RefOr::Ref(r) => {
                        let short = r.ref_.split('/').last().unwrap_or(&r.ref_);
                        ui.label(RichText::new(format!("→ {short}")).weak());
                    }
                    RefOr::Item(_) => { ui.label(RichText::new("(inline)").weak()); }
                }
            }
        });
    }
    if let Some(k) = to_remove_ex { media.examples.shift_remove(&k); ch = true; }

    // Add a reference to a component example
    if !example_refs.is_empty() {
        ui.horizontal(|ui| {
            let key_id = egui::Id::new(format!("{id}_ex_key"));
            let ref_id = egui::Id::new(format!("{id}_ex_ref"));
            let mut key: String = ui.data(|d| d.get_temp(key_id).unwrap_or_default());
            let mut ref_sel: String = ui.data(|d| d.get_temp(ref_id).unwrap_or_default());

            ui.add(egui::TextEdit::singleline(&mut key).hint_text("label").desired_width(90.0));
            ui.data_mut(|d| d.insert_temp(key_id, key.clone()));

            let short_sel = if ref_sel.is_empty() { "pick…" } else { ref_sel.split('/').last().unwrap_or("pick…") };
            egui::ComboBox::from_id_salt(egui::Id::new(format!("{id}_ex_combo")))
                .selected_text(short_sel)
                .width(140.0)
                .show_ui(ui, |ui| {
                    for rp in &example_refs {
                        let short = rp.split('/').last().unwrap_or(rp.as_str());
                        if ui.selectable_label(ref_sel == *rp, short).clicked() { ref_sel = rp.clone(); }
                    }
                });
            ui.data_mut(|d| d.insert_temp(ref_id, ref_sel.clone()));

            if ui.small_button("+ Link").clicked() && !key.is_empty() && !ref_sel.is_empty() {
                media.examples.insert(key.clone(), RefOr::Ref(Ref { ref_: ref_sel.clone(), summary: None, description: None }));
                ui.data_mut(|d| { d.insert_temp(key_id, String::new()); d.insert_temp(ref_id, String::new()); });
                ch = true;
            }
        });
    }

    ch
}

// ── Response editor ───────────────────────────────────────────────────────────

fn edit_response_ref_or(ui: &mut Ui, resp: &mut RefOr<Response>, id: &str) -> bool {
    match resp {
        RefOr::Ref(r) => {
            ui.label(format!("$ref: {}", r.ref_));
            false
        }
        RefOr::Item(r) => edit_response_item(ui, r, id),
    }
}

fn edit_response_item(ui: &mut Ui, resp: &mut Response, id: &str) -> bool {
    let mut ch = false;
    ch |= form_grid(ui, &format!("{id}_grid"), |ui| {
        row_str(ui, "Description:", &mut resp.description)
    });

    ui.label(RichText::new("Content Types:").strong());
    let ct_keys: Vec<String> = resp.content.keys().cloned().collect();
    let mut to_remove: Option<String> = None;
    for ct in &ct_keys {
        if let Some(media) = resp.content.get_mut(ct) {
            egui::CollapsingHeader::new(format!("  {ct}"))
                .id_salt(format!("{id}_ct_{ct}"))
                .show(ui, |ui| {
                    ch |= edit_media_type(ui, media, &format!("{id}_ct_{ct}_mt"));
                    if ui.small_button("🗑 Remove").clicked() {
                        to_remove = Some(ct.clone());
                    }
                });
        }
    }
    if let Some(k) = to_remove {
        resp.content.shift_remove(&k);
        ch = true;
    }

    ui.menu_button("+ Add Content Type", |ui| {
        for &(ct, init_object) in &[
            ("application/json",       true),
            ("text/plain",             false),
            ("text/html",              false),
            ("application/xml",        false),
            ("application/octet-stream", false),
            ("application/pdf",        false),
        ] {
            let already = resp.content.contains_key(ct);
            ui.add_enabled_ui(!already, |ui| {
                if ui.button(ct).clicked() {
                    let mut media = MediaType::default();
                    if init_object {
                        let mut schema = Schema::default();
                        schema.set_type_str("object");
                        media.schema = Some(RefOr::Item(schema));
                    }
                    resp.content.insert(ct.to_string(), media);
                    ch = true;
                    ui.close_menu();
                }
            });
        }
    });

    ch
}

// ── Schema editors ────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
enum SchemaMode { Regular, Composition }

#[derive(Clone, Copy, PartialEq, Eq)]
enum CompKind { AllOf, AnyOf, OneOf }

impl CompKind {
    fn label(self) -> &'static str {
        match self { Self::AllOf => "allOf", Self::AnyOf => "anyOf", Self::OneOf => "oneOf" }
    }
}

fn edit_schema_by_name(ui: &mut Ui, spec: &mut OpenApiSpec, name: &str) -> bool {
    ui.heading(format!("Schema: {name}"));
    ui.add_space(4.0);
    let Some(comps) = spec.components.as_mut() else { ui.label("No components."); return false };
    let Some(schema_ref) = comps.schemas.get_mut(name) else { ui.label("Schema not found."); return false };
    match schema_ref {
        RefOr::Ref(r) => { ui.label(format!("$ref: {}", r.ref_)); false }
        RefOr::Item(schema) => edit_schema_inline(ui, schema, name, 0),
    }
}

/// Edit a schema inline. `depth` limits nesting depth for nested properties.
pub fn edit_schema_inline(ui: &mut Ui, schema: &mut Schema, id: &str, depth: u32) -> bool {
    let mut ch = false;

    // ── $ref shortcut ─────────────────────────────────────────────────────────
    if schema.ref_.is_some() {
        ui.horizontal(|ui| {
            ui.label("$ref:");
            ch |= ref_picker(
                ui, &mut schema.ref_,
                egui::Id::new(format!("{id}__ref_pick")),
                egui::Id::new("oa_schema_refs"),
            );
            if ui.small_button("✕").clicked() { schema.ref_ = None; ch = true; }
        });
        return ch;
    }

    // ── Common fields (title + description, visible in both modes) ─────────────
    ch |= form_grid(ui, &format!("{id}__common"), |ui| {
        let mut c = false;
        c |= row_opt_str(ui, "Title:", &mut schema.title);
        c |= row_opt_multiline(ui, "Description:", &mut schema.description);
        c
    });

    // ── Mode selector ─────────────────────────────────────────────────────────
    let has_comp = !schema.all_of.is_empty() || !schema.any_of.is_empty() || !schema.one_of.is_empty();
    let mode_id = egui::Id::new(format!("{id}__mode"));
    let mut mode: SchemaMode = ui.data(|d| {
        d.get_temp(mode_id)
            .unwrap_or(if has_comp { SchemaMode::Composition } else { SchemaMode::Regular })
    });

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new("Model type:").strong());
        if ui.selectable_label(mode == SchemaMode::Regular, "  Regular  ").clicked() {
            mode = SchemaMode::Regular;
        }
        if ui.selectable_label(mode == SchemaMode::Composition, "  Composition  ").clicked() {
            mode = SchemaMode::Composition;
        }
    });
    ui.data_mut(|d| d.insert_temp(mode_id, mode));
    ui.separator();

    match mode {
        SchemaMode::Regular     => ch |= edit_schema_regular(ui, schema, id, depth),
        SchemaMode::Composition => ch |= edit_schema_composition(ui, schema, id, depth),
    }
    ch
}

// ── Regular model ─────────────────────────────────────────────────────────────

fn edit_schema_regular(ui: &mut Ui, schema: &mut Schema, id: &str, depth: u32) -> bool {
    let mut ch = false;

    // Type + flags
    ch |= form_grid(ui, &format!("{id}__base"), |ui| {
        let mut c = false;
        let mut type_str = schema.type_str().to_string();
        ui.label("Type:");
        egui::ComboBox::from_id_salt(format!("{id}__type"))
            .selected_text(type_str.as_str())
            .show_ui(ui, |ui| {
                for t in ["", "string", "number", "integer", "boolean", "array", "object", "null"] {
                    if ui.selectable_label(type_str == t, t).clicked() { type_str = t.to_string(); c = true; }
                }
            });
        ui.end_row();
        if c { schema.set_type_str(&type_str); }
        ui.label("Format:");
        let cur_fmt = schema.format.clone().unwrap_or_default();
        egui::ComboBox::from_id_salt(format!("{id}__fmt"))
            .selected_text(cur_fmt.as_str())
            .show_ui(ui, |ui| {
                for &opt in format_options(&type_str) {
                    if ui.selectable_label(cur_fmt == opt, opt).clicked() {
                        schema.format = if opt.is_empty() { None } else { Some(opt.to_string()) };
                        c = true;
                    }
                }
            });
        ui.end_row();
        c |= row_opt_bool(ui, "Nullable (3.0):", &mut schema.nullable);
        c |= row_opt_bool(ui, "Read Only:", &mut schema.read_only);
        c |= row_opt_bool(ui, "Write Only:", &mut schema.write_only);
        c |= row_opt_bool(ui, "Deprecated:", &mut schema.deprecated);
        c
    });

    let type_str = schema.type_str().to_string();

    // String constraints (always visible when relevant)
    if type_str == "string" || type_str.is_empty() {
        section_header(ui, "String Constraints");
        ch |= form_grid(ui, &format!("{id}__str"), |ui| {
            let mut c = false;
            c |= row_opt_u64(ui, "Min Length:", &mut schema.min_length);
            c |= row_opt_u64(ui, "Max Length:", &mut schema.max_length);
            c |= row_opt_str(ui, "Pattern:", &mut schema.pattern);
            c
        });
    }

    // Number constraints
    if type_str == "number" || type_str == "integer" {
        section_header(ui, "Number Constraints");
        ch |= form_grid(ui, &format!("{id}__num"), |ui| {
            let mut c = false;
            c |= row_opt_f64(ui, "Minimum:", &mut schema.minimum);
            c |= row_opt_f64(ui, "Maximum:", &mut schema.maximum);
            c |= row_opt_f64(ui, "Multiple Of:", &mut schema.multiple_of);
            c
        });
    }

    // Array
    if type_str == "array" {
        section_header(ui, "Array");
        ch |= form_grid(ui, &format!("{id}__arr"), |ui| {
            let mut c = false;
            c |= row_opt_u64(ui, "Min Items:", &mut schema.min_items);
            c |= row_opt_u64(ui, "Max Items:", &mut schema.max_items);
            c |= row_opt_bool(ui, "Unique Items:", &mut schema.unique_items);
            c
        });
        if depth < 3 {
            ui.label(RichText::new("Items schema:").strong());
            ui.indent(format!("{id}__items_indent"), |ui| {
                let items = schema.items.get_or_insert_with(|| Box::new(Schema::default()));
                ch |= edit_schema_inline(ui, items, &format!("{id}__items"), depth + 1);
            });
        }
    }

    // Properties (always fully expanded — no collapsing header)
    if type_str == "object" || !schema.properties.is_empty() {
        section_header(ui, "Properties");
        ch |= edit_schema_properties_flat(ui, schema, id, depth);
    }

    // Enum values
    section_header(ui, "Enum Values");
    let mut enum_str = schema.enum_.as_deref()
        .map(|vals| vals.iter().map(|v| serde_json::to_string(v).unwrap_or_default()).collect::<Vec<_>>().join("\n"))
        .unwrap_or_default();
    ui.label(RichText::new("One JSON value per line (blank = no enum restriction):").weak().small());
    if ui.add(egui::TextEdit::multiline(&mut enum_str).desired_rows(3).desired_width(f32::INFINITY)).changed() {
        let vals: Vec<Value> = enum_str.lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect();
        schema.enum_ = if vals.is_empty() { None } else { Some(vals) };
        ch = true;
    }

    ch
}

// ── Flat property list (no collapsing headers per property) ───────────────────

fn edit_schema_properties_flat(ui: &mut Ui, schema: &mut Schema, id: &str, depth: u32) -> bool {
    let mut ch = false;

    ch |= form_grid(ui, &format!("{id}__obj_meta"), |ui| {
        let mut c = false;
        c |= row_opt_u64(ui, "Min Properties:", &mut schema.min_properties);
        c |= row_opt_u64(ui, "Max Properties:", &mut schema.max_properties);
        c
    });

    let prop_keys: Vec<String> = schema.properties.keys().cloned().collect();
    let mut to_remove: Option<String> = None;
    let mut rename_op: Option<(String, String)> = None;
    let mut reorder_op: Option<(usize, usize)> = None;
    let is_dragging = egui::DragAndDrop::has_any_payload(ui.ctx());

    // Helper closure: render a drop-zone separator between cards.
    let drop_line = |ui: &mut Ui, target_idx: usize, reorder_op: &mut Option<(usize, usize)>| {
        let (dz, payload) = ui.dnd_drop_zone::<usize, ()>(egui::Frame::none(), |ui| {
            ui.set_min_size(egui::vec2(ui.available_width(), if is_dragging { 8.0 } else { 0.0 }));
        });
        if is_dragging {
            let color = if dz.response.contains_pointer() {
                ui.visuals().selection.bg_fill
            } else {
                egui::Color32::TRANSPARENT
            };
            ui.painter().hline(
                dz.response.rect.x_range(),
                dz.response.rect.center().y,
                egui::Stroke::new(2.0, color),
            );
        }
        if let Some(from) = payload {
            *reorder_op = Some((*from, target_idx));
        }
    };

    for (idx, prop_name) in prop_keys.iter().enumerate() {
        let is_required = schema.required.contains(prop_name);
        let mut new_required = is_required;
        let mut do_remove = false;
        let mut rename_to: Option<String> = None;

        // Drop zone above this item
        drop_line(ui, idx, &mut reorder_op);

        if let Some(prop_schema) = schema.properties.get_mut(prop_name) {
            ui.group(|ui| {
                        // Row 1: name | type | Req toggle | Dep toggle | delete
                        ui.horizontal(|ui| {

                            let nbuf_id = egui::Id::new(format!("{id}__pname__{prop_name}"));
                            let mut nbuf: String = ui.data(|d| {
                                d.get_temp(nbuf_id).unwrap_or_else(|| prop_name.clone())
                            });
                            let nresp = ui.add(
                                egui::TextEdit::singleline(&mut nbuf)
                                    .desired_width(120.0)
                                    .hint_text("name"),
                            );
                            ui.data_mut(|d| d.insert_temp(nbuf_id, nbuf.clone()));
                            if nresp.lost_focus() && !nbuf.is_empty() && nbuf != *prop_name {
                                rename_to = Some(nbuf);
                            }

                            ui.label("Type:");
                            let mut type_str = if prop_schema.ref_.is_some() {
                                "$ref".to_string()
                            } else {
                                prop_schema.type_str().to_string()
                            };
                            let mut type_changed = false;
                            egui::ComboBox::from_id_salt(format!("{id}__ptype__{prop_name}"))
                                .selected_text(type_str.as_str())
                                .width(88.0)
                                .show_ui(ui, |ui| {
                                    for t in ["", "string", "number", "integer", "boolean", "array", "object", "null", "$ref"] {
                                        if ui.selectable_label(type_str == t, t).clicked() {
                                            type_str = t.to_string();
                                            type_changed = true;
                                        }
                                    }
                                });
                            if type_changed {
                                if type_str == "$ref" {
                                    prop_schema.set_type_str("");
                                    if prop_schema.ref_.is_none() {
                                        prop_schema.ref_ = Some(String::new());
                                    }
                                } else {
                                    prop_schema.ref_ = None;
                                    prop_schema.set_type_str(&type_str);
                                }
                                ch = true;
                            }

                            ui.label("Req:");
                            toggle_switch(ui, &mut new_required);

                            ui.label("Dep:");
                            let mut dep = prop_schema.deprecated.unwrap_or(false);
                            if toggle_switch(ui, &mut dep).changed() {
                                prop_schema.deprecated = Some(dep);
                                ch = true;
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("🗑").clicked() { do_remove = true; }
                            });
                        });

                        // Row 2: description
                        ui.horizontal(|ui| {
                            ui.label("Desc:");
                            let mut desc = prop_schema.description.clone().unwrap_or_default();
                            if ui.add(
                                egui::TextEdit::singleline(&mut desc)
                                    .desired_width(f32::INFINITY)
                                    .hint_text("description"),
                            ).changed() {
                                prop_schema.description = if desc.is_empty() { None } else { Some(desc) };
                                ch = true;
                            }
                        });

                        let ptype = prop_schema.type_str().to_string();

                        // $ref picker
                        if prop_schema.ref_.is_some() {
                            ui.horizontal(|ui| {
                                ui.label("$ref:");
                                ch |= ref_picker(
                                    ui, &mut prop_schema.ref_,
                                    egui::Id::new(format!("{id}__pref__{prop_name}")),
                                    egui::Id::new("oa_schema_refs"),
                                );
                            });
                        }

                        // String-specific fields
                        if ptype == "string" && prop_schema.ref_.is_none() {
                            ui.horizontal(|ui| {
                                ui.label("Format:");
                                let cur_fmt = prop_schema.format.clone().unwrap_or_default();
                                egui::ComboBox::from_id_salt(format!("{id}__pfmt__{prop_name}"))
                                    .selected_text(cur_fmt.as_str())
                                    .width(130.0)
                                    .show_ui(ui, |ui| {
                                        for &opt in format_options("string") {
                                            if ui.selectable_label(cur_fmt == opt, opt).clicked() {
                                                prop_schema.format = if opt.is_empty() { None } else { Some(opt.to_string()) };
                                                ch = true;
                                            }
                                        }
                                    });
                                ui.label("Pattern:");
                                let mut pat = prop_schema.pattern.clone().unwrap_or_default();
                                if ui.add(egui::TextEdit::singleline(&mut pat).desired_width(110.0)).changed() {
                                    prop_schema.pattern = if pat.is_empty() { None } else { Some(pat) };
                                    ch = true;
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("Default:");
                                if opt_json_field(ui, &mut prop_schema.default, 100.0, "value") { ch = true; }
                                ui.label("Example:");
                                if opt_json_field(ui, &mut prop_schema.example, 100.0, "value") { ch = true; }
                            });
                            ui.horizontal(|ui| {
                                ui.label("MinLen:");
                                let mut s = prop_schema.min_length.map(|v| v.to_string()).unwrap_or_default();
                                if ui.add(egui::TextEdit::singleline(&mut s).desired_width(50.0)).changed() {
                                    prop_schema.min_length = s.parse().ok();
                                    ch = true;
                                }
                                ui.label("MaxLen:");
                                let mut s = prop_schema.max_length.map(|v| v.to_string()).unwrap_or_default();
                                if ui.add(egui::TextEdit::singleline(&mut s).desired_width(50.0)).changed() {
                                    prop_schema.max_length = s.parse().ok();
                                    ch = true;
                                }
                            });
                            let has_enum = prop_schema.enum_.is_some();
                            egui::CollapsingHeader::new("Enum Values")
                                .id_salt(format!("{id}__penum__{prop_name}"))
                                .default_open(has_enum)
                                .show(ui, |ui| {
                                    let count = prop_schema.enum_.as_ref().map_or(0, |v| v.len());
                                    let mut remove_idx: Option<usize> = None;
                                    for i in 0..count {
                                        ui.horizontal(|ui| {
                                            if ui.small_button("🗑").clicked() {
                                                remove_idx = Some(i);
                                            }
                                            let mut s = {
                                                let val = &prop_schema.enum_.as_ref().unwrap()[i];
                                                match val {
                                                    serde_json::Value::String(sv) => sv.clone(),
                                                    other => serde_json::to_string(other).unwrap_or_default(),
                                                }
                                            };
                                            if ui.add(egui::TextEdit::singleline(&mut s).desired_width(f32::INFINITY)).changed() {
                                                prop_schema.enum_.as_mut().unwrap()[i] = serde_json::Value::String(s);
                                                ch = true;
                                            }
                                        });
                                    }
                                    if let Some(idx) = remove_idx {
                                        let vals = prop_schema.enum_.as_mut().unwrap();
                                        vals.remove(idx);
                                        if vals.is_empty() {
                                            prop_schema.enum_ = None;
                                        }
                                        ch = true;
                                    }
                                    if ui.small_button("+ Add").clicked() {
                                        prop_schema.enum_.get_or_insert_with(Vec::new).push(serde_json::Value::String(String::new()));
                                        ch = true;
                                    }
                                });
                        }

                        // Number/Integer-specific fields
                        if (ptype == "number" || ptype == "integer") && prop_schema.ref_.is_none() {
                            ui.horizontal(|ui| {
                                ui.label("Format:");
                                let cur_fmt = prop_schema.format.clone().unwrap_or_default();
                                egui::ComboBox::from_id_salt(format!("{id}__pfmt__{prop_name}"))
                                    .selected_text(cur_fmt.as_str())
                                    .width(80.0)
                                    .show_ui(ui, |ui| {
                                        for &opt in format_options(&ptype) {
                                            if ui.selectable_label(cur_fmt == opt, opt).clicked() {
                                                prop_schema.format = if opt.is_empty() { None } else { Some(opt.to_string()) };
                                                ch = true;
                                            }
                                        }
                                    });
                                ui.label("MultipleOf:");
                                let mut s = prop_schema.multiple_of.map(|v| v.to_string()).unwrap_or_default();
                                if ui.add(egui::TextEdit::singleline(&mut s).desired_width(60.0)).changed() {
                                    prop_schema.multiple_of = s.parse().ok();
                                    ch = true;
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("Default:");
                                if opt_json_field(ui, &mut prop_schema.default, 80.0, "value") { ch = true; }
                                ui.label("Example:");
                                if opt_json_field(ui, &mut prop_schema.example, 80.0, "value") { ch = true; }
                            });
                            ui.horizontal(|ui| {
                                ui.label("Min:");
                                let mut s = prop_schema.minimum.map(|v| v.to_string()).unwrap_or_default();
                                if ui.add(egui::TextEdit::singleline(&mut s).desired_width(55.0)).changed() {
                                    prop_schema.minimum = s.parse().ok();
                                    ch = true;
                                }
                                ui.label("Max:");
                                let mut s = prop_schema.maximum.map(|v| v.to_string()).unwrap_or_default();
                                if ui.add(egui::TextEdit::singleline(&mut s).desired_width(55.0)).changed() {
                                    prop_schema.maximum = s.parse().ok();
                                    ch = true;
                                }
                                ui.label("ExclMin:");
                                if excl_bound_field(ui, &mut prop_schema.exclusive_minimum, 55.0, "number") { ch = true; }
                                ui.label("ExclMax:");
                                if excl_bound_field(ui, &mut prop_schema.exclusive_maximum, 55.0, "number") { ch = true; }
                            });
                        }

                        // Boolean-specific fields
                        if ptype == "boolean" && prop_schema.ref_.is_none() {
                            ui.horizontal(|ui| {
                                ui.label("Default:");
                                let cur = match &prop_schema.default {
                                    Some(serde_json::Value::Bool(true))  => "true",
                                    Some(serde_json::Value::Bool(false)) => "false",
                                    _                                    => "",
                                };
                                egui::ComboBox::from_id_salt(format!("{id}__pbool_default__{prop_name}"))
                                    .selected_text(cur)
                                    .width(70.0)
                                    .show_ui(ui, |ui| {
                                        for &opt in &["", "true", "false"] {
                                            if ui.selectable_label(cur == opt, opt).clicked() {
                                                prop_schema.default = match opt {
                                                    "true"  => Some(serde_json::Value::Bool(true)),
                                                    "false" => Some(serde_json::Value::Bool(false)),
                                                    _       => None,
                                                };
                                                ch = true;
                                            }
                                        }
                                    });
                            });
                        }

                        // Array
                        if ptype == "array" && prop_schema.ref_.is_none() {
                            ui.horizontal(|ui| {
                                ui.label("Items:");
                                let items = prop_schema.items.get_or_insert_with(|| Box::new(Schema::default()));
                                ch |= ref_picker(
                                    ui, &mut items.ref_,
                                    egui::Id::new(format!("{id}__pitems_ref__{prop_name}")),
                                    egui::Id::new("oa_schema_refs"),
                                );
                            });
                            let items_has_ref = prop_schema.items.as_ref().map(|i| i.ref_.is_some()).unwrap_or(false);
                            if !items_has_ref {
                                ui.horizontal(|ui| {
                                    ui.label("Items type:");
                                    let items = prop_schema.items.get_or_insert_with(|| Box::new(Schema::default()));
                                    let mut it = items.type_str().to_string();
                                    let mut it_changed = false;
                                    egui::ComboBox::from_id_salt(format!("{id}__pitems_type__{prop_name}"))
                                        .selected_text(&it).width(100.0)
                                        .show_ui(ui, |ui| {
                                            for t in ["", "string", "number", "integer", "boolean", "object"] {
                                                if ui.selectable_label(it == t, t).clicked() { it = t.to_string(); it_changed = true; }
                                            }
                                        });
                                    if it_changed { items.set_type_str(&it); ch = true; }
                                });
                            }
                        }

                        // Nested object properties (depth-limited)
                        if ptype == "object" && depth < 2 && prop_schema.ref_.is_none() {
                            ui.separator();
                            ui.label(RichText::new("Nested properties:").small().strong());
                            ch |= edit_schema_properties_flat(
                                ui, prop_schema,
                                &format!("{id}__nested__{prop_name}"),
                                depth + 1,
                            );
                        }
                        // Drag handle — bottom-right of card
                        // horizontal() constrains height so it doesn't consume remaining group space
                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.dnd_drag_source(
                                    egui::Id::new(format!("{id}__drag__{prop_name}")),
                                    idx,
                                    |ui| {
                                        // Paint a 2×3 dot grid (braille chars aren't in the bundled font)
                                        let (rect, _) = ui.allocate_exact_size(
                                            egui::vec2(14.0, 16.0),
                                            egui::Sense::hover(),
                                        );
                                        if ui.is_rect_visible(rect) {
                                            let color = ui.visuals().weak_text_color();
                                            for row in 0..3u8 {
                                                for col in 0..2u8 {
                                                    ui.painter().circle_filled(
                                                        egui::pos2(
                                                            rect.left() + 3.0 + col as f32 * 7.0,
                                                            rect.top()  + 2.0 + row as f32 * 6.0,
                                                        ),
                                                        1.5,
                                                        color,
                                                    );
                                                }
                                            }
                                        }
                                    },
                                );
                            });
                        });
            }); // end group
            ui.add_space(2.0);
        }

        // Apply deferred mutations (schema borrow fully released above)
        if new_required != is_required {
            if new_required { if !schema.required.contains(prop_name) { schema.required.push(prop_name.clone()); } }
            else { schema.required.retain(|r| r != prop_name); }
            ch = true;
        }
        if do_remove { to_remove = Some(prop_name.clone()); }
        if let Some(nn) = rename_to {
            if !schema.properties.contains_key(&nn) { rename_op = Some((prop_name.clone(), nn)); }
        }
    }

    // Final drop zone below the last item
    drop_line(ui, prop_keys.len(), &mut reorder_op);

    if let Some(k) = to_remove {
        schema.properties.shift_remove(&k);
        schema.required.retain(|r| *r != k);
        ch = true;
    }
    if let Some((old, new_name)) = rename_op {
        if let Some(val) = schema.properties.shift_remove(&old) {
            schema.properties.insert(new_name.clone(), val);
            if let Some(pos) = schema.required.iter().position(|r| r == &old) {
                schema.required[pos] = new_name;
            }
            ch = true;
        }
    }
    if let Some((from, to)) = reorder_op {
        let n = schema.properties.len();
        if from < n && to <= n && from != to && from + 1 != to {
            let mut pairs: Vec<(String, Box<Schema>)> =
                std::mem::take(&mut schema.properties).into_iter().collect();
            let dragged = pairs.remove(from);
            let insert_at = if to > from { to - 1 } else { to };
            pairs.insert(insert_at, dragged);
            schema.properties = pairs.into_iter().collect();
            ch = true;
        }
    }

    // Add property row
    ui.horizontal(|ui| {
        let buf_id = egui::Id::new(format!("{id}__new_prop"));
        let mut buf: String = ui.data_mut(|d| d.get_temp(buf_id).unwrap_or_default());
        ui.add(egui::TextEdit::singleline(&mut buf).hint_text("new property name").desired_width(150.0));
        ui.data_mut(|d| d.insert_temp(buf_id, buf.clone()));
        if ui.button("+ Add Property").clicked() && !buf.is_empty() {
            schema.properties.insert(buf.clone(), Box::new(Schema::default()));
            ui.data_mut(|d| d.insert_temp(buf_id, String::new()));
            ch = true;
        }
    });

    ch
}

// ── Composition model ─────────────────────────────────────────────────────────

fn edit_schema_composition(ui: &mut Ui, schema: &mut Schema, id: &str, depth: u32) -> bool {
    let mut ch = false;

    // Discriminator (optional — only relevant for composition)
    {
        let disc = schema.discriminator.get_or_insert_with(Discriminator::default);
        let mut pname = disc.property_name.clone();
        ui.horizontal(|ui| {
            ui.label("Discriminator property name:");
            if ui.add(egui::TextEdit::singleline(&mut pname).hint_text("optional").desired_width(160.0)).changed() {
                disc.property_name = pname.clone();
                ch = true;
            }
        });
        if schema.discriminator.as_ref().map(|d| d.property_name.is_empty()).unwrap_or(false) {
            schema.discriminator = None;
        }
    }

    ui.add_space(6.0);

    // ── Composition type radio ─────────────────────────────────────────────────
    let comp_id = egui::Id::new(format!("{id}__comp_kind"));
    let inferred = if !schema.all_of.is_empty() { CompKind::AllOf }
        else if !schema.any_of.is_empty() { CompKind::AnyOf }
        else { CompKind::OneOf };
    let mut comp_kind: CompKind = ui.data(|d| d.get_temp(comp_id).unwrap_or(inferred));
    let prev_kind = comp_kind;

    ui.horizontal(|ui| {
        ui.label(RichText::new("Composition type:").strong());
        ui.radio_value(&mut comp_kind, CompKind::AllOf, "allOf");
        ui.radio_value(&mut comp_kind, CompKind::AnyOf, "anyOf");
        ui.radio_value(&mut comp_kind, CompKind::OneOf, "oneOf");
    });
    ui.data_mut(|d| d.insert_temp(comp_id, comp_kind));

    // Migrate entries when composition type changes
    if comp_kind != prev_kind {
        let entries = match prev_kind {
            CompKind::AllOf => std::mem::take(&mut schema.all_of),
            CompKind::AnyOf => std::mem::take(&mut schema.any_of),
            CompKind::OneOf => std::mem::take(&mut schema.one_of),
        };
        match comp_kind {
            CompKind::AllOf => schema.all_of.extend(entries),
            CompKind::AnyOf => schema.any_of.extend(entries),
            CompKind::OneOf => schema.one_of.extend(entries),
        }
        ch = true;
    }

    ui.separator();
    ui.label(RichText::new(format!("Entries — {}:", comp_kind.label())).strong());
    ui.add_space(4.0);

    // ── Entry list ────────────────────────────────────────────────────────────
    // Determine which list is active, iterate by index to avoid borrow conflicts.
    let list_len = match comp_kind {
        CompKind::AllOf => schema.all_of.len(),
        CompKind::AnyOf => schema.any_of.len(),
        CompKind::OneOf => schema.one_of.len(),
    };

    let mut to_remove: Option<usize> = None;

    for i in 0..list_len {
        let entry = match comp_kind {
            CompKind::AllOf => &mut schema.all_of[i],
            CompKind::AnyOf => &mut schema.any_of[i],
            CompKind::OneOf => &mut schema.one_of[i],
        };
        let entry_label = entry.ref_.as_deref()
            .map(|r| format!("$ref: {r}"))
            .or_else(|| { let t = entry.type_str(); if t.is_empty() { None } else { Some(t.to_string()) } })
            .unwrap_or_else(|| format!("Entry {}", i + 1));

        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("{}. {}", i + 1, entry_label)).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("🗑").clicked() { to_remove = Some(i); }
                });
            });
            ch |= edit_composition_entry(ui, entry, &format!("{id}__centry_{i}"), depth);
        });
        ui.add_space(2.0);
    }

    if let Some(idx) = to_remove {
        match comp_kind {
            CompKind::AllOf => schema.all_of.remove(idx),
            CompKind::AnyOf => schema.any_of.remove(idx),
            CompKind::OneOf => schema.one_of.remove(idx),
        };
        ch = true;
    }

    if ui.button(format!("+ Add {} entry", comp_kind.label())).clicked() {
        match comp_kind {
            CompKind::AllOf => schema.all_of.push(Box::new(Schema::default())),
            CompKind::AnyOf => schema.any_of.push(Box::new(Schema::default())),
            CompKind::OneOf => schema.one_of.push(Box::new(Schema::default())),
        }
        ch = true;
    }

    ch
}

fn edit_composition_entry(ui: &mut Ui, schema: &mut Schema, id: &str, depth: u32) -> bool {
    let mut ch = false;
    let is_ref = schema.ref_.is_some();

    // Inline / $ref toggle
    ui.horizontal(|ui| {
        if ui.selectable_label(!is_ref, "Inline").clicked() && is_ref {
            schema.ref_ = None;
            ch = true;
        }
        if ui.selectable_label(is_ref, "$ref").clicked() && !is_ref {
            schema.ref_ = Some(String::new());
            ch = true;
        }
    });

    if is_ref {
        ui.horizontal(|ui| {
            ui.label("$ref:");
            ch |= ref_picker(
                ui, &mut schema.ref_,
                egui::Id::new(format!("{id}__ref_pick")),
                egui::Id::new("oa_schema_refs"),
            );
        });
    } else if depth < 3 {
        ch |= form_grid(ui, &format!("{id}__inline"), |ui| {
            let mut c = false;
            c |= row_opt_str(ui, "Title:", &mut schema.title);
            c |= row_opt_str(ui, "Description:", &mut schema.description);
            let mut type_str = schema.type_str().to_string();
            ui.label("Type:");
            egui::ComboBox::from_id_salt(format!("{id}__type"))
                .selected_text(type_str.as_str())
                .show_ui(ui, |ui| {
                    for t in ["", "string", "number", "integer", "boolean", "array", "object", "null"] {
                        if ui.selectable_label(type_str == t, t).clicked() { type_str = t.to_string(); c = true; }
                    }
                });
            ui.end_row();
            if c { schema.set_type_str(&type_str); }
            c
        });
        if schema.type_str() == "object" || !schema.properties.is_empty() {
            ui.label(RichText::new("Properties:").strong());
            ch |= edit_schema_properties_flat(ui, schema, &format!("{id}__iprops"), depth + 1);
        }
    }

    ch
}

// ── Component-level editors ───────────────────────────────────────────────────

fn edit_request_body_by_name(ui: &mut Ui, spec: &mut OpenApiSpec, name: &str) -> bool {
    ui.heading(format!("Request Body: {name}"));
    ui.add_space(4.0);
    let Some(comps) = spec.components.as_mut() else { return false };
    let Some(rb) = comps.request_bodies.get_mut(name) else { ui.label("Not found."); return false };
    edit_request_body_ref_or(ui, rb, name)
}

fn edit_component_response_by_name(ui: &mut Ui, spec: &mut OpenApiSpec, name: &str) -> bool {
    ui.heading(format!("Response: {name}"));
    ui.add_space(4.0);
    let Some(comps) = spec.components.as_mut() else { return false };
    let Some(resp) = comps.responses.get_mut(name) else { ui.label("Not found."); return false };
    edit_response_ref_or(ui, resp, name)
}

fn edit_component_parameter_by_name(ui: &mut Ui, spec: &mut OpenApiSpec, name: &str) -> bool {
    ui.heading(format!("Parameter: {name}"));
    ui.add_space(4.0);
    let Some(comps) = spec.components.as_mut() else { return false };
    let Some(p) = comps.parameters.get_mut(name) else { ui.label("Not found."); return false };
    match p {
        RefOr::Ref(r) => { ui.label(format!("$ref: {}", r.ref_)); false }
        RefOr::Item(param) => {
            form_grid(ui, &format!("cparam_{name}_grid"), |ui| {
                let mut c = false;
                c |= row_str(ui, "Name:", &mut param.name);
                ui.label("In:");
                egui::ComboBox::from_id_salt(format!("cparam_{name}_in"))
                    .selected_text(param.in_.as_str())
                    .show_ui(ui, |ui| {
                        for loc in ["query","header","path","cookie"] {
                            if ui.selectable_label(param.in_ == loc, loc).clicked() {
                                param.in_ = loc.to_string();
                                c = true;
                            }
                        }
                    });
                ui.end_row();
                c |= row_opt_multiline(ui, "Description:", &mut param.description);
                c |= row_opt_bool(ui, "Required:", &mut param.required);
                c |= row_opt_bool(ui, "Deprecated:", &mut param.deprecated);
                c
            })
        }
    }
}

fn example_uses(spec: &OpenApiSpec, ref_path: &str) -> Vec<(String, Option<(String, String)>)> {
    let mut uses: Vec<(String, Option<(String, String)>)> = Vec::new();

    let has_ref = |examples: &IndexMap<String, RefOr<OaExample>>| {
        examples.values().any(|e| matches!(e, RefOr::Ref(r) if r.ref_.as_str() == ref_path))
    };

    for (path_key, path_item) in &spec.paths {
        for (method, op) in path_item.operations() {
            if let Some(body) = op.request_body.as_ref().and_then(|rb| rb.as_item()) {
                for (ct, media) in &body.content {
                    if has_ref(&media.examples) {
                        uses.push((
                            format!("{method} {path_key}  ·  Request Body  ·  {ct}"),
                            Some((path_key.clone(), method.to_string())),
                        ));
                    }
                }
            }
            for (code, resp_ref) in &op.responses {
                if let Some(resp) = resp_ref.as_item() {
                    for (ct, media) in &resp.content {
                        if has_ref(&media.examples) {
                            uses.push((
                                format!("{method} {path_key}  ·  {code}  ·  {ct}"),
                                Some((path_key.clone(), method.to_string())),
                            ));
                        }
                    }
                }
            }
        }
    }

    if let Some(comps) = &spec.components {
        for (rb_name, rb_ref) in &comps.request_bodies {
            if let Some(body) = rb_ref.as_item() {
                for (ct, media) in &body.content {
                    if has_ref(&media.examples) {
                        uses.push((format!("Component Request Body: {rb_name}  ·  {ct}"), None));
                    }
                }
            }
        }
        for (resp_name, resp_ref) in &comps.responses {
            if let Some(resp) = resp_ref.as_item() {
                for (ct, media) in &resp.content {
                    if has_ref(&media.examples) {
                        uses.push((format!("Component Response: {resp_name}  ·  {ct}"), None));
                    }
                }
            }
        }
    }

    uses
}

fn edit_example_by_name(ui: &mut Ui, spec: &mut OpenApiSpec, name: &str) -> bool {
    ui.heading(format!("Example: {name}"));
    ui.add_space(4.0);

    // Collect uses while spec is still fully accessible (before mutable sub-borrows).
    let ref_path = format!("#/components/examples/{name}");
    let uses = example_uses(spec, &ref_path);

    let Some(comps) = spec.components.as_mut() else { return false };
    let Some(ex) = comps.examples.get_mut(name) else { ui.label("Not found."); return false };
    match ex {
        RefOr::Ref(r) => { ui.label(format!("$ref: {}", r.ref_)); false }
        RefOr::Item(example) => {
            let mut ch = false;
            ch |= form_grid(ui, &format!("ex_{name}_grid"), |ui| {
                let mut c = false;
                c |= row_opt_str(ui, "Summary:", &mut example.summary);
                c |= row_opt_multiline(ui, "Description:", &mut example.description);
                c |= row_opt_str(ui, "External Value URL:", &mut example.external_value);
                c
            });
            section_header(ui, "Value (JSON):");
            // Persist the raw text buffer so mid-edit invalid JSON doesn't snap back.
            let buf_id     = egui::Id::new(format!("ex_{name}_json_buf"));
            let tracked_id = egui::Id::new(format!("ex_{name}_json_track"));
            let tracked: String = ui.data(|d| d.get_temp(tracked_id).unwrap_or_default());
            if tracked != *name {
                let initial = example.value.as_ref()
                    .map(|v| serde_json::to_string_pretty(v).unwrap_or_default())
                    .unwrap_or_default();
                ui.data_mut(|d| {
                    d.insert_temp(tracked_id, name.to_string());
                    d.insert_temp(buf_id, initial);
                });
            }
            let mut s: String = ui.data(|d| d.get_temp(buf_id).unwrap_or_default());
            let resp = ui.add(egui::TextEdit::multiline(&mut s).desired_rows(8).desired_width(f32::INFINITY).font(egui::TextStyle::Monospace));
            ui.data_mut(|d| d.insert_temp(buf_id, s.clone()));
            if resp.changed() {
                if s.trim().is_empty() {
                    example.value = None;
                    ch = true;
                } else if let Ok(v) = serde_json::from_str::<Value>(&s) {
                    example.value = Some(v);
                    ch = true;
                }
            }

            // ── Used in ───────────────────────────────────────────────────────
            section_header(ui, "Used In:");
            if uses.is_empty() {
                ui.label(RichText::new("Not referenced anywhere.").weak().italics());
            } else {
                for (label, nav) in &uses {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(label).monospace().small());
                        if let Some((path, method)) = nav {
                            if ui.link("Edit").clicked() {
                                ui.data_mut(|d| d.insert_temp(
                                    egui::Id::new("oa_navigate_operation"),
                                    (path.clone(), method.clone()),
                                ));
                            }
                        }
                    });
                }
            }

            ch
        }
    }
}

fn edit_header_by_name(ui: &mut Ui, spec: &mut OpenApiSpec, name: &str) -> bool {
    ui.heading(format!("Header: {name}"));
    ui.add_space(4.0);
    let Some(comps) = spec.components.as_mut() else { return false };
    let Some(hdr) = comps.headers.get_mut(name) else { ui.label("Not found."); return false };
    match hdr {
        RefOr::Ref(r) => { ui.label(format!("$ref: {}", r.ref_)); false }
        RefOr::Item(h) => {
            form_grid(ui, &format!("hdr_{name}_grid"), |ui| {
                let mut c = false;
                c |= row_opt_multiline(ui, "Description:", &mut h.description);
                c |= row_opt_bool(ui, "Required:", &mut h.required);
                c |= row_opt_bool(ui, "Deprecated:", &mut h.deprecated);
                c
            })
        }
    }
}

fn edit_security_scheme_by_name(ui: &mut Ui, spec: &mut OpenApiSpec, name: &str) -> bool {
    ui.heading(format!("Security Scheme: {name}"));
    ui.add_space(4.0);
    let Some(comps) = spec.components.as_mut() else { return false };
    let Some(ss) = comps.security_schemes.get_mut(name) else { ui.label("Not found."); return false };
    match ss {
        RefOr::Ref(r) => { ui.label(format!("$ref: {}", r.ref_)); false }
        RefOr::Item(scheme) => {
            form_grid(ui, &format!("ss_{name}_grid"), |ui| {
                let mut c = false;
                ui.label("Type:");
                egui::ComboBox::from_id_salt(format!("ss_{name}_type"))
                    .selected_text(scheme.type_.as_str())
                    .show_ui(ui, |ui| {
                        for t in ["apiKey","http","oauth2","openIdConnect","mutualTLS"] {
                            if ui.selectable_label(scheme.type_ == t, t).clicked() {
                                scheme.type_ = t.to_string();
                                c = true;
                            }
                        }
                    });
                ui.end_row();
                c |= row_opt_multiline(ui, "Description:", &mut scheme.description);
                if scheme.type_ == "apiKey" {
                    c |= row_opt_str(ui, "Name (header/query key):", &mut scheme.name);
                    ui.label("In:");
                    let in_val = scheme.in_.clone().unwrap_or_default();
                    egui::ComboBox::from_id_salt(format!("ss_{name}_in"))
                        .selected_text(in_val.as_str())
                        .show_ui(ui, |ui| {
                            for loc in ["query","header","cookie"] {
                                if ui.selectable_label(in_val == loc, loc).clicked() {
                                    scheme.in_ = Some(loc.to_string());
                                    c = true;
                                }
                            }
                        });
                    ui.end_row();
                }
                if scheme.type_ == "http" {
                    c |= row_opt_str(ui, "Scheme (basic/bearer/…):", &mut scheme.scheme);
                    c |= row_opt_str(ui, "Bearer Format:", &mut scheme.bearer_format);
                }
                if scheme.type_ == "openIdConnect" {
                    c |= row_opt_str(ui, "OpenID Connect URL:", &mut scheme.open_id_connect_url);
                }
                c
            })
        }
    }
}
