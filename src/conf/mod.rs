use serde::de::{Error, Unexpected};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{from_value, Value};
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ValuesMode {
    #[serde(rename = "override")]
    Override,
    #[serde(rename = "add")]
    Add,
}

#[derive(Debug, Serialize)]
pub struct FieldConfig {
    pub path: String,
    pub values: Option<Vec<serde_json::Value>>,
    pub values_mode: Option<ValuesMode>,
    pub required: Option<bool>,
    pub minmax: Option<(usize, usize)>,

    #[serde(default)]
    pub regex: bool,

    function: Option<String>, // reserved for future feature
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConstraintConfig {
    pub resource_name: String,
    pub fields: Vec<FieldConfig>,
    pub gvk: String,
}

impl<'de> Deserialize<'de> for FieldConfig {
    /*
    since we want the user to be able to supply either a simple
    string or a more complex object, we need to implement a custom
    deserializer.
    */
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let yaml_val = Value::deserialize(deserializer)?;

        match yaml_val {
            Value::String(s) => Ok(FieldConfig {
                path: s,
                values: None,
                values_mode: None,
                required: None,
                function: None,
                minmax: None,
                regex: false,
            }),
            Value::Object(map) => {
                let path = match map.get("path") {
                    Some(p) => from_value(p.clone()).map_err(D::Error::custom)?,
                    None => return Err(D::Error::missing_field("path")),
                };

                let values = from_value(map.get("values").cloned().unwrap_or(Value::Null))
                    .map_err(D::Error::custom)?;
                let values_mode =
                    from_value(map.get("values_mode").cloned().unwrap_or(Value::Null))
                        .map_err(D::Error::custom)?;
                let required = from_value(map.get("required").cloned().unwrap_or(Value::Null))
                    .map_err(D::Error::custom)?;
                let function = from_value(map.get("function").cloned().unwrap_or(Value::Null))
                    .map_err(D::Error::custom)?;
                let minmax = from_value(map.get("minmax").cloned().unwrap_or(Value::Null))
                    .map_err(D::Error::custom)?;
                let regex = from_value(map.get("regex").cloned().unwrap_or(Value::Bool(false)))
                    .map_err(D::Error::custom)?;

                Ok(FieldConfig {
                    path,
                    values,
                    values_mode,
                    required,
                    function,
                    minmax,
                    regex,
                })
            }
            _ => Err(D::Error::invalid_type(
                Unexpected::Other(&"not a string or object"),
                &"a string or FieldConfig",
            )),
        }
    }
}
