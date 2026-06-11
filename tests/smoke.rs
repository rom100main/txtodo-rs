use txtodo::*;

#[test]
fn smoke_test() {
    let opts = TodoOptions::default();
    let mut t = TodoTxt::new(opts).unwrap();

    t.add("(A) 2025-01-15 Call Mom +Family @phone due:2025-01-20")
        .unwrap();
    t.add("    Schedule follow-up call").unwrap();
    t.add("(B) Pickup +GarageSale @phone").unwrap();

    let list = t.list();
    assert_eq!(list.len(), 3);
    assert_eq!(
        list[0].description,
        "Call Mom +Family @phone due:2025-01-20"
    );
    assert_eq!(list[0].priority, Some(Priority('A')));
    assert!(list[0].projects.contains(&"Family".to_string()));
    assert!(list[0].contexts.contains(&"phone".to_string()));
    assert!(list[0].extensions.contains_key("due"));
}
