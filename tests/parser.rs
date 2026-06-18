use txtodo::extension::ExtensionHandler;
use txtodo::*;

fn h() -> ExtensionHandler {
    ExtensionHandler::new()
}

fn t(line: &str) -> Task {
    TodoTxtParser::with_handler(h()).parse_line(line).unwrap()
}

fn tp(line: &str, parent: &Task) -> Task {
    TodoTxtParser::with_handler(h())
        .parse_line_with_parent(line, Some(parent))
        .unwrap()
}

#[test]
fn parse_simple_incomplete() {
    let t = t("Call Mom");
    assert!(!t.completed);
    assert_eq!(t.priority, None);
    assert_eq!(t.creation_date, None);
    assert_eq!(t.description, "Call Mom");
}

#[test]
fn parse_priority_and_date() {
    let t = t("(A) 2023-10-24 Call Mom");
    assert_eq!(t.priority, Some(Priority('A')));
    assert!(t.creation_date.is_some());
    assert_eq!(t.description, "Call Mom");
}

#[test]
fn parse_completed() {
    let t = t("x 2023-10-25 (A) 2023-10-24 Call Mom");
    assert!(t.completed);
    assert!(t.completion_date.is_some());
    assert_eq!(t.priority, Some(Priority('A')));
    assert!(t.creation_date.is_some());
}

#[test]
fn parse_completed_no_priority() {
    let t = t("x 2025-01-02 2025-01-01 Completed task 1");
    assert!(t.completed);
    assert!(t.completion_date.is_some());
    assert!(t.creation_date.is_some());
    assert_eq!(t.priority, None);
    assert_eq!(t.description, "Completed task 1");
}

#[test]
fn parse_projects_and_contexts() {
    let t = t("Call +Family @phone");
    assert_eq!(t.projects, vec!["Family".to_string()]);
    assert_eq!(t.contexts, vec!["phone".to_string()]);
}

#[test]
fn parse_string_extension() {
    let t = t("Task name:Romain");
    assert!(t.extensions.contains_key("name"));
    match t.extensions.get("name").unwrap() {
        ExtensionValue::String(s) => assert_eq!(s, "Romain"),
        _ => panic!("expected string"),
    }
}

#[test]
fn parse_number_extension() {
    let t = t("Task n:42");
    match t.extensions.get("n").unwrap() {
        ExtensionValue::Number(n) => assert_eq!(*n, 42.0),
        _ => panic!("expected number"),
    }
}

#[test]
fn parse_boolean_extension() {
    for v in &["true", "false", "yes", "no", "y", "n", "on", "off"] {
        let t = t(&format!("Task b:{v}"));
        assert!(t.extensions.contains_key("b"), "failed for {v}");
    }
}

#[test]
fn parse_date_extension() {
    let t = t("Task due:2024-01-15");
    match t.extensions.get("due").unwrap() {
        ExtensionValue::Date(d) => assert_eq!(d.to_string(), "2024-01-15"),
        _ => panic!("expected date"),
    }
}

#[test]
fn parse_array_extension() {
    let t = t("Task tags:home,work");
    match t.extensions.get("tags").unwrap() {
        ExtensionValue::Array(a) => {
            assert_eq!(a.len(), 2);
            assert!(matches!(a[0], ExtensionValue::String(ref s) if s == "home"));
            assert!(matches!(a[1], ExtensionValue::String(ref s) if s == "work"));
        }
        _ => panic!("expected array"),
    }
}

#[test]
fn parse_bracket_wrapped() {
    let t = t("Task w:(1)");
    match t.extensions.get("w").unwrap() {
        ExtensionValue::Number(n) => assert_eq!(*n, 1.0),
        _ => panic!("expected number"),
    }
}

#[test]
fn parse_quoted_string() {
    let t = t(r#"Task q:"hello""#);
    match t.extensions.get("q").unwrap() {
        ExtensionValue::String(s) => assert_eq!(s, "hello"),
        _ => panic!("expected string"),
    }
}

#[test]
fn parse_nested_brackets() {
    let t = t("Task w:[[2]]");
    match t.extensions.get("w").unwrap() {
        ExtensionValue::Number(n) => assert_eq!(*n, 2.0),
        _ => panic!("expected number"),
    }
}

#[test]
fn subtask_inherits_parent_projects() {
    let parent = t("Parent +Project");
    let child = tp("    Child", &parent);
    assert_eq!(child.projects, vec!["Project".to_string()]);
}

#[test]
fn subtask_does_not_inherit_priority() {
    let parent = t("(A) Parent");
    let child = tp("    Child", &parent);
    assert_eq!(child.priority, None);
}

#[test]
fn custom_extension_with_parser() {
    let mut handler = ExtensionHandler::new();
    handler
        .add_extension(
            TodoTxtExtension::new("estimate").with_parser(std::sync::Arc::new(|v: &str| {
                let n: f64 = v.trim_end_matches('h').parse().unwrap();
                Ok(ExtensionValue::Number(n))
            })),
        )
        .unwrap();
    let p = TodoTxtParser::with_handler(handler);
    let t = p.parse_line("Task estimate:2h").unwrap();
    match t.extensions.get("estimate").unwrap() {
        ExtensionValue::Number(n) => assert_eq!(*n, 2.0),
        _ => panic!("expected number"),
    }
}

#[test]
fn inherit_true_default_copies_extension() {
    let mut handler = ExtensionHandler::new();
    handler.add_extension(TodoTxtExtension::new("due")).unwrap();
    let p = TodoTxtParser::with_handler(handler);
    let parent = p.parse_line("Task due:2024-01-01").unwrap();
    let child = p
        .parse_line_with_parent("    Child", Some(&parent))
        .unwrap();
    assert!(child.extensions.contains_key("due"));
}

#[test]
fn inherit_false_blocks_extension() {
    let mut handler = ExtensionHandler::new();
    handler
        .add_extension(TodoTxtExtension::new("due").inherit(false))
        .unwrap();
    let p = TodoTxtParser::with_handler(handler);
    let parent = p.parse_line("Task due:2024-01-01").unwrap();
    let child = p
        .parse_line_with_parent("    Child", Some(&parent))
        .unwrap();
    assert!(!child.extensions.contains_key("due"));
}

#[test]
fn empty_lines_preserved() {
    let content = "Task 1\n\nTask 2";
    let p = TodoTxtParser::new();
    let tasks = p.parse_file(content).unwrap();
    // empty line becomes an empty placeholder task
    assert_eq!(tasks.len(), 3);
    assert_eq!(tasks[0].raw, "Task 1");
    assert_eq!(tasks[1].raw, "");
    assert_eq!(tasks[2].raw, "Task 2");
}

#[test]
fn subtask_nesting_via_indent() {
    let content = "Parent\n    Child A\n    Child B\n        Grandchild";
    let p = TodoTxtParser::new();
    let tasks = p.parse_file(content).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].subtasks.len(), 2);
    assert_eq!(tasks[0].subtasks[0].description, "Child A");
    assert_eq!(tasks[0].subtasks[1].subtasks.len(), 1);
    assert_eq!(tasks[0].subtasks[1].subtasks[0].description, "Grandchild");
}
