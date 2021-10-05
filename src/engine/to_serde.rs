use destream_json::Value as DValue;
use number_general::{Float as DFloat, Int as DInt, Number, UInt as DUInt};
use serde_json::{value::Number as SNumber, Value as SValue};

fn to_number<T: Into<f64>>(f: T) -> SValue {
    fn to_number_inner<T: Into<f64>>(f: T) -> SValue {
        SValue::Number(SNumber::from_f64(f.into()).expect("Unable to create serde_json::value::Number"))
    }
    let f = f.into();
    if f.is_finite() {
        to_number_inner(f)
    } else if f.is_nan() {
        SValue::Null
    } else if f.is_sign_negative() {
        to_number_inner(f64::MIN)
    } else {
        to_number_inner(f64::MAX)
    }
}

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
        DValue::Number(Number::Float(DFloat::F64(f))) => to_number(*f),
        DValue::Number(Number::Float(DFloat::F32(f))) => to_number(*f),
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
