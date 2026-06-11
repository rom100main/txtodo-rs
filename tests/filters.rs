use txtodo::extension::ExtensionHandler;
use txtodo::task::build_task_from_line;
use txtodo::*;

fn t(desc: &str) -> Task {
    build_task_from_line(desc, &ExtensionHandler::new(), None).unwrap()
}

#[test]
fn by_context_matches() {
    let task = t("Task @home");
    let f = TaskFilters::by_context("home");
    assert!(f(&task));
    let f = TaskFilters::by_context("work");
    assert!(!f(&task));
}

#[test]
fn by_project_matches() {
    let task = t("Task +ProjectX");
    let f = TaskFilters::by_project("ProjectX");
    assert!(f(&task));
}

#[test]
fn by_priority_matches() {
    let task = t("(A) Task");
    let f = TaskFilters::by_priority(Priority('A'));
    assert!(f(&task));
    let f = TaskFilters::by_priority(Priority('B'));
    assert!(!f(&task));
}

#[test]
fn by_completion_status() {
    let done = t("x 2024-01-01 done");
    let open = t("open");
    let fd = TaskFilters::completed();
    assert!(fd(&done));
    assert!(!fd(&open));
    let fo = TaskFilters::incomplete();
    assert!(!fo(&done));
    assert!(fo(&open));
}

#[test]
fn by_extension_field_presence() {
    let task = t("Task due:2024-01-01");
    let f = TaskFilters::by_extension_field("due", None);
    assert!(f(&task));
    let f = TaskFilters::by_extension_field("missing", None);
    assert!(!f(&task));
}

#[test]
fn by_extension_field_value() {
    let task = t("Task n:42");
    let f = TaskFilters::by_extension_field("n", Some(&ExtensionValue::Number(42.0)));
    assert!(f(&task));
    let f = TaskFilters::by_extension_field("n", Some(&ExtensionValue::Number(99.0)));
    assert!(!f(&task));
}

#[test]
fn has_priority() {
    let p = t("(A) T");
    let n = t("T");
    let f = TaskFilters::has_priority();
    assert!(f(&p));
    assert!(!f(&n));
}

#[test]
fn has_context() {
    let c = t("T @x");
    let n = t("T");
    let f = TaskFilters::has_context();
    assert!(f(&c));
    assert!(!f(&n));
}

#[test]
fn and_combines() {
    let task = t("(A) T @home +proj");
    let f1 = TaskFilters::by_priority(Priority('A'));
    let f2 = TaskFilters::by_context("home");
    let f = TaskFilters::and(&[&f1, &f2]);
    assert!(f(&task));
    let f3 = TaskFilters::by_context("work");
    let f = TaskFilters::and(&[&f1, &f3]);
    assert!(!f(&task));
}

#[test]
fn or_combines() {
    let task = t("(A) T");
    let f1 = TaskFilters::by_priority(Priority('A'));
    let f2 = TaskFilters::by_priority(Priority('B'));
    let f = TaskFilters::or(&[&f1, &f2]);
    assert!(f(&task));
}

#[test]
fn not_inverts() {
    let task = t("T @home");
    let f1 = TaskFilters::by_context("home");
    let f = TaskFilters::not(&f1);
    assert!(!f(&task));
}
