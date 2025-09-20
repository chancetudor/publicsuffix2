use publicsuffix2::{Error};

#[test]
fn error_enum_compiles() {
    let _ = Error::EmptyList;
}
