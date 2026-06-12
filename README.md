# TxTodo

A Rust todo.txt parser/serializer with extension support and subtask handling.

A TypeScript version is available at [txtodo-ts](https://github.com/rom100main/txtodo).

## Features

- Parse and serialize todo.txt format (priorities, dates, projects, contexts)
- Custom key:value extensions with automatic parsing and typing
- Subtask support with indentation-based hierarchy
- Task management utils (list, add, insert, remove, mark/unmark, update, sort, filter)
- Property inheritance from parent to subtasks with configurable inheritance control
- Standalone `TodoTxtParser` and `TodoTxtSerializer` structs
- Built-in task filters and sorting functions
- Comprehensive error handling with specific error variants

## Installation

```bash
cargo add txtodo
```

## Quick Start

```rust
use txtodo::*;

fn main() -> Result<(), TxtodoError> {
    let opts = TodoOptions {
        file_path: Some("todo.txt".into()),
        handle_subtasks: true,
        ..Default::default()
    };
    let mut todo = TodoTxt::new(opts)?;

    // Load existing tasks
    todo.load(None)?;

    // Add new tasks
    todo.add("(A) 2025-01-15 Call Mom +Family @phone due:2025-01-20")?;
    todo.add("    Schedule follow-up call")?;
    todo.add("(B) Schedule Goodwill pickup +GarageSale @phone")?;

    // List tasks with filters and sorting
    let filter = TaskFilters::incomplete();
    let sorter = TaskSorts::by_priority(SortDirection::Asc);
    let tasks = todo.list_filtered(Some(&filter), Some(&sorter));

    // Mark task as complete (0-indexed, supports negative indices)
    todo.mark([0])?;

    // Update task
    let patch = TaskPatch {
        priority: Some(Some(Priority('B'))),
        ..Default::default()
    };
    todo.update(0, patch)?;

    // Save changes
    todo.save(None)?;

    Ok(())
}
```

## API Reference

### TodoTxt

Main struct for managing todo.txt tasks.

#### Constructor

```rust
TodoTxt::new(options: TodoOptions) -> Result<Self, TxtodoError>

struct TodoOptions {
    file_path: Option<String>,          // File path for load/save
    auto_save: bool,                    // Auto-save after changes (default: false)
    extensions: Vec<TodoTxtExtension>,  // List of extensions
    handle_subtasks: bool,              // Handle subtasks (default: true)
}
```

#### Methods

```rust
load(&mut self, file_path: Option<&str>) -> Result<(), TxtodoError>;
save(&self, file_path: Option<&str>) -> Result<(), TxtodoError>;
list(&self) -> Vec<&Task>;
list_filtered(&self, filter: Option<&TaskFilter>, sorter: Option<&TaskSorter>) -> Vec<&Task>;
add(&mut self, input: impl Into<TaskInput>) -> Result<(), TxtodoError>;
add_many(&mut self, inputs: impl IntoIterator<Item = impl Into<TaskInput>>) -> Result<(), TxtodoError>;
insert(&mut self, index: i64, input: impl Into<TaskInput>) -> Result<(), TxtodoError>;
remove(&mut self, numbers: impl IntoIterator<Item = i64>) -> Result<(), TxtodoError>;
mark(&mut self, numbers: impl IntoIterator<Item = i64>) -> Result<(), TxtodoError>;
unmark(&mut self, numbers: impl IntoIterator<Item = i64>) -> Result<(), TxtodoError>;
update(&mut self, index: i64, patch: TaskPatch) -> Result<(), TxtodoError>;
update_many(&mut self, updates: impl IntoIterator<Item = (i64, TaskPatch)>) -> Result<(), TxtodoError>;
filter(&self, filter: &TaskFilter) -> Vec<Task>;
sort(&mut self, sorter: &TaskSorter);
set_auto_save(&mut self, on: bool);
```

### Task Struct

```rust
struct Task {
    raw: String,                                   // Original line
    completed: bool,                               // Task completion status
    priority: Option<Priority>,                    // A-Z priority
    creation_date: Option<time::Date>,             // Task creation date
    completion_date: Option<time::Date>,           // Task completion date
    description: String,                           // Task description
    projects: Vec<String>,                         // +project tags
    contexts: Vec<String>,                         // @context tags
    extensions: IndexMap<String, ExtensionValue>,  // Custom extensions
    subtasks: Vec<Task>,                           // Nested subtasks
    indent_level: usize,                           // Indentation level
    parent: Option<NonNull<Task>>,                 // Parent task reference
}
```

### TodoTxtExtension

```rust
let ext = TodoTxtExtension::new("due")
    .with_parser(Arc::new(|value: &str| {
        let d = time::Date::parse(value, &time::format_description::parse("[year]-[month]-[day]")?)?;
        Ok(ExtensionValue::Date(d))
    }))
    .with_serializer(Arc::new(|value: &ExtensionValue| {
        match value {
            ExtensionValue::Date(d) => Ok(format!("{d}")),
            _ => Err(TxtodoError::Serialization { message: "expected date".into() }),
        }
    }))
    .inherit(true)   // Inherit by subtasks (default: true)
    .shadow(true);   // Override parent value (default: true)
```

**Auto-detection and parsing**:
Extensions are automatically detected from `key:value` patterns in task text.
When no custom parser is provided, the library attempts automatic type detection:
- Date-like strings -> `DateExtension`
- Numbers -> `NumberExtension`
- "true"/"false", "yes"/"no", "y"/"n", "on"/"off" -> `BooleanExtension`
- Comma-separated values -> `ArrayExtension`
- Everything else -> `StringExtension`

#### Extension Value Types

```rust
enum ExtensionValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Date(time::Date),
    Array(Vec<ExtensionValue>),
}
```

### Task Filters

```rust
// Basic filters
TaskFilters::completed();
TaskFilters::incomplete();
TaskFilters::by_priority(Priority('A'));
TaskFilters::by_project("Work");
TaskFilters::by_context("@home");

// Date filters
TaskFilters::created_after(date);
TaskFilters::completed_on(date);

// Extension filters
TaskFilters::by_extension_field("due", Some(&ExtensionValue::Date(date)));

// Logical combinations
TaskFilters::and(&[&TaskFilters::incomplete(), &TaskFilters::by_project("Work")]);
TaskFilters::or(&[&TaskFilters::by_priority(Priority('A')), &TaskFilters::by_priority(Priority('B'))]);
TaskFilters::not(&filter);
```

### Task Sorting

```rust
// Basic sorting
TaskSorts::by_priority(SortDirection::Asc);
TaskSorts::by_date_created(SortDirection::Desc);
TaskSorts::by_description(SortDirection::Asc);

// Advanced sorting
TaskSorts::by_extension_field("due", SortDirection::Asc);
TaskSorts::by_subtask_count(SortDirection::Desc);

// Composite sorting
TaskSorts::composite(&[
    &TaskSorts::by_completion_status(SortDirection::Asc),
    &TaskSorts::by_priority(SortDirection::Asc),
    &TaskSorts::by_date_created(SortDirection::Desc),
]);
```

### TodoTxtParser

Independent parser for converting todo.txt text to Task objects. Can be used standalone without the full `TodoTxt` struct.

```rust
use txtodo::*;

let handler = ExtensionHandler::new();
let parser = TodoTxtParser::with_handler(handler);

// Parse full file content
let tasks = parser.parse_file(&content)?;
```

#### Methods

```rust
new() -> Self;
with_handler(handler: ExtensionHandler) -> Self;
handler(&self) -> &ExtensionHandler;
parse_file(&self, content: &str) -> Result<Vec<Task>, TxtodoError>;
```

### TodoTxtSerializer

Independent serializer for converting Task objects back to todo.txt format.

```rust
use txtodo::*;

let handler = ExtensionHandler::new();
let serializer = TodoTxtSerializer::with_handler(handler);

// Serialize single task (returns lines including subtasks)
let lines: Vec<String> = serializer.serialize_task(&task)?;

// Serialize task array
let content = serializer.serialize_tasks(&tasks)?;
```

#### Methods

```rust
new() -> Self;
with_handler(handler: ExtensionHandler) -> Self;
handler(&self) -> &ExtensionHandler;
serialize_task(&self, task: &Task) -> Result<Vec<String>, TxtodoError>;
serialize_tasks(&self, tasks: &[Task]) -> Result<String, TxtodoError>;
```

### Error Handling

The library uses `thiserror` and provides a unified error enum:

```rust
enum TxtodoError {
    Parse { message, line, line_number },
    Extension { message, extension_key },
    Serialization { message },
    Validation { message, field },
    Date { message, date_str },
    Priority { message, priority },
    Generic(String),
    Io(std::io::Error),
    DateParse(time::error::Parse),
}
```

Each variant has a `code()` method returning a string identifier (e.g. `"PARSE_ERROR"`, `"EXTENSION_ERROR"`).

## Support

For bug reports and feature requests, please fill an issue at [GitHub repository](https://github.com/rom100main/todotxt-ts/issues).

## Changelog

See [CHANGELOG](CHANGELOG.md) for a list of changes in each version.

## Development

For development information, see [CONTRIBUTING](CONTRIBUTING.md).

## License

MIT
