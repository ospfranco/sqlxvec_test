use serde_json::Value as JsonValue;
use sqlx::{sqlite::SqliteValueRef, TypeInfo, Value, ValueRef};

use super::Error;

pub(crate) fn to_json(v: SqliteValueRef) -> Result<JsonValue, Error> {
    if v.is_null() {
        return Ok(JsonValue::Null);
    }

    let res = match v.type_info().name() {
        "TEXT" => {
            // watermelon is shit and doesn't properly create types for the tables
            // sqlx is also shit and relies on this types to do any smart parsing

            // so we are stuck between a rock and a hard place...
            if let Ok(v) = v.to_owned().try_decode::<String>() {
                if let Ok(parsed_number) = v.parse::<i64>() {
                    JsonValue::Number(parsed_number.into())
                } else {
                    JsonValue::String(v)
                }
            } else {
                JsonValue::Null
            }
        }
        "REAL" => {
            if let Ok(v) = v.to_owned().try_decode::<f64>() {
                JsonValue::from(v)
            } else {
                JsonValue::Null
            }
        }
        "INTEGER" | "NUMERIC" => {
            if let Ok(v) = v.to_owned().try_decode::<i64>() {
                JsonValue::Number(v.into())
            } else {
                JsonValue::Null
            }
        }
        "BOOLEAN" => {
            if let Ok(v) = v.to_owned().try_decode() {
                JsonValue::Bool(v)
            } else {
                JsonValue::Null
            }
        }
        // "DATE" => {
        //     if let Ok(v) = v.to_owned().try_decode::<Date>() {
        //         JsonValue::String(v.to_string())
        //     } else {
        //         JsonValue::Null
        //     }
        // }
        // "TIME" => {
        //     if let Ok(v) = v.to_owned().try_decode::<Time>() {
        //         JsonValue::String(v.to_string())
        //     } else {
        //         JsonValue::Null
        //     }
        // }
        // "DATETIME" => {
        //     if let Ok(v) = v.to_owned().try_decode::<PrimitiveDateTime>() {
        //         JsonValue::String(v.to_string())
        //     } else {
        //         JsonValue::Null
        //     }
        // }
        "BLOB" => {
            if let Ok(v) = v.to_owned().try_decode::<Vec<u8>>() {
                JsonValue::Array(v.into_iter().map(|n| JsonValue::Number(n.into())).collect())
            } else {
                JsonValue::Null
            }
        }
        "NULL" => JsonValue::Null,
        _ => return Err(Error::UnsupportedDatatype(v.type_info().name().to_string())),
    };

    Ok(res)
}
