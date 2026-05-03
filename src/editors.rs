use crate::app::{NewItemBuffers, Selection};
use crate::model::*;
use egui::{Grid, RichText, ScrollArea, Ui};
use serde_json::Value;

// ── Entry point ───────────────────────────────────────────────────────────────

/// Returns true if any value was changed.
pub fn show(
    ui: &mut Ui,
    spec: &mut OpenApiSpec,
    selection: &Selection,
    new_item: &mut NewItemBuffers,
) -> bool {
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
            Selection::Link(_) => {
                ui.label("Link editing not yet implemented.");
            }
        }
    });
    changed
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
    let r = ui.checkbox(&mut b, "").changed();
    ui.end_row();
    if r {
        *val = Some(b);
    }
    r
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

    if ui.button("＋ Add Server").clicked() {
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

    if ui.button("＋ Add Tag").clicked() {
        spec.tags.push(Tag { name: format!("tag{}", spec.tags.len() + 1), ..Default::default() });
        ch = true;
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
    ui.heading("Path Item");
    ui.label(RichText::new(path).monospace().strong());
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
            } else if ui.small_button("＋ Add").clicked() {
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

        // Tags as comma-separated string
        let mut tags_str = op.tags.join(", ");
        ui.label("Tags (comma-separated):");
        if ui.text_edit_singleline(&mut tags_str).changed() {
            op.tags = tags_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            c = true;
        }
        ui.end_row();

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
        if ui.small_button("＋ Add Parameter").clicked() {
            op.parameters.push(RefOr::Item(Parameter {
                in_: "query".to_string(),
                ..Default::default()
            }));
            ch = true;
        }
        ui.add(
            egui::TextEdit::singleline(&mut new_item.parameter_name)
                .hint_text("$ref path…")
                .desired_width(180.0),
        );
        if ui.small_button("＋ Add $ref").clicked() && !new_item.parameter_name.is_empty() {
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
            if ui.button("＋ Add Request Body").clicked() {
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

    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut new_item.response_code)
                .hint_text("200")
                .desired_width(60.0),
        );
        if ui.small_button("＋ Add Response").clicked() && !new_item.response_code.is_empty() {
            let code = new_item.response_code.clone();
            new_item.response_code.clear();
            op.responses.insert(code, RefOr::Item(Response {
                description: "OK".to_string(),
                ..Default::default()
            }));
            ch = true;
        }
    });

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

    // Add content type
    ui.horizontal(|ui| {
        // Use egui memory for the ephemeral buffer
        let buf_id = egui::Id::new(format!("{id}_new_ct"));
        let mut buf: String = ui.data_mut(|d| d.get_temp::<String>(buf_id).unwrap_or_default());
        ui.add(egui::TextEdit::singleline(&mut buf).hint_text("application/json").desired_width(180.0));
        ui.data_mut(|d| d.insert_temp(buf_id, buf.clone()));
        if ui.small_button("＋").clicked() && !buf.is_empty() {
            body.content.entry(buf.clone()).or_default();
            ui.data_mut(|d| d.insert_temp(buf_id, String::new()));
            ch = true;
        }
    });

    ch
}

fn edit_media_type(ui: &mut Ui, media: &mut MediaType, id: &str) -> bool {
    let mut ch = false;
    // Schema ref
    ui.label("Schema:");
    match &media.schema {
        None => {
            if ui.small_button("＋ Set Schema").clicked() {
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

    ui.horizontal(|ui| {
        let buf_id = egui::Id::new(format!("{id}_new_ct"));
        let mut buf: String = ui.data_mut(|d| d.get_temp::<String>(buf_id).unwrap_or_default());
        ui.add(egui::TextEdit::singleline(&mut buf).hint_text("application/json").desired_width(180.0));
        ui.data_mut(|d| d.insert_temp(buf_id, buf.clone()));
        if ui.small_button("＋").clicked() && !buf.is_empty() {
            resp.content.entry(buf.clone()).or_default();
            ui.data_mut(|d| d.insert_temp(buf_id, String::new()));
            ch = true;
        }
    });

    ch
}

// ── Schema editors ────────────────────────────────────────────────────────────

fn edit_schema_by_name(ui: &mut Ui, spec: &mut OpenApiSpec, name: &str) -> bool {
    ui.heading(format!("Schema: {name}"));
    ui.add_space(4.0);

    let Some(comps) = spec.components.as_mut() else {
        ui.label("No components.");
        return false;
    };
    let Some(schema_ref) = comps.schemas.get_mut(name) else {
        ui.label("Schema not found.");
        return false;
    };

    match schema_ref {
        RefOr::Ref(r) => {
            ui.label(format!("$ref: {}", r.ref_));
            false
        }
        RefOr::Item(schema) => edit_schema_inline(ui, schema, name, 0),
    }
}

/// Edit a schema inline. `depth` limits property nesting depth.
pub fn edit_schema_inline(ui: &mut Ui, schema: &mut Schema, id: &str, depth: u32) -> bool {
    let mut ch = false;

    // $ref
    if schema.ref_.is_some() {
        ch |= form_grid(ui, &format!("{id}_ref_grid"), |ui| {
            let mut r = schema.ref_.clone().unwrap_or_default();
            ui.label("$ref:");
            let changed = ui.text_edit_singleline(&mut r).changed();
            ui.end_row();
            if changed {
                schema.ref_ = if r.is_empty() { None } else { Some(r) };
            }
            changed
        });
        return ch;
    }

    ch |= form_grid(ui, &format!("{id}_base_grid"), |ui| {
        let mut c = false;
        c |= row_opt_str(ui, "Title:", &mut schema.title);
        c |= row_opt_str(ui, "Description:", &mut schema.description);

        // Type dropdown (common 3.0/3.1 types)
        let mut type_str = schema.type_str().to_string();
        ui.label("Type:");
        egui::ComboBox::from_id_salt(format!("{id}_type"))
            .selected_text(type_str.as_str())
            .show_ui(ui, |ui| {
                for t in ["","string","number","integer","boolean","array","object","null"] {
                    if ui.selectable_label(type_str == t, t).clicked() {
                        type_str = t.to_string();
                        c = true;
                    }
                }
            });
        ui.end_row();
        if c { schema.set_type_str(&type_str); }

        c |= row_opt_str(ui, "Format:", &mut schema.format);
        c |= row_opt_bool(ui, "Nullable (3.0):", &mut schema.nullable);
        c |= row_opt_bool(ui, "Read Only:", &mut schema.read_only);
        c |= row_opt_bool(ui, "Write Only:", &mut schema.write_only);
        c |= row_opt_bool(ui, "Deprecated:", &mut schema.deprecated);
        c
    });

    let type_str = schema.type_str().to_string();

    // String constraints
    if type_str == "string" || type_str.is_empty() {
        egui::CollapsingHeader::new("String Constraints")
            .id_salt(format!("{id}_str_constraints"))
            .show(ui, |ui| {
                ch |= form_grid(ui, &format!("{id}_str_grid"), |ui| {
                    let mut c = false;
                    c |= row_opt_u64(ui, "Min Length:", &mut schema.min_length);
                    c |= row_opt_u64(ui, "Max Length:", &mut schema.max_length);
                    c |= row_opt_str(ui, "Pattern:", &mut schema.pattern);
                    c
                });
            });
    }

    // Number constraints
    if type_str == "number" || type_str == "integer" {
        egui::CollapsingHeader::new("Number Constraints")
            .id_salt(format!("{id}_num_constraints"))
            .show(ui, |ui| {
                ch |= form_grid(ui, &format!("{id}_num_grid"), |ui| {
                    let mut c = false;
                    c |= row_opt_f64(ui, "Minimum:", &mut schema.minimum);
                    c |= row_opt_f64(ui, "Maximum:", &mut schema.maximum);
                    c |= row_opt_f64(ui, "Multiple Of:", &mut schema.multiple_of);
                    c
                });
            });
    }

    // Array constraints
    if type_str == "array" {
        egui::CollapsingHeader::new("Array Constraints")
            .id_salt(format!("{id}_arr_constraints"))
            .default_open(true)
            .show(ui, |ui| {
                ch |= form_grid(ui, &format!("{id}_arr_grid"), |ui| {
                    let mut c = false;
                    c |= row_opt_u64(ui, "Min Items:", &mut schema.min_items);
                    c |= row_opt_u64(ui, "Max Items:", &mut schema.max_items);
                    c |= row_opt_bool(ui, "Unique Items:", &mut schema.unique_items);
                    c
                });

                if depth < 3 {
                    ui.label("Items Schema:");
                    let items = schema.items.get_or_insert_with(|| Box::new(Schema::default()));
                    ch |= edit_schema_inline(ui, items, &format!("{id}_items"), depth + 1);
                } else {
                    ui.label(RichText::new("(max nesting depth reached)").weak());
                }
            });
    }

    // Object properties
    if type_str == "object" || !schema.properties.is_empty() {
        egui::CollapsingHeader::new("Properties")
            .id_salt(format!("{id}_props"))
            .default_open(true)
            .show(ui, |ui| {
                ch |= form_grid(ui, &format!("{id}_obj_grid"), |ui| {
                    let mut c = false;
                    c |= row_opt_u64(ui, "Min Properties:", &mut schema.min_properties);
                    c |= row_opt_u64(ui, "Max Properties:", &mut schema.max_properties);
                    c
                });

                let prop_keys: Vec<String> = schema.properties.keys().cloned().collect();
                let mut to_remove: Option<String> = None;

                for prop_name in &prop_keys {
                    let is_required = schema.required.contains(prop_name);
                    let req_label = if is_required { "* " } else { "  " };

                    if let Some(prop_schema) = schema.properties.get_mut(prop_name) {
                        let ptype = prop_schema.type_str().to_string();
                        let hdr = format!("{req_label}{prop_name}: {ptype}");
                        egui::CollapsingHeader::new(RichText::new(&hdr).monospace())
                            .id_salt(format!("{id}_prop_{prop_name}"))
                            .show(ui, |ui| {
                                if depth < 3 {
                                    ch |= edit_schema_inline(
                                        ui, prop_schema,
                                        &format!("{id}_prop_{prop_name}_schema"),
                                        depth + 1,
                                    );
                                }
                                ui.horizontal(|ui| {
                                    let mut req = is_required;
                                    if ui.checkbox(&mut req, "Required").changed() {
                                        if req {
                                            if !schema.required.contains(prop_name) {
                                                schema.required.push(prop_name.clone());
                                            }
                                        } else {
                                            schema.required.retain(|r| r != prop_name);
                                        }
                                        ch = true;
                                    }
                                    if ui.small_button("🗑 Remove").clicked() {
                                        to_remove = Some(prop_name.clone());
                                    }
                                });
                            });
                    }
                }

                if let Some(k) = to_remove {
                    schema.properties.shift_remove(&k);
                    schema.required.retain(|r| *r != k);
                    ch = true;
                }

                // Add property
                ui.horizontal(|ui| {
                    let buf_id = egui::Id::new(format!("{id}_new_prop"));
                    let mut buf: String = ui.data_mut(|d| d.get_temp::<String>(buf_id).unwrap_or_default());
                    ui.add(egui::TextEdit::singleline(&mut buf).hint_text("property name").desired_width(140.0));
                    ui.data_mut(|d| d.insert_temp(buf_id, buf.clone()));
                    if ui.small_button("＋").clicked() && !buf.is_empty() {
                        schema.properties.insert(buf.clone(), Box::new(Schema::default()));
                        ui.data_mut(|d| d.insert_temp(buf_id, String::new()));
                        ch = true;
                    }
                });
            });
    }

    // Enum values
    egui::CollapsingHeader::new("Enum Values")
        .id_salt(format!("{id}_enum"))
        .show(ui, |ui| {
            let enum_str = schema.enum_.as_deref()
                .map(|vals| vals.iter().map(|v| serde_json::to_string(v).unwrap_or_default()).collect::<Vec<_>>().join("\n"))
                .unwrap_or_default();
            let mut s = enum_str;
            ui.label(RichText::new("One value per line (JSON format):").weak().small());
            if ui.add(egui::TextEdit::multiline(&mut s).desired_rows(4).desired_width(f32::INFINITY)).changed() {
                let vals: Vec<Value> = s.lines()
                    .filter(|l| !l.trim().is_empty())
                    .filter_map(|l| serde_json::from_str(l).ok())
                    .collect();
                schema.enum_ = if vals.is_empty() { None } else { Some(vals) };
                ch = true;
            }
        });

    // Composition (allOf / anyOf / oneOf)
    if !schema.all_of.is_empty() || !schema.any_of.is_empty() || !schema.one_of.is_empty() || depth == 0 {
        egui::CollapsingHeader::new("Composition (allOf / anyOf / oneOf)")
            .id_salt(format!("{id}_composition"))
            .show(ui, |ui| {
                ch |= edit_schema_list(ui, "allOf", &mut schema.all_of, &format!("{id}_allof"), depth);
                ch |= edit_schema_list(ui, "anyOf", &mut schema.any_of, &format!("{id}_anyof"), depth);
                ch |= edit_schema_list(ui, "oneOf", &mut schema.one_of, &format!("{id}_oneof"), depth);
            });
    }

    ch
}

fn edit_schema_list(ui: &mut Ui, label: &str, list: &mut Vec<Box<Schema>>, id: &str, depth: u32) -> bool {
    let mut ch = false;
    ui.label(RichText::new(label).strong());
    let mut to_remove: Option<usize> = None;

    for (i, schema) in list.iter_mut().enumerate() {
        egui::CollapsingHeader::new(
            schema.ref_.as_deref().map(|r| format!("$ref: {r}")).unwrap_or_else(|| format!("(schema {i})"))
        )
        .id_salt(format!("{id}_{i}"))
        .show(ui, |ui| {
            if depth < 2 {
                ch |= edit_schema_inline(ui, schema, &format!("{id}_{i}_inner"), depth + 1);
            }
            if ui.small_button("🗑 Remove").clicked() {
                to_remove = Some(i);
            }
        });
    }

    if let Some(idx) = to_remove {
        list.remove(idx);
        ch = true;
    }

    if ui.small_button(format!("＋ Add {label} entry")).clicked() {
        list.push(Box::new(Schema::default()));
        ch = true;
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

fn edit_example_by_name(ui: &mut Ui, spec: &mut OpenApiSpec, name: &str) -> bool {
    ui.heading(format!("Example: {name}"));
    ui.add_space(4.0);
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
            let val_str = example.value.as_ref()
                .map(|v| serde_json::to_string_pretty(v).unwrap_or_default())
                .unwrap_or_default();
            let mut s = val_str;
            if ui.add(egui::TextEdit::multiline(&mut s).desired_rows(8).desired_width(f32::INFINITY).font(egui::TextStyle::Monospace)).changed() {
                example.value = serde_json::from_str(&s).ok();
                ch = true;
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
