
/// Format a number with commas as thousands separators
pub fn format_count(count: usize) -> String {
    let mut s = count.to_string();
    let l = s.len();
    for i in 1..l {
        if i % 3 == 0 {
            s.insert(l-i, ',');
        }
    }
    s
}

#[test]
fn test_format_count() {
    assert_eq!(&format_count(1), "1");
    assert_eq!(&format_count(12), "12");
    assert_eq!(&format_count(123), "123");
    assert_eq!(&format_count(1234), "1,234");
    assert_eq!(&format_count(12345), "12,345");
    assert_eq!(&format_count(123456), "123,456");
    assert_eq!(&format_count(1234567), "1,234,567");
    assert_eq!(&format_count(12345678), "12,345,678");
    assert_eq!(&format_count(1234567890), "1,234,567,890");
}
