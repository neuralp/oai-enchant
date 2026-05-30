use crate::app::{App, Selection};
use crate::lint::{self, Level};
use crate::model::OpenApiSpec;
use egui::RichText;
use std::collections::HashMap;

// ── Search ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct SearchHit {
    kind:       &'static str,
    kind_color: egui::Color32,
    label:      String,
    excerpt:    String,
    sel:        Selection,
}

fn excerpt(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_owned()
    } else {
        format!("{}…", s.chars().take(max).collect::<String>())
    }
}

fn search_spec(spec: &OpenApiSpec, query: &str) -> Vec<SearchHit> {
    let q = query.to_lowercase();
    let mut hits: Vec<SearchHit> = Vec::new();

    macro_rules! hit {
        ($kind:expr, $color:expr, $label:expr, $exc:expr, $sel:expr) => {{
            let label: String = $label;
            let exc: String   = $exc;
            if label.to_lowercase().contains(&q) || exc.to_lowercase().contains(&q) {
                hits.push(SearchHit {
                    kind:       $kind,
                    kind_color: $color,
                    label,
                    excerpt:    exc,
                    sel:        $sel,
                });
            }
        }};
    }

    let grey   = egui::Color32::from_rgb(110, 110, 140);
    let purple = egui::Color32::from_rgb(130, 100, 200);
    let teal   = egui::Color32::from_rgb(60,  160, 160);
    let orange = egui::Color32::from_rgb(200, 130, 60);
    let blue   = egui::Color32::from_rgb(80,  140, 220);
    let green  = egui::Color32::from_rgb(80,  160, 100);
    let pink   = egui::Color32::from_rgb(190, 100, 150);

    // ── Info ──────────────────────────────────────────────────────────────────
    hit!("INFO", grey,
         spec.info.title.clone(),
         excerpt(spec.info.description.as_deref().unwrap_or(""), 50),
         Selection::Info);

    // ── Tags ──────────────────────────────────────────────────────────────────
    for tag in &spec.tags {
        hit!("TAG", grey,
             tag.name.clone(),
             excerpt(tag.description.as_deref().unwrap_or(""), 50),
             Selection::Tag(tag.name.clone()));
    }

    // ── Servers ───────────────────────────────────────────────────────────────
    for srv in &spec.servers {
        hit!("SERVER", grey,
             srv.url.clone(),
             excerpt(srv.description.as_deref().unwrap_or(""), 50),
             Selection::Servers);
    }

    // ── Paths & Operations ────────────────────────────────────────────────────
    for (path_key, path_item) in &spec.paths {
        hit!("PATH", green,
             path_key.clone(),
             excerpt(path_item.summary.as_deref()
                     .or(path_item.description.as_deref())
                     .unwrap_or(""), 50),
             Selection::Path(path_key.clone()));

        for (method, op) in path_item.operations() {
            let label = format!("{method}  {path_key}");
            let mut details = Vec::new();
            if let Some(id) = &op.operation_id { details.push(id.as_str()); }
            if let Some(s)  = &op.summary      { details.push(s.as_str()); }
            if let Some(d)  = &op.description  { details.push(d.as_str()); }
            let exc = excerpt(&details.join(" · "), 55);
            hit!("OPER", method_color(method),
                 label,
                 exc,
                 Selection::Operation(path_key.clone(), method.to_string()));

            // Parameters on the operation
            for p in &op.parameters {
                if let Some(param) = p.as_item() {
                    let pname = format!("{method}  {path_key}  › {}", param.name);
                    let pdesc = excerpt(param.description.as_deref().unwrap_or(""), 45);
                    hit!("PARAM", teal, pname, pdesc,
                         Selection::Operation(path_key.clone(), method.to_string()));
                }
            }
        }
    }

    // ── Component: Schemas ────────────────────────────────────────────────────
    if let Some(comps) = &spec.components {
        for (name, schema_ref) in &comps.schemas {
            if let Some(schema) = schema_ref.as_item() {
                let desc = schema.description.as_deref()
                    .or(schema.title.as_deref())
                    .unwrap_or("");
                hit!("SCHEMA", purple,
                     name.clone(),
                     excerpt(desc, 50),
                     Selection::Schema(name.clone()));

                // Properties
                for (prop_name, prop_schema) in &schema.properties {
                    let plabel = format!("{name}  › {prop_name}");
                    let pdesc  = excerpt(prop_schema.description.as_deref().unwrap_or(""), 45);
                    hit!("PROP", purple, plabel, pdesc,
                         Selection::Schema(name.clone()));
                }
            }
        }

        // ── Component: Request Bodies ─────────────────────────────────────────
        for (name, rb_ref) in &comps.request_bodies {
            if let Some(rb) = rb_ref.as_item() {
                hit!("REQ BODY", orange,
                     name.clone(),
                     excerpt(rb.description.as_deref().unwrap_or(""), 50),
                     Selection::RequestBody(name.clone()));
            }
        }

        // ── Component: Responses ──────────────────────────────────────────────
        for (name, resp_ref) in &comps.responses {
            if let Some(resp) = resp_ref.as_item() {
                hit!("RESPONSE", blue,
                     name.clone(),
                     excerpt(&resp.description, 50),
                     Selection::ComponentResponse(name.clone()));
            }
        }

        // ── Component: Parameters ─────────────────────────────────────────────
        for (name, param_ref) in &comps.parameters {
            if let Some(param) = param_ref.as_item() {
                let exc = excerpt(param.description.as_deref().unwrap_or(""), 50);
                hit!("PARAM", teal, name.clone(), exc,
                     Selection::ComponentParameter(name.clone()));
            }
        }

        // ── Component: Examples ───────────────────────────────────────────────
        for (name, ex_ref) in &comps.examples {
            if let Some(ex) = ex_ref.as_item() {
                let mut details = Vec::new();
                if let Some(s) = &ex.summary     { details.push(s.as_str()); }
                if let Some(d) = &ex.description { details.push(d.as_str()); }
                hit!("EXAMPLE", pink,
                     name.clone(),
                     excerpt(&details.join(" · "), 50),
                     Selection::Example(name.clone()));
            }
        }

        // ── Component: Security Schemes ───────────────────────────────────────
        for (name, ss_ref) in &comps.security_schemes {
            if let Some(ss) = ss_ref.as_item() {
                hit!("SEC SCHEME", grey,
                     name.clone(),
                     excerpt(ss.description.as_deref().unwrap_or(&ss.type_), 50),
                     Selection::SecurityScheme(name.clone()));
            }
        }
    }

    hits
}

const METHOD_COLORS: &[(&str, egui::Color32)] = &[
    ("GET",     egui::Color32::from_rgb(97,  175, 95)),
    ("POST",    egui::Color32::from_rgb(73,  135, 230)),
    ("PUT",     egui::Color32::from_rgb(252, 161, 48)),
    ("PATCH",   egui::Color32::from_rgb(80,  200, 200)),
    ("DELETE",  egui::Color32::from_rgb(220, 80,  80)),
    ("OPTIONS", egui::Color32::from_rgb(155, 89,  182)),
    ("HEAD",    egui::Color32::from_rgb(155, 89,  182)),
    ("TRACE",   egui::Color32::from_rgb(155, 89,  182)),
];

fn method_color(method: &str) -> egui::Color32 {
    METHOD_COLORS.iter().find(|(m, _)| *m == method).map(|(_, c)| *c).unwrap_or(egui::Color32::GRAY)
}

fn level_color(level: Level) -> Option<egui::Color32> {
    match level {
        Level::Error   => Some(egui::Color32::from_rgb(220, 80, 80)),
        Level::Warning => Some(egui::Color32::from_rgb(220, 160, 60)),
        Level::Info    => None,
    }
}

/// Pre-computed worst diagnostic level per item, used to color sidebar entries.
struct DiagLevels {
    paths:          HashMap<String, Level>,
    operations:     HashMap<(String, String), Level>,
    schemas:        HashMap<String, Level>,
    request_bodies: HashMap<String, Level>,
    responses:      HashMap<String, Level>,
    parameters:     HashMap<String, Level>,
    examples:       HashMap<String, Level>,
    tags:           HashMap<String, Level>,
}

impl DiagLevels {
    fn from_diagnostics(diags: &[lint::Diagnostic]) -> Self {
        let mut paths:          HashMap<String, Level> = HashMap::new();
        let mut operations:     HashMap<(String, String), Level> = HashMap::new();
        let mut schemas:        HashMap<String, Level> = HashMap::new();
        let mut request_bodies: HashMap<String, Level> = HashMap::new();
        let mut responses:      HashMap<String, Level> = HashMap::new();
        let mut parameters:     HashMap<String, Level> = HashMap::new();
        let mut examples:       HashMap<String, Level> = HashMap::new();
        let mut tags:           HashMap<String, Level> = HashMap::new();

        for diag in diags {
            let lv = diag.level;
            let Some(sel) = &diag.goto else { continue };
            match sel {
                Selection::Tag(n) => Self::put(&mut tags, n.clone(), lv),
                Selection::Path(p) => Self::put(&mut paths, p.clone(), lv),
                Selection::Operation(p, m) => {
                    Self::put(&mut operations, (p.clone(), m.clone()), lv);
                    Self::put(&mut paths, p.clone(), lv);
                }
                Selection::Schema(n)              => Self::put(&mut schemas,        n.clone(), lv),
                Selection::RequestBody(n)         => Self::put(&mut request_bodies, n.clone(), lv),
                Selection::ComponentResponse(n)   => Self::put(&mut responses,      n.clone(), lv),
                Selection::ComponentParameter(n)  => Self::put(&mut parameters,     n.clone(), lv),
                Selection::Example(n)             => Self::put(&mut examples,       n.clone(), lv),
                _ => {}
            }
        }

        DiagLevels { paths, operations, schemas, request_bodies, responses, parameters, examples, tags }
    }

    fn put<K: Eq + std::hash::Hash>(map: &mut HashMap<K, Level>, key: K, lv: Level) {
        let e = map.entry(key).or_insert(lv);
        if lv < *e { *e = lv; }
    }

    fn item_color(&self, map: &HashMap<String, Level>, name: &str) -> Option<egui::Color32> {
        map.get(name).and_then(|&lv| level_color(lv))
    }

    fn op_color(&self, path: &str, method: &str) -> Option<egui::Color32> {
        self.operations.get(&(path.to_string(), method.to_string())).and_then(|&lv| level_color(lv))
    }
}

// All display data collected from the spec in one pass (owns strings so borrows are released).
struct SidebarData {
    server_labels: Vec<String>,
    tag_names: Vec<String>,
    // (path_key, [method, ...])
    paths: Vec<(String, Vec<String>)>,
    schema_names: Vec<String>,
    rb_names: Vec<String>,
    resp_names: Vec<String>,
    param_names: Vec<String>,
    ex_names: Vec<String>,
    hdr_names: Vec<String>,
    ss_names: Vec<String>,
    diagnostics: Vec<lint::Diagnostic>,
    diag_levels: DiagLevels,
}

impl SidebarData {
    fn from_spec(spec: &OpenApiSpec) -> Self {
        let server_labels = spec
            .servers
            .iter()
            .enumerate()
            .map(|(i, s)| {
                format!("{}: {}", i, s.description.as_deref().unwrap_or(s.url.as_str()))
            })
            .collect();

        let tag_names = spec.tags.iter().map(|t| t.name.clone()).collect();

        let paths = spec
            .paths
            .iter()
            .map(|(k, pi)| {
                let methods: Vec<String> =
                    pi.operations().iter().map(|(m, _)| m.to_string()).collect();
                (k.clone(), methods)
            })
            .collect();

        let comps = spec.components.as_ref();
        let schema_names  = comps.map(|c| c.schemas.keys().cloned().collect()).unwrap_or_default();
        let rb_names      = comps.map(|c| c.request_bodies.keys().cloned().collect()).unwrap_or_default();
        let resp_names    = comps.map(|c| c.responses.keys().cloned().collect()).unwrap_or_default();
        let param_names   = comps.map(|c| c.parameters.keys().cloned().collect()).unwrap_or_default();
        let ex_names      = comps.map(|c| c.examples.keys().cloned().collect()).unwrap_or_default();
        let hdr_names     = comps.map(|c| c.headers.keys().cloned().collect()).unwrap_or_default();
        let ss_names      = comps.map(|c| c.security_schemes.keys().cloned().collect()).unwrap_or_default();

        let diagnostics = lint::lint(spec);
        let diag_levels = DiagLevels::from_diagnostics(&diagnostics);

        SidebarData { server_labels, tag_names, paths, schema_names, rb_names, resp_names, param_names, ex_names, hdr_names, ss_names, diagnostics, diag_levels }
    }
}

pub fn show(ui: &mut egui::Ui, app: &mut App) {
    // ── Search box (fixed, above the scroll area) ─────────────────────────────
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new("🔍").size(14.0));
        let resp = ui.add(
            egui::TextEdit::singleline(&mut app.search_query)
                .id(egui::Id::new("sidebar_search"))
                .hint_text("Search spec…  (Ctrl+F)")
                .desired_width(f32::INFINITY),
        );
        if !app.search_query.is_empty() {
            if ui.small_button("✕").on_hover_text("Clear search").clicked() {
                app.search_query.clear();
                resp.request_focus();
            }
        }
    });
    ui.separator();

    // ── Content ───────────────────────────────────────────────────────────────
    // Collect search hits before borrowing ui for the scroll area.
    let hits: Vec<SearchHit> = if !app.search_query.is_empty() {
        app.spec.as_ref()
            .map(|spec| search_spec(spec, &app.search_query))
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    egui::ScrollArea::vertical()
        .id_salt("sidebar_scroll")
        .show(ui, |ui| {
            if app.search_query.is_empty() {
                let Some(spec) = app.spec.as_ref() else { return };
                let data = SidebarData::from_spec(spec);
                show_tree(ui, app, &data);
            } else {
                show_search_results(ui, app, &hits);
            }
        });
}

fn show_search_results(ui: &mut egui::Ui, app: &mut App, hits: &[SearchHit]) {
    if hits.is_empty() {
        ui.add_space(8.0);
        ui.label(RichText::new("  No results found.").weak().italics());
        return;
    }

    ui.add_space(4.0);
    ui.label(RichText::new(format!("  {} result{}", hits.len(), if hits.len() == 1 { "" } else { "s" })).weak().small());
    ui.add_space(2.0);

    let mut navigate_to: Option<Selection> = None;

    for hit in hits {
        let is_sel = app.selection == hit.sel;
        ui.horizontal(|ui| {
            // Kind badge
            let badge = RichText::new(hit.kind)
                .small()
                .monospace()
                .color(hit.kind_color);
            ui.label(badge);

            // Label + excerpt as a single clickable row
            let line = if hit.excerpt.is_empty() {
                hit.label.clone()
            } else {
                format!("{}\n{}", hit.label, hit.excerpt)
            };

            let resp = ui.selectable_label(is_sel, RichText::new(&hit.label).small().strong());
            if resp.clicked() {
                navigate_to = Some(hit.sel.clone());
            }
            if !hit.excerpt.is_empty() {
                resp.on_hover_text(&hit.excerpt);
            }
            let _ = line; // suppress warning
        });

        // Excerpt on second line if present
        if !hit.excerpt.is_empty() {
            ui.label(RichText::new(format!("    {}", hit.excerpt)).small().weak());
        }

        ui.add_space(1.0);
    }

    if let Some(sel) = navigate_to {
        app.selection = sel;
        app.search_query.clear();
    }
}

fn show_tree(ui: &mut egui::Ui, app: &mut App, data: &SidebarData) {
    ui.add_space(4.0);

    // ── Raw Editor ────────────────────────────────────────────────────────────
    if ui
        .selectable_label(
            app.selection == Selection::RawEditor,
            RichText::new("</>  Raw Editor").strong(),
        )
        .clicked()
    {
        app.open_raw_editor();
    }
    ui.separator();

    // ── API Info ──────────────────────────────────────────────────────────────
    if ui
        .selectable_label(app.selection == Selection::Info, RichText::new("API Info").strong())
        .clicked()
    {
        app.selection = Selection::Info;
    }

    if ui
        .selectable_label(app.selection == Selection::ExternalDocs, RichText::new("External Docs").strong())
        .clicked()
    {
        app.selection = Selection::ExternalDocs;
    }

    // ── Servers ───────────────────────────────────────────────────────────────
    let srv_hdr_color = if app.selection == Selection::Servers {
        ui.visuals().selection.stroke.color
    } else {
        ui.visuals().text_color()
    };
    let srv_cr = egui::CollapsingHeader::new(
        RichText::new(format!("Servers  ({})", data.server_labels.len()))
            .strong()
            .color(srv_hdr_color),
    )
    .id_salt("sb_servers")
    .show(ui, |ui| {
        for label in &data.server_labels {
            let sel = app.selection == Selection::Servers;
            if ui.selectable_label(sel, format!("  {label}")).clicked() {
                app.selection = Selection::Servers;
            }
        }
        if ui.small_button("+ Add Server").clicked() {
            if let Some(spec) = app.spec.as_mut() {
                spec.servers.push(crate::model::Server::default());
                app.dirty = true;
                app.selection = Selection::Servers;
            }
        }
    });
    if srv_cr.header_response.clicked() {
        app.selection = Selection::Servers;
    }

    // ── Tags ──────────────────────────────────────────────────────────────────
    let tags_hdr_color = if app.selection == Selection::Tags {
        ui.visuals().selection.stroke.color
    } else {
        ui.visuals().text_color()
    };
    let tags_cr = egui::CollapsingHeader::new(
        RichText::new(format!("Tags  ({})", data.tag_names.len()))
            .strong()
            .color(tags_hdr_color),
    )
    .id_salt("sb_tags")
    .show(ui, |ui| {
        for name in &data.tag_names {
            let sel = app.selection == Selection::Tag(name.clone());
            let text = match data.diag_levels.item_color(&data.diag_levels.tags, name) {
                Some(c) => RichText::new(format!("  {name}")).color(c),
                None    => RichText::new(format!("  {name}")),
            };
            if ui.selectable_label(sel, text).clicked() {
                app.selection = Selection::Tag(name.clone());
            }
        }
        if ui.small_button("+ Add Tag").clicked() {
            if let Some(spec) = app.spec.as_mut() {
                let n = spec.tags.len() + 1;
                spec.tags.push(crate::model::Tag {
                    name: format!("tag{n}"),
                    ..Default::default()
                });
                app.dirty = true;
                app.selection = Selection::Tags;
            }
        }
    });
    if tags_cr.header_response.clicked() {
        app.selection = Selection::Tags;
    }

    // ── Paths ─────────────────────────────────────────────────────────────────
    let paths_hdr_color = if app.selection == Selection::Paths {
        ui.visuals().selection.stroke.color
    } else {
        ui.visuals().text_color()
    };
    let paths_cr = egui::CollapsingHeader::new(
        RichText::new(format!("Paths  ({})", data.paths.len()))
            .strong()
            .color(paths_hdr_color),
    )
    .id_salt("sb_paths")
    .default_open(true)
    .show(ui, |ui| {
        let mut dup_path: Option<String> = None;
        let mut del_path: Option<String> = None;

        for (path_str, methods) in &data.paths {
            let path_sel = app.selection == Selection::Path(path_str.clone());
            let any_op_sel = methods.iter().any(|m| {
                app.selection == Selection::Operation(path_str.clone(), m.clone())
            });

            let hdr_color = if path_sel || any_op_sel {
                ui.visuals().selection.stroke.color
            } else if let Some(c) = data.diag_levels.item_color(&data.diag_levels.paths, path_str) {
                c
            } else {
                ui.visuals().text_color()
            };

            let cr = egui::CollapsingHeader::new(
                RichText::new(path_str.as_str()).monospace().color(hdr_color),
            )
            .id_salt(format!("sb_path_{path_str}"))
            .default_open(any_op_sel)
            .show(ui, |ui| {
                // Path item row
                if ui.selectable_label(path_sel, "  (path item)").clicked() {
                    app.selection = Selection::Path(path_str.clone());
                }

                // Existing operations
                for method in methods {
                    let op_sel = app.selection == Selection::Operation(path_str.clone(), method.clone());
                    let color = data.diag_levels.op_color(path_str, method)
                        .unwrap_or_else(|| method_color(method));
                    let label = RichText::new(format!("  {method}")).monospace().color(color);
                    if ui.selectable_label(op_sel, label).clicked() {
                        app.selection = Selection::Operation(path_str.clone(), method.clone());
                    }
                }

                // Add operation
                let all_methods = ["GET","POST","PUT","DELETE","PATCH","OPTIONS","HEAD","TRACE"];
                let available: Vec<&str> = all_methods.iter()
                    .filter(|m| !methods.iter().any(|em| em.as_str() == **m))
                    .copied()
                    .collect();
                if !available.is_empty() {
                    ui.menu_button("  + Add operation…", |ui| {
                        for m in &available {
                            if ui.button(*m).clicked() {
                                app.add_operation(path_str, m);
                                ui.close_menu();
                            }
                        }
                    });
                }
            });

            let ps = path_str.clone();
            cr.header_response.context_menu(|ui| {
                if ui.button("Duplicate").clicked() {
                    dup_path = Some(ps.clone());
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(RichText::new("Delete").color(egui::Color32::from_rgb(220, 80, 80))).clicked() {
                    del_path = Some(ps.clone());
                    ui.close_menu();
                }
            });
        }

        if let Some(p) = dup_path { app.duplicate_path(&p); }
        if let Some(p) = del_path  { app.delete_path(&p); }

        // Add path row
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            let resp = ui.add(
                egui::TextEdit::singleline(&mut app.new_item.path)
                    .hint_text("/new-path")
                    .desired_width(140.0)
                    .return_key(None),
            );
            let enter = resp.has_focus()
                && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
            let clicked = ui.small_button("+").clicked();
            if enter || clicked {
                let p = app.new_item.path.clone();
                app.new_item.path.clear();
                app.add_path(p);
            }
        });
    });
    if paths_cr.header_response.clicked() {
        app.selection = Selection::Paths;
    }

    // ── Components ────────────────────────────────────────────────────────────
    egui::CollapsingHeader::new(RichText::new("Components").strong())
        .id_salt("sb_components")
        .default_open(true)
        .show(ui, |ui| {
            // Schemas
            section_with_add(ui, app,
                &format!("Schemas  ({})", data.schema_names.len()),
                "sb_schemas", &data.schema_names,
                |n| Selection::Schema(n), |app, n| app.add_schema(n),
                |app| &mut app.new_item.schema_name,
                |app, n| app.duplicate_schema(&n),
                |app, n| app.delete_schema(&n),
                &[("Add default paths", |app, n| app.add_default_paths_for_schema(n))],
                Some(Selection::Schemas),
                Some(&data.diag_levels.schemas),
            );

            // Request Bodies
            section_with_add(ui, app,
                &format!("Request Bodies  ({})", data.rb_names.len()),
                "sb_reqbodies", &data.rb_names,
                |n| Selection::RequestBody(n), |app, n| app.add_request_body(n),
                |app| &mut app.new_item.request_body_name,
                |app, n| app.duplicate_request_body(&n),
                |app, n| app.delete_request_body(&n),
                &[],
                Some(Selection::RequestBodies),
                Some(&data.diag_levels.request_bodies),
            );

            // Responses
            section_with_add(ui, app,
                &format!("Responses  ({})", data.resp_names.len()),
                "sb_responses", &data.resp_names,
                |n| Selection::ComponentResponse(n), |app, n| app.add_component_response(n),
                |app| &mut app.new_item.response_name,
                |app, n| app.duplicate_component_response(&n),
                |app, n| app.delete_component_response(&n),
                &[],
                Some(Selection::ComponentResponses),
                Some(&data.diag_levels.responses),
            );

            // Parameters
            section_with_add(ui, app,
                &format!("Parameters  ({})", data.param_names.len()),
                "sb_parameters", &data.param_names,
                |n| Selection::ComponentParameter(n), |app, n| app.add_component_parameter(n),
                |app| &mut app.new_item.parameter_name,
                |app, n| app.duplicate_component_parameter(&n),
                |app, n| app.delete_component_parameter(&n),
                &[],
                Some(Selection::ComponentParameters),
                Some(&data.diag_levels.parameters),
            );

            // Examples
            section_with_add(ui, app,
                &format!("Examples  ({})", data.ex_names.len()),
                "sb_examples", &data.ex_names,
                |n| Selection::Example(n), |app, n| app.add_example(n),
                |app| &mut app.new_item.example_name,
                |app, n| app.duplicate_example(&n),
                |app, n| app.delete_example(&n),
                &[],
                Some(Selection::Examples),
                Some(&data.diag_levels.examples),
            );

            // Headers (display only — no individual editor, no reorder page)
            if !data.hdr_names.is_empty() {
                egui::CollapsingHeader::new(format!("  Headers  ({})", data.hdr_names.len()))
                    .id_salt("sb_headers")
                    .show(ui, |ui| {
                        for name in &data.hdr_names {
                            let sel = app.selection == Selection::Header(name.clone());
                            if ui.selectable_label(sel, format!("    {name}")).clicked() {
                                app.selection = Selection::Header(name.clone());
                            }
                        }
                    });
            }

            // Security Schemes
            if !data.ss_names.is_empty() {
                let ss_hdr_color = if app.selection == Selection::SecuritySchemes {
                    ui.visuals().selection.stroke.color
                } else {
                    ui.visuals().text_color()
                };
                let ss_cr = egui::CollapsingHeader::new(
                    RichText::new(format!("  Security Schemes  ({})", data.ss_names.len()))
                        .color(ss_hdr_color),
                )
                .id_salt("sb_secschemes")
                .show(ui, |ui| {
                    for name in &data.ss_names {
                        let sel = app.selection == Selection::SecurityScheme(name.clone());
                        if ui.selectable_label(sel, format!("    {name}")).clicked() {
                            app.selection = Selection::SecurityScheme(name.clone());
                        }
                    }
                });
                if ss_cr.header_response.clicked() {
                    app.selection = Selection::SecuritySchemes;
                }
            }
        });

    // ── Errors & Warnings ─────────────────────────────────────────────────────
    ui.add_space(6.0);
    ui.separator();
    show_diagnostics(ui, app, &data.diagnostics);
}

fn show_diagnostics(ui: &mut egui::Ui, app: &mut App, diags: &[lint::Diagnostic]) {
    let errors   = diags.iter().filter(|d| d.level == Level::Error).count();
    let warnings = diags.iter().filter(|d| d.level == Level::Warning).count();

    let (hdr_text, hdr_color) = if errors > 0 {
        (
            format!("Errors & Warnings  ({errors}E {warnings}W)"),
            egui::Color32::from_rgb(220, 80, 80),
        )
    } else if warnings > 0 {
        (
            format!("Errors & Warnings  ({warnings}W)"),
            egui::Color32::from_rgb(220, 160, 60),
        )
    } else {
        ("Errors & Warnings".to_string(), ui.visuals().weak_text_color())
    };

    let default_open = errors > 0 || warnings > 0;

    egui::CollapsingHeader::new(RichText::new(hdr_text).strong().color(hdr_color))
        .id_salt("sb_diagnostics")
        .default_open(default_open)
        .show(ui, |ui| {
            if diags.is_empty() {
                ui.label(RichText::new("  No issues found").weak().small());
                return;
            }
            let mut goto: Option<Selection> = None;
            for diag in diags {
                let (icon, color) = match diag.level {
                    Level::Error   => ("E", egui::Color32::from_rgb(220, 80, 80)),
                    Level::Warning => ("W", egui::Color32::from_rgb(220, 160, 60)),
                    Level::Info    => ("i", egui::Color32::GRAY),
                };
                ui.horizontal(|ui| {
                    ui.label(RichText::new(icon).small().strong().color(color));
                    let label = RichText::new(&diag.message).small();
                    if diag.goto.is_some() {
                        if ui.selectable_label(false, label).on_hover_text("Click to navigate").clicked() {
                            goto = diag.goto.clone();
                        }
                    } else {
                        ui.label(label);
                    }
                });
            }
            if let Some(sel) = goto {
                app.selection = sel;
            }
        });
}

/// Generic collapsible component section: lists names, handles selection, has an add row.
/// Uses function pointers to avoid borrow issues with &mut App fields.
/// `extra_actions` adds additional context-menu items between Duplicate and Delete.
fn section_with_add(
    ui: &mut egui::Ui,
    app: &mut App,
    label: &str,
    id_salt: &str,
    names: &[String],
    make_sel: fn(String) -> Selection,
    add_fn: fn(&mut App, String),
    get_buf: fn(&mut App) -> &mut String,
    duplicate_fn: fn(&mut App, String),
    delete_fn: fn(&mut App, String),
    extra_actions: &[(&'static str, fn(&mut App, String))],
    header_sel: Option<Selection>,
    diag_colors: Option<&HashMap<String, Level>>,
) {
    let mut dup_name: Option<String> = None;
    let mut del_name: Option<String> = None;
    // (action_index, item_name) — deferred so we apply it after the borrow ends.
    let mut extra_action: Option<(usize, String)> = None;

    let is_hdr_active = header_sel.as_ref().map(|s| &app.selection == s).unwrap_or(false);
    let hdr_color = if is_hdr_active {
        ui.visuals().selection.stroke.color
    } else {
        ui.visuals().text_color()
    };
    let cr = egui::CollapsingHeader::new(
        RichText::new(format!("  {label}")).color(hdr_color),
    )
    .id_salt(id_salt)
    .show(ui, |ui| {
            for name in names {
                let sel = app.selection == make_sel(name.clone());
                let text = match diag_colors.and_then(|m| m.get(name.as_str())).and_then(|&lv| level_color(lv)) {
                    Some(c) => RichText::new(format!("    {name}")).color(c),
                    None    => RichText::new(format!("    {name}")),
                };
                let resp = ui.selectable_label(sel, text);
                if resp.clicked() {
                    app.selection = make_sel(name.clone());
                }
                let n = name.clone();
                resp.context_menu(|ui| {
                    if ui.button("Duplicate").clicked() {
                        dup_name = Some(n.clone());
                        ui.close_menu();
                    }
                    if !extra_actions.is_empty() {
                        ui.separator();
                        for (idx, (act_label, _)) in extra_actions.iter().enumerate() {
                            if ui.button(*act_label).clicked() {
                                extra_action = Some((idx, n.clone()));
                                ui.close_menu();
                            }
                        }
                    }
                    ui.separator();
                    if ui
                        .button(RichText::new("Delete").color(egui::Color32::from_rgb(220, 80, 80)))
                        .clicked()
                    {
                        del_name = Some(n.clone());
                        ui.close_menu();
                    }
                });
            }
            // Split borrows: copy buffer value, call add_fn if clicked
            let buf_val = get_buf(app).clone();
            let mut buf_edit = buf_val;
            ui.horizontal(|ui| {
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut buf_edit)
                        .hint_text("name…")
                        .desired_width(110.0)
                        .return_key(None),
                );
                let enter = resp.has_focus()
                    && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
                let add_clicked = ui.small_button("+").clicked();
                if resp.changed() {
                    *get_buf(app) = buf_edit.clone();
                }
                if (enter || add_clicked) && !buf_edit.is_empty() {
                    *get_buf(app) = String::new();
                    add_fn(app, buf_edit);
                }
            });
        });

    if cr.header_response.clicked() {
        if let Some(sel) = header_sel {
            app.selection = sel;
        }
    }

    if let Some(n) = dup_name { duplicate_fn(app, n); }
    if let Some(n) = del_name { delete_fn(app, n); }
    if let Some((idx, n)) = extra_action { extra_actions[idx].1(app, n); }
}
