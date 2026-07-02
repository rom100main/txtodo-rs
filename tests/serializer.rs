use std::sync::Arc;
use txtodo::extension::ExtensionHandler;
use txtodo::*;

#[test]
fn round_trip_simple() {
    let s = TodoTxtSerializer::new();
    let line = "Call Mom +Family @phone";
    let p = TodoTxtParser::new();
    let tasks = p.parse_file(line).unwrap();
    let out = s.serialize_tasks(&tasks).unwrap();
    assert_eq!(out, line);
}

#[test]
fn round_trip_with_priority_and_date() {
    let s = TodoTxtSerializer::new();
    let p = TodoTxtParser::new();
    let line = "(A) 2023-10-24 Call Mom +Family @phone";
    let tasks = p.parse_file(line).unwrap();
    let out = s.serialize_tasks(&tasks).unwrap();
    assert_eq!(out, line);
}

#[test]
fn round_trip_completed_with_priority_and_dates() {
    let s = TodoTxtSerializer::new();
    let p = TodoTxtParser::new();
    let line = "x 2023-10-25 (A) 2023-10-24 Call Mom";
    let tasks = p.parse_file(line).unwrap();
    let out = s.serialize_tasks(&tasks).unwrap();
    assert_eq!(out, line);
}

#[test]
fn round_trip_subtask() {
    let s = TodoTxtSerializer::new();
    let p = TodoTxtParser::new();
    let content = "Parent\n    Child";
    let tasks = p.parse_file(content).unwrap();
    let out = s.serialize_tasks(&tasks).unwrap();
    assert_eq!(out, content);
}

#[test]
fn round_trip_with_extension() {
    let s = TodoTxtSerializer::new();
    let p = TodoTxtParser::new();
    let line = "Task due:2024-01-15";
    let tasks = p.parse_file(line).unwrap();
    let out = s.serialize_tasks(&tasks).unwrap();
    assert_eq!(out, line);
}

#[test]
fn round_trip_custom_extension() {
    let mut handler = ExtensionHandler::new();
    handler
        .add_extension(
            TodoTxtExtension::new("estimate")
                .with_parser(Arc::new(|v: &str| {
                    let n: f64 = v.trim_end_matches('h').parse().unwrap();
                    Ok(ExtensionValue::Number(n))
                }))
                .with_serializer(Arc::new(|v: &ExtensionValue| Ok(format!("{v}h")))),
        )
        .unwrap();
    let s = TodoTxtSerializer::with_handler(handler.clone());
    let p = TodoTxtParser::with_handler(handler);
    let line = "Task estimate:2h";
    let tasks = p.parse_file(line).unwrap();
    let out = s.serialize_tasks(&tasks).unwrap();
    assert_eq!(out, line);
}

#[test]
fn completed_no_priority_serialization() {
    let s = TodoTxtSerializer::new();
    let p = TodoTxtParser::new();
    let t = p.parse_line("x 2023-10-25 Task").unwrap();
    let out = s.serialize_task(&t).unwrap();
    assert_eq!(out[0], "x 2023-10-25 Task");
}

#[test]
fn empty_lines_serialized() {
    let s = TodoTxtSerializer::new();
    let p = TodoTxtParser::new();
    let content = "Task 1\n\nTask 2";
    let tasks = p.parse_file(content).unwrap();
    let out = s.serialize_tasks(&tasks).unwrap();
    assert_eq!(out, content);
}

#[test]
fn extension_at_start_not_duplicated() {
    let s = TodoTxtSerializer::new();
    let p = TodoTxtParser::new();
    let line = "n:2 blabla";
    let tasks = p.parse_file(line).unwrap();
    let out = s.serialize_tasks(&tasks).unwrap();
    assert_eq!(out, "n:2 blabla");
}

#[test]
fn extension_in_middle_preserves_position() {
    let s = TodoTxtSerializer::new();
    let p = TodoTxtParser::new();
    let line = "Some text due:2024-01-15 some text";
    let tasks = p.parse_file(line).unwrap();
    let out = s.serialize_tasks(&tasks).unwrap();
    assert_eq!(out, line);
}

#[test]
fn custom_serializer_preserves_position() {
    let mut handler = ExtensionHandler::new();
    handler
        .add_extension(
            TodoTxtExtension::new("estimate")
                .with_parser(Arc::new(|v: &str| {
                    let n: f64 = v.trim_end_matches('h').parse().unwrap();
                    Ok(ExtensionValue::Number(n))
                }))
                .with_serializer(Arc::new(|v: &ExtensionValue| Ok(format!("{v}h")))),
        )
        .unwrap();
    let s = TodoTxtSerializer::with_handler(handler.clone());
    let p = TodoTxtParser::with_handler(handler);
    let line = "Some text estimate:2 more text";
    let tasks = p.parse_file(line).unwrap();
    let out = s.serialize_tasks(&tasks).unwrap();
    assert_eq!(out, "Some text estimate:2h more text");
}

#[test]
fn multiple_extensions_preserve_position() {
    let s = TodoTxtSerializer::new();
    let p = TodoTxtParser::new();
    let line = "Task due:2024-01-15 more text pri:high end";
    let tasks = p.parse_file(line).unwrap();
    let out = s.serialize_tasks(&tasks).unwrap();
    assert_eq!(out, line);
}
