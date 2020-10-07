use {
    toml::{
        value::Table,
        Value,
    },
};


/// try to read the field_name property of the given value, as a string
pub fn string_field(tbl: &Table, field_name: &str) -> Option<String> {
    tbl.get(field_name)
        .and_then(|fv| fv.as_str())
        .map(|s| s.to_string())
}

/// try to read the field_name property of the given value, as an array of strings.
/// Return None if the type isn't compatible
pub fn string_array_field(tbl: &Table, field_name: &str) -> Option<Vec<String>> {
    if let Some(fv) = tbl.get(field_name) {
        if let Value::Array(arr_val) = fv {
            let mut arr = Vec::new();
            for v in arr_val {
                match v.as_str() {
                    Some(s) => {
                        arr.push(s.to_string());
                    }
                    None => {
                        return None; // non matching value
                    }
                }
            }
            return Some(arr);
        }
    }
    None

}

/// try to read the field_name property of the given value, as a boolean
/// (only read it if it's a proper toml boolean, doesn't try to do hazardous
/// string to bool conversions)
pub fn bool_field(tbl: &Table, field_name: &str) -> Option<bool> {
    match tbl.get(field_name) {
        Some(Value::Boolean(b)) => Some(*b),
        _ => None,
    }
}
