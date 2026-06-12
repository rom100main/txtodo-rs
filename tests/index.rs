use txtodo::*;

fn todo() -> TodoTxt {
    TodoTxt::new(TodoOptions::default()).unwrap()
}

#[test]
fn add_and_list() {
    let mut t = todo();
    t.add("Task 1").unwrap();
    t.add("Task 2").unwrap();
    let list = t.list();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].description, "Task 1");
    assert_eq!(list[1].description, "Task 2");
}

#[test]
fn add_many() {
    let mut t = todo();
    t.add_many(["A", "B", "C"]).unwrap();
    assert_eq!(t.list().len(), 3);
}

#[test]
fn mark_uses_flattened_indices() {
    let mut t = todo();
    t.add_many(["Parent", "    Child"]).unwrap();
    t.mark([1]).unwrap();
    let list = t.list();
    assert!(!list[0].completed);
    assert!(list[1].completed);
}

#[test]
fn mark_negative_index() {
    let mut t = todo();
    t.add_many(["A", "B", "C"]).unwrap();
    t.mark([-1]).unwrap();
    let list = t.list();
    assert!(!list[0].completed);
    assert!(!list[1].completed);
    assert!(list[2].completed);
}

#[test]
fn unmark_clears_completion() {
    let mut t = todo();
    t.add("x 2024-01-01 done").unwrap();
    t.unmark([0]).unwrap();
    let list = t.list();
    assert!(!list[0].completed);
    assert_eq!(list[0].completion_date, None);
}

#[test]
fn remove_with_positive_index() {
    let mut t = todo();
    t.add_many(["A", "B", "C"]).unwrap();
    t.remove([1]).unwrap();
    let list = t.list();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].description, "A");
    assert_eq!(list[1].description, "C");
}

#[test]
fn remove_with_negative_index() {
    let mut t = todo();
    t.add_many(["A", "B", "C"]).unwrap();
    t.remove([-1]).unwrap();
    let list = t.list();
    assert_eq!(list.len(), 2);
    assert_eq!(list[1].description, "B");
}

#[test]
fn update_changes_description() {
    let mut t = todo();
    t.add("Original").unwrap();
    let patch = TaskPatch {
        description: Some("Updated".to_string()),
        ..Default::default()
    };
    t.update(0, patch).unwrap();
    assert_eq!(t.list()[0].description, "Updated");
}

#[test]
fn update_clears_priority_with_none() {
    let mut t = todo();
    t.add("(A) Task").unwrap();
    let patch = TaskPatch {
        priority: Some(None),
        ..Default::default()
    };
    t.update(0, patch).unwrap();
    assert_eq!(t.list()[0].priority, None);
}

#[test]
fn update_sets_priority() {
    let mut t = todo();
    t.add("Task").unwrap();
    let patch = TaskPatch {
        priority: Some(Some(Priority('B'))),
        ..Default::default()
    };
    t.update(0, patch).unwrap();
    assert_eq!(t.list()[0].priority, Some(Priority('B')));
}

#[test]
fn insert_at_start() {
    let mut t = todo();
    t.add_many(["A", "B", "C"]).unwrap();
    t.insert(0, "First").unwrap();
    let list = t.list();
    assert_eq!(list[0].description, "First");
    assert_eq!(list[1].description, "A");
}

#[test]
fn insert_at_end_via_negative_len() {
    let mut t = todo();
    t.add_many(["A", "B", "C"]).unwrap();
    t.insert(-3, "Z").unwrap();
    let list = t.list();
    assert_eq!(list.len(), 4);
    assert_eq!(list[3].description, "Z");
}

#[test]
fn insert_negative_clamps_to_zero() {
    let mut t = todo();
    t.add_many(["A", "B"]).unwrap();
    t.insert(-10, "First").unwrap();
    assert_eq!(t.list()[0].description, "First");
}

#[test]
fn insert_with_indent_creates_subtask() {
    let mut t = todo();
    t.add_many(["Parent", "Other"]).unwrap();
    t.insert(1, "    Subtask").unwrap();
    let list = t.list();
    assert_eq!(t.tasks.len(), 2);
    assert_eq!(list[0].description, "Parent");
    assert_eq!(list[0].subtasks.len(), 1);
    assert_eq!(list[0].subtasks[0].description, "Subtask");
}

#[test]
fn mark_out_of_bounds_error() {
    let mut t = todo();
    t.add_many(["A", "B", "C"]).unwrap();
    let e = t.mark([5]).unwrap_err();
    assert_eq!(
        e.to_string(),
        "Index out of bounds: 5. Valid range is 0..2 or -3..-1"
    );
}

#[test]
fn filter_returns_tree() {
    let mut t = todo();
    t.add_many(["Alpha", "    Child1", "    Child2", "Beta", "    Child3"])
        .unwrap();
    let f = TaskFilters::by_project("alpha");
    let filtered = t.filter(&f);
    // (project detection is case-sensitive; let's filter by description)
    let _f = TaskFilters::completed();
    let _ = filtered;
    let f2: TaskFilter =
        std::rc::Rc::new(|t: &Task| t.description == "Child1" || t.description == "Alpha");
    let filtered = t.filter(&f2);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].description, "Alpha");
    assert_eq!(filtered[0].subtasks.len(), 1);
    assert_eq!(filtered[0].subtasks[0].description, "Child1");
}

#[test]
fn sort_mutates_in_place() {
    let mut t = todo();
    t.add_many(["Charlie", "Alpha", "Bravo"]).unwrap();
    let s = TaskSorts::by_description(SortDirection::Asc);
    t.sort(&s);
    let list = t.list();
    assert_eq!(list[0].description, "Alpha");
    assert_eq!(list[1].description, "Bravo");
    assert_eq!(list[2].description, "Charlie");
}

#[test]
fn sort_recursive() {
    let mut t = todo();
    t.add_many(["Parent", "    Charlie", "    Alpha", "    Bravo"])
        .unwrap();
    let s = TaskSorts::by_description(SortDirection::Asc);
    t.sort(&s);
    let list = t.list();
    let parent = &list[0];
    assert_eq!(parent.subtasks[0].description, "Alpha");
    assert_eq!(parent.subtasks[1].description, "Bravo");
    assert_eq!(parent.subtasks[2].description, "Charlie");
}

#[test]
fn save_and_load_round_trip() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("txtodo-test-{}.txt", std::process::id()));
    let path_str = path.to_str().unwrap().to_string();

    let mut t = TodoTxt::new(TodoOptions {
        file_path: Some(path_str.clone()),
        ..Default::default()
    })
    .unwrap();
    t.add("(A) 2024-01-15 Call Mom +Family @phone due:2024-01-20")
        .unwrap();
    t.add("    Follow up").unwrap();
    t.save(None).unwrap();

    let mut t2 = TodoTxt::new(TodoOptions {
        file_path: Some(path_str.clone()),
        ..Default::default()
    })
    .unwrap();
    t2.load(None).unwrap();
    let list = t2.list();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].priority, Some(Priority('A')));
    assert!(list[0].creation_date.is_some());
    assert_eq!(list[1].description, "Follow up");

    let _ = std::fs::remove_file(&path);
}
