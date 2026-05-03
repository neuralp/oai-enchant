use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── File format ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FileFormat {
    #[default]
    Yaml,
    Json,
}

impl std::fmt::Display for FileFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileFormat::Yaml => write!(f, "YAML"),
            FileFormat::Json => write!(f, "JSON"),
        }
    }
}

// ── RefOr ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RefOr<T> {
    Ref(Ref),
    Item(T),
}

impl<T: Default> Default for RefOr<T> {
    fn default() -> Self {
        RefOr::Item(T::default())
    }
}

impl<T> RefOr<T> {
    pub fn ref_str(&self) -> Option<&str> {
        match self {
            RefOr::Ref(r) => Some(&r.ref_),
            RefOr::Item(_) => None,
        }
    }
    pub fn as_item(&self) -> Option<&T> {
        match self {
            RefOr::Item(t) => Some(t),
            RefOr::Ref(_) => None,
        }
    }
    pub fn as_item_mut(&mut self) -> Option<&mut T> {
        match self {
            RefOr::Item(t) => Some(t),
            RefOr::Ref(_) => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Ref {
    #[serde(rename = "$ref")]
    pub ref_: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// ── Root spec ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OpenApiSpec {
    pub openapi: String,
    pub info: Info,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub servers: Vec<Server>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub paths: IndexMap<String, PathItem>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub webhooks: IndexMap<String, PathItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Components>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub security: Vec<IndexMap<String, Vec<String>>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<ExternalDocs>,
}

// ── Info ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Info {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terms_of_service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<Contact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<License>,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Contact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct License {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

// ── Server ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Server {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub variables: IndexMap<String, ServerVariable>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ServerVariable {
    #[serde(rename = "enum", skip_serializing_if = "Vec::is_empty")]
    pub enum_: Vec<String>,
    pub default: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// ── Tag ───────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Tag {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<ExternalDocs>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ExternalDocs {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// ── Components ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Components {
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub schemas: IndexMap<String, RefOr<Schema>>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub responses: IndexMap<String, RefOr<Response>>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub parameters: IndexMap<String, RefOr<Parameter>>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub examples: IndexMap<String, RefOr<OaExample>>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub request_bodies: IndexMap<String, RefOr<RequestBody>>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub headers: IndexMap<String, RefOr<Header>>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub security_schemes: IndexMap<String, RefOr<SecurityScheme>>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub links: IndexMap<String, RefOr<Link>>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub path_items: IndexMap<String, PathItem>,
}

// ── Path / Operation ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PathItem {
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub ref_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub put: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<Operation>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub servers: Vec<Server>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<RefOr<Parameter>>,
}

impl PathItem {
    pub fn operations(&self) -> Vec<(&'static str, &Operation)> {
        let mut ops = Vec::new();
        if let Some(op) = &self.get    { ops.push(("GET",     op)); }
        if let Some(op) = &self.put    { ops.push(("PUT",     op)); }
        if let Some(op) = &self.post   { ops.push(("POST",    op)); }
        if let Some(op) = &self.delete { ops.push(("DELETE",  op)); }
        if let Some(op) = &self.options { ops.push(("OPTIONS", op)); }
        if let Some(op) = &self.head   { ops.push(("HEAD",    op)); }
        if let Some(op) = &self.patch  { ops.push(("PATCH",   op)); }
        if let Some(op) = &self.trace  { ops.push(("TRACE",   op)); }
        ops
    }

    pub fn operation_mut(&mut self, method: &str) -> Option<&mut Operation> {
        match method {
            "GET"     => self.get.as_mut(),
            "PUT"     => self.put.as_mut(),
            "POST"    => self.post.as_mut(),
            "DELETE"  => self.delete.as_mut(),
            "OPTIONS" => self.options.as_mut(),
            "HEAD"    => self.head.as_mut(),
            "PATCH"   => self.patch.as_mut(),
            "TRACE"   => self.trace.as_mut(),
            _ => None,
        }
    }

    pub fn set_operation(&mut self, method: &str, op: Option<Operation>) {
        match method {
            "GET"     => self.get     = op,
            "PUT"     => self.put     = op,
            "POST"    => self.post    = op,
            "DELETE"  => self.delete  = op,
            "OPTIONS" => self.options = op,
            "HEAD"    => self.head    = op,
            "PATCH"   => self.patch   = op,
            "TRACE"   => self.trace   = op,
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Operation {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<ExternalDocs>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<RefOr<Parameter>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_body: Option<RefOr<RequestBody>>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub responses: IndexMap<String, RefOr<Response>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<IndexMap<String, Vec<String>>>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub servers: Vec<Server>,
}

// ── Parameter ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub in_: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_empty_value: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<RefOr<Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<Value>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub examples: IndexMap<String, RefOr<OaExample>>,
}

// ── RequestBody ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct RequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub content: IndexMap<String, MediaType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct MediaType {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<RefOr<Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<Value>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub examples: IndexMap<String, RefOr<OaExample>>,
}

// ── Response ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Response {
    pub description: String,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub headers: IndexMap<String, RefOr<Header>>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub content: IndexMap<String, MediaType>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub links: IndexMap<String, RefOr<Link>>,
}

// ── Header ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Header {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<RefOr<Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<Value>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub examples: IndexMap<String, RefOr<OaExample>>,
}

// ── Example ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OaExample {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_value: Option<String>,
}

// ── Link ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Link {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub parameters: IndexMap<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_body: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<Server>,
}

// ── SecurityScheme ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct SecurityScheme {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "in", skip_serializing_if = "Option::is_none")]
    pub in_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flows: Option<OAuthFlows>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_id_connect_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OAuthFlows {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implicit: Option<OAuthFlow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<OAuthFlow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_credentials: Option<OAuthFlow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_code: Option<OAuthFlow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OAuthFlow {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<String>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub scopes: IndexMap<String, String>,
}

// ── Schema ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Schema {
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub ref_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    // type: string in 3.0, string or [string] in 3.1
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nullable: Option<bool>,
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<Value>,
    // Object
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub properties: IndexMap<String, Box<Schema>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<Box<AdditionalProperties>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_properties: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_properties: Option<u64>,
    // Array
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique_items: Option<bool>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub prefix_items: Vec<Box<Schema>>,
    // String
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    // Number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusive_minimum: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusive_maximum: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple_of: Option<f64>,
    // Composition
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub all_of: Vec<Box<Schema>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub any_of: Vec<Box<Schema>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub one_of: Vec<Box<Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not: Option<Box<Schema>>,
    // Conditional (3.1+)
    #[serde(rename = "if", skip_serializing_if = "Option::is_none")]
    pub if_: Option<Box<Schema>>,
    #[serde(rename = "then", skip_serializing_if = "Option::is_none")]
    pub then_: Option<Box<Schema>>,
    #[serde(rename = "else", skip_serializing_if = "Option::is_none")]
    pub else_: Option<Box<Schema>>,
    // OpenAPI specific
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discriminator: Option<Discriminator>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<ExternalDocs>,
    // JSON Schema $defs (3.1+)
    #[serde(rename = "$defs", skip_serializing_if = "IndexMap::is_empty")]
    pub defs: IndexMap<String, Box<Schema>>,
}

impl Schema {
    pub fn type_str(&self) -> &str {
        match &self.type_ {
            Some(Value::String(s)) => s.as_str(),
            _ => "",
        }
    }

    pub fn set_type_str(&mut self, s: &str) {
        if s.is_empty() {
            self.type_ = None;
        } else {
            self.type_ = Some(Value::String(s.to_string()));
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AdditionalProperties {
    Bool(bool),
    Schema(Schema),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Discriminator {
    pub property_name: String,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub mapping: IndexMap<String, String>,
}
