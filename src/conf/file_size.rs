use::serde::{
    de,
    Deserialize,
};

pub fn parse_file_size(input: &str) -> Result<u64, String> {
    let s = input.to_lowercase();
    let s = s.trim_end_matches('b');
    let (s, binary) = match s.strip_suffix('i') {
        Some(s) => (s, true),
        None => (s, false),
    };
    let cut = s.find(|c: char| !(c.is_ascii_digit() || c=='.'));
    let (digits, factor): (&str, u64) = match cut {
        Some(idx) => (
            &s[..idx],
            match (&s[idx..], binary) {
                ("k", false) => 1000,
                ("k", true) => 1024,
                ("m", false) => 1000*1000,
                ("m", true) => 1024*1024,
                ("g", false) => 1000*1000*1000,
                ("g", true) => 1024*1024*1024,
                ("t", false) => 1000*1000*1000*1000,
                ("t", true) => 1024*1024*1024*1024,
                _ => {
                    // it's not a number
                    return Err(format!("{input:?} can't be parsed as file size"));
                }
            }
        ),
        None => (s, 1),
    };
    match digits.parse::<f64>() {
        Ok(n) => Ok((n * factor as f64).ceil() as u64),
        _ => Err(format!("{input:?} can't be parsed as file size"))
    }
}

#[test]
fn test_parse_file_size(){
    assert_eq!(parse_file_size("33"), Ok(33));
    assert_eq!(parse_file_size("55G"), Ok(55_000_000_000));
    assert_eq!(parse_file_size("2kb"), Ok(2_000));
    assert_eq!(parse_file_size("1.23kiB"), Ok(1260));
}

pub fn deserialize<'de, D>(d: D) -> Result<Option<u64>, D::Error> where D: de::Deserializer<'de> {
    <Option<String> as Deserialize>::deserialize(d)?
        .map(|s| parse_file_size(&s).map_err(de::Error::custom))
        .transpose()
}
