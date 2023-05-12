use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DetailedFieldconfig {
    pub path: String,
    values: Vec<serde_json::Value>,
    optional: Option<bool>,

    function: Option<String>, // reserved for future feature
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FieldConfig {
    String(String),
    Struct(DetailedFieldconfig),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConstraintConfig {
    pub resource_name: String,
    pub fields: Vec<FieldConfig>,
}
