use txtodo::extension::ExtensionHandler;
use txtodo::task::build_task_from_line;
use txtodo::*;

fn h() -> ExtensionHandler {
    ExtensionHandler::new()
}

#[test]
fn parse_simple_incomplete() {
    let t = build_task_from_line("Call Mom", &h(), None).unwrap();
    assert!(!t.completed);
    assert_eq!(t.priority, None);
    assert_eq!(t.creation_date, None);
    assert_eq!(t.description, "Call Mom");
}

#[test]
fn parse_priority_and_date() {
    let t = build_task_from_line("(A) 2023-10-24 Call Mom", &h(), None).unwrap();
    assert_eq!(t.priority, Some(Priority('A')));
    assert!(t.creation_date.is_some());
    assert_eq!(t.description, "Call Mom");
}

#[test]
fn parse_completed() {
    let t = build_task_from_line("x 2023-10-25 (A) 2023-10-24 Call Mom", &h(), None).unwrap();
    assert!(t.completed);
    assert!(t.completion_date.is_some());
    assert_eq!(t.priority, Some(Priority('A')));
    assert!(t.creation_date.is_some());
}

#[test]
fn parse_completed_no_priority() {
    let t = build_task_from_line("x 2025-01-02 2025-01-01 Completed task 1", &h(), None).unwrap();
    assert!(t.completed);
    assert!(t.completion_date.is_some());
    assert!(t.creation_date.is_some());
    assert_eq!(t.priority, None);
    assert_eq!(t.description, "Completed task 1");
}

#[test]
fn parse_projects_and_contexts() {
    let t = build_task_from_line("Call +Family @phone", &h(), None).unwrap();
    assert_eq!(t.projects, vec!["Family".to_string()]);
    assert_eq!(t.contexts, vec!["phone".to_string()]);
}

#[test]
fn parse_string_extension() {
    let t = build_task_from_line("Task name:Romain", &h(), None).unwrap();
    assert!(t.extensions.contains_key("name"));
    match t.extensions.get("name").unwrap() {
        ExtensionValue::String(s) => assert_eq!(s, "Romain"),
        _ => panic!("expected string"),
    }
}

#[test]
fn parse_number_extension() {
    let t = build_task_from_line("Task n:42", &h(), None).unwrap();
    match t.extensions.get("n").unwrap() {
        ExtensionValue::Number(n) => assert_eq!(*n, 42.0),
        _ => panic!("expected number"),
    }
}

#[test]
fn parse_boolean_extension() {
    for v in &["true", "false", "yes", "no", "y", "n", "on", "off"] {
        let t = build_task_from_line(&format!("Task b:{v}"), &h(), None).unwrap();
        assert!(t.extensions.contains_key("b"), "failed for {v}");
    }
}

#[test]
fn parse_date_extension() {
    let t = build_task_from_line("Task due:2024-01-15", &h(), None).unwrap();
    match t.extensions.get("due").unwrap() {
        ExtensionValue::Date(d) => assert_eq!(d.to_string(), "2024-01-15"),
        _ => panic!("expected date"),
    }
}

#[test]
fn parse_array_extension() {
    let t = build_task_from_line("Task tags:home,work", &h(), None).unwrap();
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
    let t = build_task_from_line("Task w:(1)", &h(), None).unwrap();
    match t.extensions.get("w").unwrap() {
        ExtensionValue::Number(n) => assert_eq!(*n, 1.0),
        _ => panic!("expected number"),
    }
}

#[test]
fn parse_quoted_string() {
    let t = build_task_from_line(r#"Task q:"hello""#, &h(), None).unwrap();
    match t.extensions.get("q").unwrap() {
        ExtensionValue::String(s) => assert_eq!(s, "hello"),
        _ => panic!("expected string"),
    }
}

#[test]
fn parse_nested_brackets() {
    let t = build_task_from_line("Task w:[[2]]", &h(), None).unwrap();
    match t.extensions.get("w").unwrap() {
        ExtensionValue::Number(n) => assert_eq!(*n, 2.0),
        _ => panic!("expected number"),
    }
}

#[test]
fn subtask_inherits_parent_projects() {
    let parent = build_task_from_line("Parent +Project", &h(), None).unwrap();
    let child = build_task_from_line("    Child", &h(), Some(&parent)).unwrap();
    assert_eq!(child.projects, vec!["Project".to_string()]);
}

#[test]
fn subtask_does_not_inherit_priority() {
    let parent = build_task_from_line("(A) Parent", &h(), None).unwrap();
    let child = build_task_from_line("    Child", &h(), Some(&parent)).unwrap();
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
    let t = build_task_from_line("Task estimate:2h", &handler, None).unwrap();
    match t.extensions.get("estimate").unwrap() {
        ExtensionValue::Number(n) => assert_eq!(*n, 2.0),
        _ => panic!("expected number"),
    }
}

#[test]
fn inherit_true_default_copies_extension() {
    let mut handler = ExtensionHandler::new();
    handler.add_extension(TodoTxtExtension::new("due")).unwrap();
    let parent = build_task_from_line("Task due:2024-01-01", &handler, None).unwrap();
    let child = build_task_from_line("    Child", &handler, Some(&parent)).unwrap();
    assert!(child.extensions.contains_key("due"));
}

#[test]
fn inherit_false_blocks_extension() {
    let mut handler = ExtensionHandler::new();
    handler
        .add_extension(TodoTxtExtension::new("due").inherit(false))
        .unwrap();
    let parent = build_task_from_line("Task due:2024-01-01", &handler, None).unwrap();
    let child = build_task_from_line("    Child", &handler, Some(&parent)).unwrap();
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
