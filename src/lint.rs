use crate::app::Selection;
use crate::model::{OpenApiSpec, RefOr};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: Level,
    pub message: String,
    /// Selection to navigate to when the user clicks this entry.
    pub goto: Option<Selection>,
}

impl Diagnostic {
    fn error(msg: impl Into<String>, goto: Option<Selection>) -> Self {
        Self { level: Level::Error, message: msg.into(), goto }
    }
    fn warning(msg: impl Into<String>, goto: Option<Selection>) -> Self {
        Self { level: Level::Warning, message: msg.into(), goto }
    }
    fn info(msg: impl Into<String>, goto: Option<Selection>) -> Self {
        Self { level: Level::Info, message: msg.into(), goto }
    }
}

pub fn lint(spec: &OpenApiSpec) -> Vec<Diagnostic> {
    let mut d: Vec<Diagnostic> = Vec::new();

    // ── Info ──────────────────────────────────────────────────────────────────
    if spec.info.title.trim().is_empty() {
        d.push(Diagnostic::error("API title is empty", Some(Selection::Info)));
    }
    if spec.info.version.trim().is_empty() {
        d.push(Diagnostic::warning("API version is empty", Some(Selection::Info)));
    }
    if spec.info.description.as_deref().unwrap_or("").trim().is_empty() {
        d.push(Diagnostic::info("API description is missing", Some(Selection::Info)));
    }

    if spec.paths.is_empty() && spec.webhooks.is_empty() {
        d.push(Diagnostic::info("No paths or webhooks defined", None));
    }

    // ── Build component key sets for $ref validation ───────────────────────────
    let comps = spec.components.as_ref();
    let schema_keys: HashSet<String> = comps
        .map(|c| c.schemas.keys().map(|k| format!("#/components/schemas/{k}")).collect())
        .unwrap_or_default();
    let param_keys: HashSet<String> = comps
        .map(|c| c.parameters.keys().map(|k| format!("#/components/parameters/{k}")).collect())
        .unwrap_or_default();
    let resp_keys: HashSet<String> = comps
        .map(|c| c.responses.keys().map(|k| format!("#/components/responses/{k}")).collect())
        .unwrap_or_default();
    let rb_keys: HashSet<String> = comps
        .map(|c| c.request_bodies.keys().map(|k| format!("#/components/requestBodies/{k}")).collect())
        .unwrap_or_default();

    // ── Declared global tags ───────────────────────────────────────────────────
    let declared_tags: HashSet<&str> = spec.tags.iter().map(|t| t.name.as_str()).collect();

    // ── Collect all operationIds to detect duplicates ──────────────────────────
    let mut op_id_map: HashMap<String, Vec<(String, String)>> = HashMap::new(); // id → [(path, method)]

    // ── Paths ─────────────────────────────────────────────────────────────────
    for (path, path_item) in &spec.paths {
        let template_params = extract_path_params(path);

        for (method, op) in path_item.operations() {
            let sel = Some(Selection::Operation(path.clone(), method.to_string()));

            // operationId
            match op.operation_id.as_deref() {
                None | Some("") => {
                    d.push(Diagnostic::warning(
                        format!("{method} {path}: missing operationId"),
                        sel.clone(),
                    ));
                }
                Some(id) => {
                    op_id_map.entry(id.to_string()).or_default().push((path.clone(), method.to_string()));
                }
            }

            // Responses
            if op.responses.is_empty() {
                d.push(Diagnostic::error(
                    format!("{method} {path}: no responses defined"),
                    sel.clone(),
                ));
            } else {
                let has_success = op.responses.keys().any(|k| {
                    k == "default" || k.starts_with('2') || k.starts_with('3')
                });
                if !has_success {
                    d.push(Diagnostic::warning(
                        format!("{method} {path}: no 2xx/3xx/default response defined"),
                        sel.clone(),
                    ));
                }
                for (code, rr) in &op.responses {
                    if let Some(r) = rr.ref_str() {
                        if !resp_keys.contains(r) {
                            d.push(Diagnostic::error(
                                format!("{method} {path} [{code}]: broken $ref '{r}'"),
                                sel.clone(),
                            ));
                        }
                    } else if let Some(resp) = rr.as_item() {
                        if resp.description.trim().is_empty() {
                            d.push(Diagnostic::warning(
                                format!("{method} {path} [{code}]: response description is empty"),
                                sel.clone(),
                            ));
                        }
                    }
                }
            }

            // Tags
            for tag in &op.tags {
                if !declared_tags.contains(tag.as_str()) {
                    d.push(Diagnostic::warning(
                        format!("{method} {path}: tag '{tag}' not declared in global Tags"),
                        sel.clone(),
                    ));
                }
            }

            // Request body $ref
            if let Some(rb) = &op.request_body {
                if let Some(r) = rb.ref_str() {
                    if !rb_keys.contains(r) {
                        d.push(Diagnostic::error(
                            format!("{method} {path}: broken requestBody $ref '{r}'"),
                            sel.clone(),
                        ));
                    }
                }
            }

            // Parameters — collect defined path params to compare with template
            let mut covered_path_params: HashSet<String> = HashSet::new();
            let all_params = path_item.parameters.iter().chain(op.parameters.iter());
            for pr in all_params {
                match pr {
                    RefOr::Ref(r) => {
                        if !param_keys.contains(&r.ref_) {
                            d.push(Diagnostic::error(
                                format!("{method} {path}: broken parameter $ref '{}'", r.ref_),
                                sel.clone(),
                            ));
                        }
                    }
                    RefOr::Item(p) => {
                        if p.name.trim().is_empty() {
                            d.push(Diagnostic::warning(
                                format!("{method} {path}: a parameter has no name"),
                                sel.clone(),
                            ));
                        }
                        if p.in_ == "path" {
                            covered_path_params.insert(p.name.clone());
                            if !template_params.contains(&p.name) {
                                d.push(Diagnostic::warning(
                                    format!(
                                        "{method} {path}: path parameter '{}' not in URL template",
                                        p.name
                                    ),
                                    sel.clone(),
                                ));
                            }
                        }
                    }
                }
            }
            // Template params with no matching parameter object
            for pp in &template_params {
                if !covered_path_params.contains(pp) {
                    d.push(Diagnostic::warning(
                        format!("{method} {path}: {{{pp}}} in URL has no parameter definition"),
                        sel.clone(),
                    ));
                }
            }
        }
    }

    // ── Duplicate operationIds ─────────────────────────────────────────────────
    for (id, locations) in &op_id_map {
        if locations.len() > 1 {
            for (path, method) in locations {
                d.push(Diagnostic::error(
                    format!("{method} {path}: duplicate operationId '{id}'"),
                    Some(Selection::Operation(path.clone(), method.clone())),
                ));
            }
        }
    }

    // ── Component schemas — broken $refs ──────────────────────────────────────
    if let Some(comps) = &spec.components {
        for (name, sr) in &comps.schemas {
            if let RefOr::Ref(r) = sr {
                if !schema_keys.contains(&r.ref_) {
                    d.push(Diagnostic::error(
                        format!("Schema '{name}': broken $ref '{}'", r.ref_),
                        Some(Selection::Schema(name.clone())),
                    ));
                }
            }
        }
    }

    // Sort: errors first, warnings second, info last.
    d.sort_by_key(|x| x.level);
    d
}

fn extract_path_params(path: &str) -> HashSet<String> {
    let mut params = HashSet::new();
    let mut s = path;
    while let Some(start) = s.find('{') {
        s = &s[start + 1..];
        if let Some(end) = s.find('}') {
            params.insert(s[..end].to_string());
            s = &s[end + 1..];
        } else {
            break;
        }
    }
    params
}
