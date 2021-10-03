use destream_json::Value as DValue;
use number_general::{Float as DFloat, Int as DInt, Number, UInt as DUInt};
use serde_json::{value::Number as SNumber, Value as SValue};

pub fn convert_value_to_value(input: &DValue) -> SValue {
    match input {
        DValue::Bytes(_) => unimplemented!(),
        DValue::List(l) => SValue::Array(l.into_iter().map(convert_value_to_value).collect::<Vec<_>>()),
        DValue::Map(m) => SValue::Object(
            m.into_iter()
                .map(|(k, v)| (k.clone(), convert_value_to_value(v)))
                .collect::<serde_json::value::Map<String, _>>(),
        ),
        DValue::None => SValue::Null,
        DValue::Number(Number::Bool(b)) => SValue::Bool(b.into()),
        DValue::Number(Number::Complex(_)) => unimplemented!(),
        DValue::Number(Number::Float(DFloat::F64(f))) => SValue::Number(SNumber::from_f64(*f).unwrap()),
        DValue::Number(Number::Float(DFloat::F32(f))) => SValue::Number(SNumber::from_f64((*f).into()).unwrap()),
        DValue::Number(Number::Int(DInt::I64(i))) => SValue::Number((*i).into()),
        DValue::Number(Number::Int(DInt::I32(i))) => SValue::Number((*i).into()),
        DValue::Number(Number::Int(DInt::I16(i))) => SValue::Number((*i).into()),
        DValue::Number(Number::Int(DInt::I8(i))) => SValue::Number((*i).into()),
        DValue::Number(Number::UInt(DUInt::U64(i))) => SValue::Number((*i).into()),
        DValue::Number(Number::UInt(DUInt::U32(i))) => SValue::Number((*i).into()),
        DValue::Number(Number::UInt(DUInt::U16(i))) => SValue::Number((*i).into()),
        DValue::Number(Number::UInt(DUInt::U8(i))) => SValue::Number((*i).into()),
        DValue::String(s) => SValue::String(s.clone()),
    }
}
