use serde_json::{Value, json};
use badbee_backend::model::model::{DataValue};
use crate::handlers::vec2id;
use std::collections::HashMap;
use badbee_backend::db::DataFieldValue;

pub(crate) fn to_json(val: &DataFieldValue, embed_refs: bool) -> Value {
    let mut out = match &val.value {
        DataValue::Null => json!({"value": Value::Null}),
        DataValue::Boolean { value } => json!({"value": value, "type": "boolean"}),
        DataValue::Int { value } => json!({"value": value, "type": "int"}),
        DataValue::Float { value } => json!({"value": value, "type": "float"}),
        DataValue::String { value } => json!({"value": value, "type": "string"}),
        DataValue::Color { value } => json!({"value": value.to_hex_color(), "type": "color"}),
        DataValue::Image { width, height, data_url } => json!({
                "type": "image", "value": {
                    "width": width, "height": height, "data_url": data_url
                }
            }),
        DataValue::Histogram { value } => json!({
            "type": "pie",
            "value": value.iter().map(|(k,v)| (k.to_hex_color(), *v)).collect::<HashMap<String, f32>>()
        }),
        DataValue::Custom { subtype, value } => json!({"type": subtype, "value": value})
    };
    if let Some(id) = val.reference {
        out["reference"] = json!(vec2id(id));
        if !embed_refs {
            out["value"] = Value::Null;
        }
    }
    json!(out)
}

pub(crate) fn from_json(val: &Value) -> Option<DataValue> {
    //todo: check "type" as well
    let _vtype = val.get("type").and_then(|v| v.as_str());
    let val = val.get("value")?;
    match val {
        Value::Null => None,
        Value::Bool(value) => Some(DataValue::Boolean { value: *value }),
        Value::Number(value) if value.is_f64() => Some(DataValue::Float { value: value.as_f64().unwrap() as f32 }),
        Value::Number(value) => Some(DataValue::Int { value: value.as_i64().unwrap() as i32 }),
        Value::String(value) if value.starts_with("#") => Some(DataValue::Color { value: value.into() }),
        Value::String(value) => Some(DataValue::String { value: value.clone() }),
        Value::Array(_) => None,
        Value::Object(obj) if obj.contains_key("data_url") =>
        //todo: support other objects
            Some(DataValue::Image {
                width: obj["width"].as_i64().unwrap() as u32,
                height: obj["height"].as_i64().unwrap() as u32,
                data_url: obj["data_url"].as_str().unwrap().to_string(),
            }),
        Value::Object(obj) =>
            Some(DataValue::Histogram {
                value: obj.iter()
                    .map(|(k, v)| (k.into(), v.as_f64().unwrap() as f32))
                    .collect()
            })
    }
}


