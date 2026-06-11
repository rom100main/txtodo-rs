use txtodo::error::TxtodoError;
use txtodo::*;

#[test]
fn out_of_bounds_empty_list() {
    let r = resolve_index(0, 0);
    let e = r.unwrap_err();
    assert_eq!(
        e.to_string(),
        "Index out of bounds: 0. Valid range is 0..-1 or -0..-1"
    );
}

#[test]
fn out_of_bounds_three_tasks() {
    let e = resolve_index(5, 3).unwrap_err();
    assert_eq!(
        e.to_string(),
        "Index out of bounds: 5. Valid range is 0..2 or -3..-1"
    );
    let e = resolve_index(-5, 3).unwrap_err();
    assert_eq!(
        e.to_string(),
        "Index out of bounds: -5. Valid range is 0..2 or -3..-1"
    );
}

#[test]
fn negative_index_wraps() {
    assert_eq!(resolve_index(-1, 3).unwrap(), 2);
    assert_eq!(resolve_index(-3, 3).unwrap(), 0);
}

#[test]
fn positive_index_in_range() {
    assert_eq!(resolve_index(0, 3).unwrap(), 0);
    assert_eq!(resolve_index(2, 3).unwrap(), 2);
}

#[test]
fn error_code_values() {
    let e = TxtodoError::Parse {
        message: "x".to_string(),
        line: None,
        line_number: None,
    };
    assert_eq!(e.code(), "PARSE_ERROR");
    let e = TxtodoError::Extension {
        message: "x".to_string(),
        extension_key: None,
    };
    assert_eq!(e.code(), "EXTENSION_ERROR");
    let e = TxtodoError::Generic("x".to_string());
    assert_eq!(e.code(), "");
}

#[test]
fn priority_from_token() {
    assert_eq!(Priority::from_token("A").unwrap(), Priority('A'));
    assert_eq!(Priority::from_token("(A)").unwrap(), Priority('A'));
    assert!(Priority::from_token("a").is_err());
    assert!(Priority::from_token("AA").is_err());
    assert!(Priority::from_token("(AB)").is_err());
}

#[test]
fn priority_display() {
    assert_eq!(Priority('A').to_string(), "(A)");
    assert_eq!(Priority('Z').to_string(), "(Z)");
}

#[test]
fn extension_value_strict_equality() {
    assert!(ExtensionValue::String("1".into()).equals(&ExtensionValue::String("1".into())));
    assert!(!ExtensionValue::String("1".into()).equals(&ExtensionValue::Number(1.0)));
    assert!(ExtensionValue::Number(1.0).equals(&ExtensionValue::Number(1.0)));
    assert!(!ExtensionValue::Boolean(true).equals(&ExtensionValue::String("true".into())));
}

#[test]
fn duplicate_extension_rejected() {
    let h = ExtensionHandler::new();
    let mut h2 = h.clone();
    h2.add_extension(TodoTxtExtension::new("due")).unwrap();
    let r = h2.add_extension(TodoTxtExtension::new("due"));
    assert!(r.is_err());
    assert_eq!(r.unwrap_err().code(), "EXTENSION_ERROR");
}

#[test]
fn remove_unknown_extension_errors() {
    let mut h = ExtensionHandler::new();
    let r = h.remove_extension("nope");
    assert!(r.is_err());
    assert_eq!(r.unwrap_err().code(), "EXTENSION_ERROR");
}

#[test]
fn remove_empty_key_errors() {
    let mut h = ExtensionHandler::new();
    let r = h.remove_extension("");
    assert!(r.is_err());
    assert_eq!(r.unwrap_err().code(), "VALIDATION_ERROR");
}
