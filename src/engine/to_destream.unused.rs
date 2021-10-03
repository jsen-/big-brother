use std::collections::HashMap;

pub fn convert_value_to_value(input: &serde_json::Value) -> destream_json::Value {
    match input {
        serde_json::Value::Array(v) => {
            destream_json::Value::List(v.into_iter().map(convert_value_to_value).collect::<Vec<_>>())
        }
        serde_json::Value::Bool(b) => destream_json::Value::Number(number_general::Number::Bool((*b).into())),
        serde_json::Value::Null => destream_json::Value::None,
        serde_json::Value::Number(n) => {
            let n = match n.as_f64() {
                Some(fl) => number_general::Number::Float(number_general::Float::F64(fl)),
                None => match n.as_i64() {
                    Some(i) => number_general::Number::Int(number_general::Int::I64(i)),
                    None => match n.as_u64() {
                        Some(u) => number_general::Number::UInt(number_general::UInt::U64(u)),
                        None => panic!(),
                    },
                },
            };
            destream_json::Value::Number(n)
        }
        serde_json::Value::Object(obj) => destream_json::Value::Map(
            obj.into_iter()
                .map(|(k, v)| (k.clone(), convert_value_to_value(v)))
                .collect::<HashMap<String, _>>(),
        ),
        serde_json::Value::String(s) => destream_json::Value::String(s.clone()),
    }
}
