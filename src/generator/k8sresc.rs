use serde::Deserializer;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Debug)]
pub struct K8sResourceSpec {
    #[serde(rename = "type")]
    pub _type: String,

    #[serde(default)]
    pub properties: HashMap<String, Box<K8sResourceSpec>>,

    #[serde(rename = "enum", default)]
    pub _enum: Vec<serde_json::Value>,

    #[serde(rename = "enum_regex", default)]
    pub _enum_regex: Vec<String>,

    #[serde(default)]
    pub required: Vec<String>,

    pub minmax: Option<(usize, usize)>,

    pub items: Option<Box<K8sResourceSpec>>,

    pub format: Option<String>,

    pub is_quant: bool,

    #[serde(rename = "additionalProperties")]
    pub additional_properties: Option<Box<K8sResourceSpec>>,

    // set later at runtime based on constraint config
    pub gvk: Option<String>,
}

impl<'de> Deserialize<'de> for K8sResourceSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Intermediate {
            #[serde(rename = "type")]
            _type: String,
            #[serde(default)]
            properties: HashMap<String, Box<K8sResourceSpec>>,
            #[serde(rename = "enum", default)]
            _enum: Vec<Value>,
            #[serde(rename = "enum_regex", default)]
            _enum_regex: Vec<String>,
            #[serde(default)]
            required: Vec<String>,
            minmax: Option<(usize, usize)>,
            items: Option<Box<K8sResourceSpec>>,
            format: Option<String>,
            #[serde(rename = "additionalProperties")]
            additional_properties: Option<Box<K8sResourceSpec>>,
            description: Option<String>,
            gvk: Option<String>,
        }

        let intermediate: Intermediate = Intermediate::deserialize(deserializer)?;
        let is_quant = intermediate
            .description
            .map_or(false, |desc| desc.contains("quantity"));

        Ok(K8sResourceSpec {
            _type: intermediate._type,
            properties: intermediate.properties,
            _enum: intermediate._enum,
            _enum_regex: intermediate._enum_regex,
            required: intermediate.required,
            minmax: intermediate.minmax,
            items: intermediate.items,
            format: intermediate.format,
            additional_properties: intermediate.additional_properties,
            gvk: intermediate.gvk,
            is_quant,
        })
    }
}
