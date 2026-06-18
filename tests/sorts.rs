use std::cmp::Ordering;
use txtodo::*;

fn t(desc: &str) -> Task {
    TodoTxtParser::new().parse_line(desc).unwrap()
}

#[test]
fn by_priority_asc() {
    let a = t("(A) A");
    let b = t("(B) B");
    let n = t("None");
    let s = TaskSorts::by_priority(SortDirection::Asc);
    assert_eq!(s(&a, &b), Ordering::Less);
    assert_eq!(s(&b, &a), Ordering::Greater);
    assert_eq!(s(&a, &a), Ordering::Equal);
    // No-priority sorts last in ASC
    assert_eq!(s(&n, &a), Ordering::Greater);
    assert_eq!(s(&a, &n), Ordering::Less);
}

#[test]
fn by_priority_desc() {
    let a = t("(A) A");
    let b = t("(B) B");
    let s = TaskSorts::by_priority(SortDirection::Desc);
    assert_eq!(s(&a, &b), Ordering::Greater);
    assert_eq!(s(&b, &a), Ordering::Less);
}

#[test]
fn by_description_asc() {
    let a = t("apple");
    let b = t("banana");
    let s = TaskSorts::by_description(SortDirection::Asc);
    assert_eq!(s(&a, &b), Ordering::Less);
}

#[test]
fn by_completion_status_asc() {
    let open = t("open");
    let done = t("x 2024-01-01 done");
    let s = TaskSorts::by_completion_status(SortDirection::Asc);
    // Incomplete first
    assert_eq!(s(&open, &done), Ordering::Less);
}

#[test]
fn by_context_count() {
    let a = t("A @x");
    let b = t("B @x @y @z");
    let s = TaskSorts::by_context_count(SortDirection::Asc);
    assert_eq!(s(&a, &b), Ordering::Less);
}

#[test]
fn by_subtask_count() {
    let parent = t("P");
    let mut parent = parent;
    parent.subtasks.push(t("C1"));
    parent.subtasks.push(t("C2"));
    let alone = t("Alone");
    let s = TaskSorts::by_subtask_count(SortDirection::Asc);
    assert_eq!(s(&alone, &parent), Ordering::Less);
}

#[test]
fn composite_orders_by_first_then_second() {
    let a = t("(A) A");
    let b = t("(A) B");
    let c = t("(B) C");
    let s1 = TaskSorts::by_priority(SortDirection::Asc);
    let s2 = TaskSorts::by_description(SortDirection::Asc);
    let s = TaskSorts::composite(&[&s1, &s2]);
    // a < b (same priority A, by description)
    assert_eq!(s(&a, &b), Ordering::Less);
    // a < c (priority A < B)
    assert_eq!(s(&a, &c), Ordering::Less);
}

#[test]
fn then_sugar() {
    let a = t("(A) A");
    let b = t("(B) B");
    let s1 = TaskSorts::by_priority(SortDirection::Asc);
    let s2 = TaskSorts::by_description(SortDirection::Asc);
    let s = TaskSorts::then(&s1, &s2);
    assert_eq!(s(&a, &b), Ordering::Less);
}

#[test]
fn by_extension_field_asc_no_field_sorts_last() {
    let with = t("Task due:2024-01-01");
    let without = t("Task");
    let s = TaskSorts::by_extension_field("due", SortDirection::Asc);
    // Task without 'due' sorts last in ASC
    assert_eq!(s(&with, &without), Ordering::Less);
    assert_eq!(s(&without, &with), Ordering::Greater);
}
